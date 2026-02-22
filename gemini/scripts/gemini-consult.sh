#!/usr/bin/env bash
set -euo pipefail

MODEL="gemini-3.1-pro-preview"
APPROVAL_MODE="plan"

usage() {
  cat <<'EOF'
Usage:
  gemini-consult.sh ask --mode <mode> --task <text> [options]
  gemini-consult.sh followup --prompt <text> [options]
  gemini-consult.sh sessions
  gemini-consult.sh delete-session <index>

Subcommands:
  ask            Start a new structured consultation, or continue one with --resume.
  followup       Continue an existing conversation with a focused follow-up prompt.
  sessions       List saved Gemini sessions for the current project.
  delete-session Delete a saved session by index.

ask options:
  --mode <value>          One of: decision, plan, debug, problem-solving, pre-implement, frontend
  --task <text>           Core task/question for Gemini
  --context-file <path>   Add context file (repeatable)
  --prompt-file <path>    Add extra prompt instructions from file
  --extra <text>          Extra guidance
  --implementation-package
                          Force full implementation package output (file tree + full files + tests + runbook) for non-frontend modes
  --resume <id>           Resume an existing session (e.g. latest, 3)
  --interactive           Send prompt and keep interactive chat open
  --dry-run               Print final prompt and exit without calling Gemini

followup options:
  --prompt <text>         Follow-up question/instruction
  --prompt-file <path>    Read follow-up prompt from file
  --resume <id>           Session to continue (default: latest)
  --interactive           Continue in interactive mode
  --dry-run               Print final prompt and exit without calling Gemini

Notes:
  - This wrapper always uses model: gemini-3.1-pro-preview
  - This wrapper always uses read-only mode: --approval-mode plan
  - This wrapper never performs code edits
EOF
}

fail() {
  echo "Error: $*" >&2
  exit 1
}

require_gemini() {
  if ! command -v gemini >/dev/null 2>&1; then
    fail "Gemini CLI not found in PATH. Install/configure Gemini CLI before using this skill."
  fi
}

validate_mode() {
  local mode="$1"
  case "$mode" in
    decision|plan|debug|problem-solving|pre-implement|frontend) ;;
    *)
      fail "Unsupported mode '$mode'. Use: decision, plan, debug, problem-solving, pre-implement, frontend."
      ;;
  esac
}

read_file() {
  local file="$1"
  [[ -f "$file" ]] || fail "File not found: $file"
  cat "$file"
}

mode_template() {
  local mode="$1"
  case "$mode" in
    decision)
      cat <<'EOF'
- List 2-4 viable options.
- Compare tradeoffs across complexity, risk, maintainability, and delivery speed.
- Recommend one option with explicit decision criteria.
- State what evidence would change the decision.
EOF
      ;;
    plan)
      cat <<'EOF'
- Produce a concrete implementation plan with ordered steps.
- Include assumptions, dependencies, and sequencing constraints.
- Include risk checkpoints and fallback paths.
- Include validation checkpoints after each major step.
EOF
      ;;
    debug)
      cat <<'EOF'
- Prioritize hypotheses by likelihood and impact.
- Provide a shortest-path debug sequence to isolate root cause.
- Define expected observations for each check.
- Provide the likely fix strategy once root cause is confirmed.
EOF
      ;;
    problem-solving)
      cat <<'EOF'
- Decompose the problem into sub-problems.
- Identify constraints and unknowns.
- Propose candidate approaches and select the best one.
- Provide an execution path with explicit tradeoffs.
EOF
      ;;
    pre-implement)
      cat <<'EOF'
- Define architecture and implementation approach before coding.
- Specify interface boundaries, data flow, and error handling strategy.
- Include test strategy (unit/integration/e2e as relevant).
- Include rollout and rollback considerations when applicable.
EOF
      ;;
    frontend)
      cat <<'EOF'
- Produce an implementation package, not just a plan summary.
- Include component/page hierarchy with responsibilities.
- Include state model, data flow, and API contract expectations.
- Include accessibility requirements, responsive behavior, and loading/error states.
- Include styling/theming direction and motion/animation guidance.
- Include a complete file tree and FULL file contents (copy-paste ready) for the core implementation files.
- Do not output pseudocode or placeholder comments for core files.
- Include at least one executable component test file.
- Include a short runbook (dependencies, integration steps, and test command).
EOF
      ;;
  esac
}

implementation_package_requirements() {
  cat <<'EOF'
- Produce an implementation package, not just a plan summary.
- Include a complete file tree and FULL file contents (copy-paste ready) for the core implementation files.
- Do not output pseudocode or placeholder comments for core files.
- Include at least one executable component test file.
- Include a short runbook (dependencies, integration steps, and test command).
EOF
}

return_format_template() {
  local mode="$1"
  local force_package="$2"
  case "$mode" in
    frontend)
      cat <<'EOF'
Section A: Final architecture summary (concise)
Section B: File tree
Section C: Full file contents (all core implementation files, copy-paste ready)
Section D: Runbook (install deps, wire into app, run tests)
Section E: Verified evidence (codebase references and external sources used)
Section F: Open gaps and explicitly UNVERIFIED assumptions
EOF
      ;;
    *)
      if [[ "$force_package" -eq 1 ]]; then
        cat <<'EOF'
Section A: Final architecture summary (concise)
Section B: File tree
Section C: Full file contents (all core implementation files, copy-paste ready)
Section D: Runbook (install deps, wire into app, run tests)
Section E: Verified evidence (codebase references and external sources used)
Section F: Open gaps and explicitly UNVERIFIED assumptions
EOF
      else
        cat <<'EOF'
1) Problem framing
2) Assumptions and missing information
3) Recommended approach
4) Detailed execution plan
5) Risks and mitigations
6) Validation and test strategy
7) Verified evidence from codebase (paths + symbols/literals)
8) External verification used (URLs + accessed date), or "Not needed"
9) Open gaps and explicitly UNVERIFIED assumptions
10) Immediate next step for the coding agent
EOF
      fi
      ;;
  esac
}

quality_gates_template() {
  local mode="$1"
  local force_package="$2"
  if [[ "$mode" == "frontend" || "$force_package" -eq 1 ]]; then
    cat <<'EOF'
- Must include Section E with concrete evidence from the codebase:
  - cite file paths you inspected
  - cite key symbols/literals that informed decisions
- If external APIs/version-specific behavior is discussed, include official source URLs in this exact format:
  - URL (accessed YYYY-MM-DD)
- Must include Section F with explicit "UNVERIFIED" items for anything not confirmed.
- Do not include tool-control chatter (for example: "submitting plan", "exit plan mode", or tool denial messages).
EOF
  else
    cat <<'EOF'
- Must include item 7 with concrete codebase evidence:
  - cite file paths inspected
  - cite key symbols/literals used for conclusions
- If external APIs/version-specific behavior is discussed, include item 8 with official source URLs in this exact format:
  - URL (accessed YYYY-MM-DD)
  otherwise write "Not needed".
- Must include item 9 with explicit "UNVERIFIED" items for anything not confirmed.
- Do not include tool-control chatter (for example: "submitting plan", "exit plan mode", or tool denial messages).
EOF
  fi
}

build_context_block() {
  local context=""
  local file
  for file in "$@"; do
    [[ -f "$file" ]] || fail "Context file not found: $file"
    context+=$'\n'
    context+="----- BEGIN CONTEXT FILE: ${file} -----"
    context+=$'\n'
    context+="$(cat "$file")"
    context+=$'\n'
    context+="----- END CONTEXT FILE: ${file} -----"
    context+=$'\n'
  done
  printf '%s' "$context"
}

build_ask_prompt() {
  local mode="$1"
  local task="$2"
  local extra="$3"
  local prompt_file="$4"
  local implementation_package="$5"
  shift 5
  local context_files=("$@")

  local mode_block base_mode_block package_block=""
  base_mode_block="$(mode_template "$mode")"
  if [[ "$implementation_package" -eq 1 && "$mode" != "frontend" ]]; then
    package_block="$(implementation_package_requirements)"
  fi
  mode_block="$base_mode_block"
  if [[ -n "$package_block" ]]; then
    mode_block+=$'\n'
    mode_block+="$package_block"
  fi
  local return_format_block
  return_format_block="$(return_format_template "$mode" "$implementation_package")"
  local quality_gates_block
  quality_gates_block="$(quality_gates_template "$mode" "$implementation_package")"

  local extra_block=""
  if [[ -n "$extra" ]]; then
    extra_block=$'\nAdditional guidance:\n'
    extra_block+="$extra"
    extra_block+=$'\n'
  fi

  local prompt_file_block=""
  if [[ -n "$prompt_file" ]]; then
    prompt_file_block=$'\nAdditional instructions from prompt file:\n'
    prompt_file_block+="$(read_file "$prompt_file")"
    prompt_file_block+=$'\n'
  fi

  local context_block="(No context files provided)"
  if [[ "${#context_files[@]}" -gt 0 ]]; then
    context_block="$(build_context_block "${context_files[@]}")"
  fi

  cat <<EOF
You are a senior technical planning partner for another coding agent.

Hard constraints:
- You are in read-only planning mode.
- You may use read-only tools to explore the codebase and inspect files/logs.
- You may use web/documentation search tools to verify APIs, versions, and best practices.
- Before recommending decisions, plans, or fixes, inspect relevant codebase artifacts first.
- Before invoking any tool, confirm the tool exists in the environment and only use available read-only tools.
- Do not call unknown/unavailable tools (for example, do not attempt generic shell tool names that are not exposed).
- Do not assume repository structure or behavior without checking evidence.
- Do not modify files, do not run write/edit commands, and do not perform destructive actions.
- If evidence is insufficient, return UNVERIFIED findings and next evidence-gathering steps instead of definitive conclusions.
- Provide concrete, implementable planning output.

Task mode: ${mode}
Task statement:
${task}
${extra_block}${prompt_file_block}
Mode-specific deliverables:
${mode_block}

Project context:
${context_block}

Return format:
${return_format_block}

Quality gates (must pass):
${quality_gates_block}

If details are missing, state assumptions clearly and still provide a best-effort plan.
EOF
}

build_followup_prompt() {
  local prompt="$1"
  local prompt_file="$2"

  local result=""
  if [[ -n "$prompt" ]]; then
    result+="$prompt"
    result+=$'\n'
  fi

  if [[ -n "$prompt_file" ]]; then
    result+="$(read_file "$prompt_file")"
    result+=$'\n'
  fi

  [[ -n "$result" ]] || fail "Provide --prompt or --prompt-file for followup."
  printf '%s' "$result"
}

run_gemini_prompt() {
  local prompt="$1"
  local resume="$2"
  local interactive="$3"
  local dry_run="$4"

  if [[ "$dry_run" -eq 1 ]]; then
    echo "Dry run: Gemini was not called."
    echo "Model: $MODEL"
    echo "Approval mode: $APPROVAL_MODE"
    if [[ -n "$resume" ]]; then
      echo "Resume session: $resume"
    else
      echo "Resume session: (none)"
    fi
    if [[ "$interactive" -eq 1 ]]; then
      echo "Interactive: true"
      echo "Prompt flag: --prompt-interactive"
    else
      echo "Interactive: false"
      echo "Prompt flag: --prompt"
    fi
    echo
    echo "----- BEGIN PROMPT -----"
    printf '%s\n' "$prompt"
    echo "----- END PROMPT -----"
    return 0
  fi

  local -a cmd=(gemini --model "$MODEL" --approval-mode "$APPROVAL_MODE")
  if [[ -n "$resume" ]]; then
    cmd+=(--resume "$resume")
  fi

  if [[ "$interactive" -eq 1 ]]; then
    cmd+=(--prompt-interactive "$prompt")
  else
    cmd+=(--prompt "$prompt")
  fi

  "${cmd[@]}"
}

handle_ask() {
  local mode=""
  local task=""
  local extra=""
  local prompt_file=""
  local implementation_package=0
  local resume=""
  local interactive=0
  local dry_run=0
  local -a context_files=()

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --mode)
        [[ $# -gt 1 ]] || fail "Missing value for --mode"
        mode="$2"
        shift 2
        ;;
      --task)
        [[ $# -gt 1 ]] || fail "Missing value for --task"
        task="$2"
        shift 2
        ;;
      --context-file)
        [[ $# -gt 1 ]] || fail "Missing value for --context-file"
        context_files+=("$2")
        shift 2
        ;;
      --prompt-file)
        [[ $# -gt 1 ]] || fail "Missing value for --prompt-file"
        prompt_file="$2"
        shift 2
        ;;
      --extra)
        [[ $# -gt 1 ]] || fail "Missing value for --extra"
        extra="$2"
        shift 2
        ;;
      --implementation-package)
        implementation_package=1
        shift
        ;;
      --resume)
        [[ $# -gt 1 ]] || fail "Missing value for --resume"
        resume="$2"
        shift 2
        ;;
      --interactive)
        interactive=1
        shift
        ;;
      --dry-run)
        dry_run=1
        shift
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        fail "Unknown ask option: $1"
        ;;
    esac
  done

  [[ -n "$mode" ]] || fail "--mode is required for ask"
  [[ -n "$task" ]] || fail "--task is required for ask"
  validate_mode "$mode"

  if [[ -n "$prompt_file" ]]; then
    [[ -f "$prompt_file" ]] || fail "Prompt file not found: $prompt_file"
  fi

  local prompt
  prompt="$(build_ask_prompt "$mode" "$task" "$extra" "$prompt_file" "$implementation_package" "${context_files[@]}")"
  run_gemini_prompt "$prompt" "$resume" "$interactive" "$dry_run"
}

handle_followup() {
  local prompt=""
  local prompt_file=""
  local resume="latest"
  local interactive=0
  local dry_run=0

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --prompt)
        [[ $# -gt 1 ]] || fail "Missing value for --prompt"
        prompt="$2"
        shift 2
        ;;
      --prompt-file)
        [[ $# -gt 1 ]] || fail "Missing value for --prompt-file"
        prompt_file="$2"
        shift 2
        ;;
      --resume)
        [[ $# -gt 1 ]] || fail "Missing value for --resume"
        resume="$2"
        shift 2
        ;;
      --interactive)
        interactive=1
        shift
        ;;
      --dry-run)
        dry_run=1
        shift
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        fail "Unknown followup option: $1"
        ;;
    esac
  done

  local final_prompt
  final_prompt="$(build_followup_prompt "$prompt" "$prompt_file")"
  run_gemini_prompt "$final_prompt" "$resume" "$interactive" "$dry_run"
}

main() {
  require_gemini
  [[ $# -gt 0 ]] || {
    usage
    exit 1
  }

  local subcommand="$1"
  shift

  case "$subcommand" in
    ask)
      handle_ask "$@"
      ;;
    followup)
      handle_followup "$@"
      ;;
    sessions)
      gemini --list-sessions
      ;;
    delete-session)
      [[ $# -eq 1 ]] || fail "Usage: gemini-consult.sh delete-session <index>"
      gemini --delete-session "$1"
      ;;
    -h|--help|help)
      usage
      ;;
    *)
      fail "Unknown subcommand '$subcommand'. Use: ask, followup, sessions, delete-session."
      ;;
  esac
}

main "$@"
