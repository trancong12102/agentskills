#!/usr/bin/env bash
set -euo pipefail

MODEL="gemini-3.1-pro-preview"
APPROVAL_MODE="plan"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

usage() {
  cat <<'EOF'
Usage:
  gemini-review.sh branch [--base <branch>] [options]
  gemini-review.sh uncommitted [options]
  gemini-review.sh commit <SHA> [options]
  gemini-review.sh pr <PR_NUMBER> [options]

Subcommands:
  branch       Review current branch diff against a base branch
  uncommitted  Review staged, unstaged, and untracked changes
  commit       Review changes introduced by a specific commit
  pr           Checkout and review a GitHub Pull Request

Options:
  --base <branch>         Base branch for comparison (default: main)
  --focus <text>          Narrow the review to specific concerns
  --context-file <path>   Add extra context file (repeatable)
  --format <format>       Output format: markdown (default) or structured (YAML for LLM consumption)
  --dry-run               Print the prompt without calling Gemini
  --interactive           Keep Gemini chat open after review

Notes:
  - Model is fixed to gemini-3.1-pro-preview
  - Always runs in read-only mode (--approval-mode plan)
EOF
}

fail() {
  echo "Error: $*" >&2
  exit 1
}

require_gemini() {
  if ! command -v gemini >/dev/null 2>&1; then
    fail "Gemini CLI not found in PATH. Install and authenticate Gemini CLI before using this skill."
  fi
}

require_gh() {
  if ! command -v gh >/dev/null 2>&1; then
    fail "GitHub CLI (gh) not found in PATH. Install it to review PRs."
  fi
}

get_branch_diff() {
  local base="$1"
  local diff
  diff="$(git diff "${base}"...HEAD 2>/dev/null)" || fail "Failed to get diff against '${base}'. Is '${base}' a valid branch?"
  if [[ -z "$diff" ]]; then
    fail "No changes found between '${base}' and HEAD."
  fi
  printf '%s' "$diff"
}

get_uncommitted_diff() {
  local diff
  diff="$(git diff HEAD 2>/dev/null)"
  local staged
  staged="$(git diff --staged 2>/dev/null)"
  local -a untracked_files=()
  while IFS= read -r f; do
    [[ -n "$f" ]] && untracked_files+=("$f")
  done < <(git ls-files --others --exclude-standard 2>/dev/null)

  local result=""
  if [[ -n "$staged" ]]; then
    result+="=== STAGED CHANGES ==="$'\n'"$staged"
    result+=$'\n\n'
  fi
  if [[ -n "$diff" ]]; then
    result+="=== UNSTAGED CHANGES ==="$'\n'"$diff"
    result+=$'\n\n'
  fi
  if [[ "${#untracked_files[@]}" -gt 0 ]]; then
    result+="=== UNTRACKED FILES (NEW) ==="$'\n'
    for f in "${untracked_files[@]}"; do
      result+="--- new file: ${f} ---"$'\n'
      if file --brief "$f" 2>/dev/null | grep -q text; then
        result+="$(cat "$f")"$'\n'
      else
        result+="(binary file, skipped)"$'\n'
      fi
      result+=$'\n'
    done
  fi

  if [[ -z "$result" ]]; then
    fail "No uncommitted changes found."
  fi
  printf '%s' "$result"
}

get_commit_diff() {
  local sha="$1"
  local diff
  diff="$(git show --format="%H %s%n%b" "$sha" 2>/dev/null)" || fail "Failed to get commit '${sha}'."
  printf '%s' "$diff"
}

get_pr_diff() {
  local pr_number="$1"
  require_gh
  gh pr checkout "$pr_number" 2>/dev/null || fail "Failed to checkout PR #${pr_number}."
  local base
  base="$(gh pr view "$pr_number" --json baseRefName -q '.baseRefName' 2>/dev/null)" || base="main"
  local diff
  diff="$(git diff "${base}"...HEAD 2>/dev/null)" || fail "Failed to get PR diff."
  local pr_body
  pr_body="$(gh pr view "$pr_number" --json title,body -q '"PR #\(.number // ""): \(.title // "")\n\(.body // "")"' 2>/dev/null)" || pr_body=""

  local result=""
  if [[ -n "$pr_body" ]]; then
    result+="=== PR DESCRIPTION ==="$'\n'"$pr_body"$'\n\n'
  fi
  result+="$diff"
  printf '%s' "$result"
}

build_context_block() {
  local context=""
  local file
  for file in "$@"; do
    [[ -f "$file" ]] || fail "Context file not found: $file"
    context+=$'\n'"----- BEGIN: ${file} -----"$'\n'
    context+="$(cat "$file")"
    context+=$'\n'"----- END: ${file} -----"$'\n'
  done
  printf '%s' "$context"
}

build_common_preamble() {
  local focus="$1"
  local context_block="$2"

  local focus_block=""
  if [[ -n "$focus" ]]; then
    focus_block="
REVIEW FOCUS:
Pay special attention to: ${focus}
"
  fi

  local context_section=""
  if [[ -n "$context_block" ]]; then
    context_section="
ADDITIONAL CONTEXT:
${context_block}
"
  fi

  cat <<EOF
You are a senior code reviewer performing a thorough, professional code review.

INSTRUCTIONS:
Analyze the code changes below. Evaluate each change against the following criteria:

1. **Correctness**: Does the code achieve its stated purpose without bugs or logical errors?
2. **Security**: Are there potential security vulnerabilities (injection, XSS, auth bypass, secrets exposure)?
3. **Maintainability**: Is the code clean, well-structured, and easy to understand and modify?
4. **Efficiency**: Are there obvious performance bottlenecks or resource inefficiencies?
5. **Edge Cases**: Does the code handle edge cases and errors appropriately?
6. **Testability**: Is the code adequately testable? Suggest missing test cases if relevant.
${focus_block}
CONSTRAINTS:
- You are in read-only mode. Do not modify any files.
- You may use read-only tools to explore the codebase for additional context.
- Cite specific file paths and line numbers when referencing issues.
- Be constructive and explain *why* a change is needed, not just *what* to change.
${context_section}
EOF
}

build_review_prompt() {
  local diff="$1"
  local focus="$2"
  local context_block="$3"
  local format="${4:-markdown}"

  local preamble
  preamble="$(build_common_preamble "$focus" "$context_block")"

  case "$format" in
    markdown) build_markdown_prompt "$preamble" "$diff" ;;
    structured) build_structured_prompt "$preamble" "$diff" ;;
    *) fail "Unknown format '${format}'. Use: markdown, structured." ;;
  esac
}

build_markdown_prompt() {
  local preamble="$1"
  local diff="$2"

  cat <<EOF
${preamble}
OUTPUT FORMAT:
Return your review in this exact Markdown structure:

## Review

**Verdict: <verdict>**

Where verdict is one of: "Approved", "Approved with suggestions", "Request Changes"

### Summary
A 2-3 sentence high-level overview of the changes and their quality.

### Changes Walkthrough

| File | Changes |
|------|---------|
| \`path/to/file.ts\` | Brief description of changes in this file |

### Findings

All findings sorted by severity (critical first). Each finding MUST include an explicit severity tag. Use this exact format:

#### <SEVERITY_EMOJI> <Short title>

**<Category>** · \`file/path.ts:LINE\`

Explanation of the issue and why it matters.

**Suggested fix:**
\`\`\`lang
code suggestion here
\`\`\`

Severity emoji mapping (use exactly these):
- 🔴 Critical — Exploitable vulnerability, data loss, or crash in production
- 🟠 High — Likely bug or incident under realistic conditions
- 🟡 Medium — Incorrect behavior under edge cases or degraded performance
- 🟢 Low — Code quality issue that could escalate over time
- 🔵 Info — Observation or suggestion, no action required

Category is one of: Bug, Security, Performance, Maintainability, Edge Case, Testing, Style

If no findings: "No issues found."

### Highlights
1-3 positive observations worth calling out. Skip if nothing stands out.

### Verdict
Restate the verdict with a one-sentence justification.

CODE CHANGES TO REVIEW:
${diff}
EOF
}

build_structured_prompt() {
  local preamble="$1"
  local diff="$2"

  cat <<EOF
${preamble}
OUTPUT FORMAT:
Return your review as a YAML document. This output will be consumed by another LLM for synthesis, so strict adherence to the schema is critical.

Return ONLY the YAML block below — no prose, no markdown fences, no explanation outside the YAML.

verdict: approved | approved_with_suggestions | request_changes
summary: |
  2-3 sentence high-level overview.
changes:
  - file: path/to/file.ts
    description: Brief description of changes
findings:
  - severity: critical | high | medium | low | info
    category: bug | security | performance | maintainability | edge_case | testing | style
    file: path/to/file.ts
    line: 42
    title: Short title
    description: |
      Explanation of the issue and why it matters.
    suggestion: |
      code fix here (optional, omit key if no suggestion)
highlights:
  - Short description of a positive pattern

Field definitions:
- severity: critical = exploitable vulnerability/data loss/crash, high = likely bug under realistic conditions, medium = edge case or perf issue, low = quality issue that could escalate, info = observation only
- category: bug, security, performance, maintainability, edge_case, testing, style
- findings: empty list [] if no issues found
- highlights: empty list [] if nothing stands out
- suggestion: omit this key entirely if no code fix to suggest

CODE CHANGES TO REVIEW:
${diff}
EOF
}

run_gemini() {
  local prompt="$1"
  local interactive="$2"
  local dry_run="$3"

  if [[ "$dry_run" -eq 1 ]]; then
    echo "=== DRY RUN ==="
    echo "Model: $MODEL"
    echo "Approval mode: $APPROVAL_MODE"
    echo ""
    echo "----- BEGIN PROMPT -----"
    printf '%s\n' "$prompt"
    echo "----- END PROMPT -----"
    return 0
  fi

  local -a cmd=(gemini --model "$MODEL" --approval-mode "$APPROVAL_MODE")

  if [[ "$interactive" -eq 1 ]]; then
    cmd+=(--prompt-interactive "$prompt")
  else
    cmd+=(--prompt "$prompt")
  fi

  "${cmd[@]}"
}

handle_branch() {
  local base="main"
  local focus=""
  local format="markdown"
  local interactive=0
  local dry_run=0
  local -a context_files=()

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --base) base="$2"; shift 2 ;;
      --focus) focus="$2"; shift 2 ;;
      --format) format="$2"; shift 2 ;;
      --context-file) context_files+=("$2"); shift 2 ;;
      --interactive) interactive=1; shift ;;
      --dry-run) dry_run=1; shift ;;
      -h|--help) usage; exit 0 ;;
      *) fail "Unknown option: $1" ;;
    esac
  done

  local diff context_block="" prompt
  diff="$(get_branch_diff "$base")"
  if [[ "${#context_files[@]}" -gt 0 ]]; then
    context_block="$(build_context_block "${context_files[@]}")"
  fi
  prompt="$(build_review_prompt "$diff" "$focus" "$context_block" "$format")"
  run_gemini "$prompt" "$interactive" "$dry_run"
}

handle_uncommitted() {
  local focus=""
  local format="markdown"
  local interactive=0
  local dry_run=0
  local -a context_files=()

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --focus) focus="$2"; shift 2 ;;
      --format) format="$2"; shift 2 ;;
      --context-file) context_files+=("$2"); shift 2 ;;
      --interactive) interactive=1; shift ;;
      --dry-run) dry_run=1; shift ;;
      -h|--help) usage; exit 0 ;;
      *) fail "Unknown option: $1" ;;
    esac
  done

  local diff context_block="" prompt
  diff="$(get_uncommitted_diff)"
  if [[ "${#context_files[@]}" -gt 0 ]]; then
    context_block="$(build_context_block "${context_files[@]}")"
  fi
  prompt="$(build_review_prompt "$diff" "$focus" "$context_block" "$format")"
  run_gemini "$prompt" "$interactive" "$dry_run"
}

handle_commit() {
  local sha="$1"; shift
  local focus=""
  local format="markdown"
  local interactive=0
  local dry_run=0
  local -a context_files=()

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --focus) focus="$2"; shift 2 ;;
      --format) format="$2"; shift 2 ;;
      --context-file) context_files+=("$2"); shift 2 ;;
      --interactive) interactive=1; shift ;;
      --dry-run) dry_run=1; shift ;;
      -h|--help) usage; exit 0 ;;
      *) fail "Unknown option: $1" ;;
    esac
  done

  local diff context_block="" prompt
  diff="$(get_commit_diff "$sha")"
  if [[ "${#context_files[@]}" -gt 0 ]]; then
    context_block="$(build_context_block "${context_files[@]}")"
  fi
  prompt="$(build_review_prompt "$diff" "$focus" "$context_block" "$format")"
  run_gemini "$prompt" "$interactive" "$dry_run"
}

handle_pr() {
  local pr_number="$1"; shift
  local focus=""
  local format="markdown"
  local interactive=0
  local dry_run=0
  local -a context_files=()

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --focus) focus="$2"; shift 2 ;;
      --format) format="$2"; shift 2 ;;
      --context-file) context_files+=("$2"); shift 2 ;;
      --interactive) interactive=1; shift ;;
      --dry-run) dry_run=1; shift ;;
      -h|--help) usage; exit 0 ;;
      *) fail "Unknown option: $1" ;;
    esac
  done

  local diff context_block="" prompt
  diff="$(get_pr_diff "$pr_number")"
  if [[ "${#context_files[@]}" -gt 0 ]]; then
    context_block="$(build_context_block "${context_files[@]}")"
  fi
  prompt="$(build_review_prompt "$diff" "$focus" "$context_block" "$format")"
  run_gemini "$prompt" "$interactive" "$dry_run"
}

main() {
  require_gemini
  [[ $# -gt 0 ]] || { usage; exit 1; }

  local subcommand="$1"; shift

  case "$subcommand" in
    branch) handle_branch "$@" ;;
    uncommitted) handle_uncommitted "$@" ;;
    commit)
      [[ $# -ge 1 ]] || fail "Usage: gemini-review.sh commit <SHA> [options]"
      handle_commit "$@"
      ;;
    pr)
      [[ $# -ge 1 ]] || fail "Usage: gemini-review.sh pr <PR_NUMBER> [options]"
      handle_pr "$@"
      ;;
    -h|--help|help) usage ;;
    *) fail "Unknown subcommand '$subcommand'. Use: branch, uncommitted, commit, pr." ;;
  esac
}

main "$@"
