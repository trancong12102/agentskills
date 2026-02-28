#!/usr/bin/env bash
set -euo pipefail

MODEL="gpt-5.3-codex"

usage() {
  cat <<'EOF'
Usage:
  codex-review.sh uncommitted [options]
  codex-review.sh branch [--base <branch>] [options]
  codex-review.sh commit <SHA> [options]

Subcommands:
  uncommitted  Review staged, unstaged, and untracked changes
  branch       Review current branch diff against a base branch
  commit       Review changes introduced by a specific commit

Options:
  --base <branch>         Base branch for comparison (default: main)
  --focus <text>          Narrow the review to specific concerns
  --dry-run               Print the command without running Codex

Notes:
  - Model is fixed to gpt-5.3-codex
EOF
}

fail() {
  echo "Error: $*" >&2
  exit 1
}

require_codex() {
  if ! command -v codex >/dev/null 2>&1; then
    fail "Codex CLI not found in PATH. Install with: npm i -g @openai/codex && codex login"
  fi
}

main() {
  require_codex
  [[ $# -gt 0 ]] || { usage; exit 1; }

  local subcommand="$1"; shift

  case "$subcommand" in
    uncommitted) handle_uncommitted "$@" ;;
    branch) handle_branch "$@" ;;
    commit)
      [[ $# -ge 1 ]] || fail "Usage: codex-review.sh commit <SHA> [options]"
      handle_commit "$@"
      ;;
    -h|--help|help) usage ;;
    *) fail "Unknown subcommand '$subcommand'. Use: uncommitted, branch, commit." ;;
  esac
}

handle_uncommitted() {
  local focus=""
  local dry_run=0

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --focus) focus="$2"; shift 2 ;;
      --dry-run) dry_run=1; shift ;;
      -h|--help) usage; exit 0 ;;
      *) fail "Unknown option: $1" ;;
    esac
  done

  local -a cmd=(codex review --uncommitted -c model="$MODEL")
  [[ -n "$focus" ]] && cmd+=("$focus")
  run_codex cmd "$dry_run"
}

handle_branch() {
  local base="main"
  local focus=""
  local dry_run=0

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --base) base="$2"; shift 2 ;;
      --focus) focus="$2"; shift 2 ;;
      --dry-run) dry_run=1; shift ;;
      -h|--help) usage; exit 0 ;;
      *) fail "Unknown option: $1" ;;
    esac
  done

  local -a cmd=(codex review --base "$base" -c model="$MODEL")
  [[ -n "$focus" ]] && cmd+=("$focus")
  run_codex cmd "$dry_run"
}

handle_commit() {
  local sha="$1"; shift
  local focus=""
  local dry_run=0

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --focus) focus="$2"; shift 2 ;;
      --dry-run) dry_run=1; shift ;;
      -h|--help) usage; exit 0 ;;
      *) fail "Unknown option: $1" ;;
    esac
  done

  local -a cmd=(codex review --commit "$sha" -c model="$MODEL")
  [[ -n "$focus" ]] && cmd+=("$focus")
  run_codex cmd "$dry_run"
}

run_codex() {
  local -n _cmd=$1
  local dry_run="$2"

  if [[ "$dry_run" -eq 1 ]]; then
    echo "=== DRY RUN ==="
    echo "Command: ${_cmd[*]}"
    return 0
  fi

  "${_cmd[@]}"
}

main "$@"
