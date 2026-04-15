#!/usr/bin/env python3
"""
Run a code review using Codex CLI.

Usage:
    python3 codex-review.py uncommitted [options]
    python3 codex-review.py branch [--base <branch>] [options]
    python3 codex-review.py commit <SHA> [options]

Subcommands:
    uncommitted  Review staged, unstaged, and untracked changes
    branch       Review current branch diff against a base branch
    commit       Review changes introduced by a specific commit

Options:
    --base <branch>         Base branch for comparison (default: main)
    --focus <text>          Narrow the review to specific concerns
                            (ignored — Codex CLI does not support [PROMPT]
                            together with scope flags; the script accepts it
                            for interface parity; accepted but ignored)
    --dry-run               Print the command without running Codex

Notes:
    - Model is fixed to gpt-5.4 (codex review does not support profiles)
    - Codex CLI's [PROMPT] arg is mutually exclusive with --uncommitted,
      --base, and --commit. When --focus is provided, the script prints a
      warning and drops it instead of failing.
"""

import json
import os
import shutil
import subprocess
import sys
import tempfile


def fail(msg):
    print(f"Error: {msg}", file=sys.stderr)
    sys.exit(1)


def usage():
    print(__doc__.strip())


def require_codex():
    if not shutil.which("codex"):
        fail("Codex CLI not found in PATH. Install with: npm i -g @openai/codex && codex login")


def extract_response(raw_path):
    """Extract agent_message text from Codex CLI JSONL output."""
    messages = []
    with open(raw_path, "r", errors="replace") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                event = json.loads(line)
            except json.JSONDecodeError:
                continue
            item = event.get("item", {})
            if item.get("type") == "agent_message":
                text = item.get("text", "").strip()
                if text:
                    messages.append(text)
    return "\n\n".join(messages)


def run_codex(cmd, dry_run):
    if dry_run:
        print("=== DRY RUN ===")
        print(f"Command: {' '.join(cmd)}")
        return

    raw = tempfile.NamedTemporaryFile(
        prefix="codex-review-raw-", suffix=".jsonl", delete=False, mode="w"
    )
    try:
        result = subprocess.run(cmd, stdout=raw)
    finally:
        raw.close()

    response = extract_response(raw.name)
    os.unlink(raw.name)

    print(response)
    sys.exit(result.returncode)


def warn_focus_ignored(focus):
    if focus:
        print(
            f"Warning: --focus '{focus}' ignored — Codex CLI does not support "
            f"[PROMPT] together with scope flags (--uncommitted/--base/--commit).",
            file=sys.stderr,
        )


def parse_common_opts(args):
    focus = ""
    dry_run = False
    rest = []
    i = 0
    while i < len(args):
        if args[i] == "--focus":
            if i + 1 >= len(args):
                fail("--focus requires a value")
            focus = args[i + 1]
            i += 2
        elif args[i] == "--dry-run":
            dry_run = True
            i += 1
        elif args[i] in ("-h", "--help"):
            usage()
            sys.exit(0)
        else:
            rest.append(args[i])
            i += 1
    return focus, dry_run, rest


def handle_uncommitted(args):
    focus, dry_run, rest = parse_common_opts(args)
    if rest:
        fail(f"Unknown option: {rest[0]}")
    warn_focus_ignored(focus)
    cmd = ["codex", "exec", "review", "--uncommitted", "--json", "-c", "model=gpt-5.4", "-c", 'model_reasoning_effort="xhigh"']
    run_codex(cmd, dry_run)


def handle_branch(args):
    base = "main"
    focus, dry_run, rest = parse_common_opts(args)
    # Extract --base from rest
    filtered = []
    i = 0
    while i < len(rest):
        if rest[i] == "--base":
            if i + 1 >= len(rest):
                fail("--base requires a value")
            base = rest[i + 1]
            i += 2
        else:
            filtered.append(rest[i])
            i += 1
    if filtered:
        fail(f"Unknown option: {filtered[0]}")
    warn_focus_ignored(focus)
    cmd = ["codex", "exec", "review", "--base", base, "--json", "-c", "model=gpt-5.4", "-c", 'model_reasoning_effort="xhigh"']
    run_codex(cmd, dry_run)


def handle_commit(args):
    if not args:
        fail("Usage: codex-review.py commit <SHA> [options]")
    sha = args[0]
    focus, dry_run, rest = parse_common_opts(args[1:])
    if rest:
        fail(f"Unknown option: {rest[0]}")
    warn_focus_ignored(focus)
    cmd = ["codex", "exec", "review", "--commit", sha, "--json", "-c", "model=gpt-5.4", "-c", 'model_reasoning_effort="xhigh"']
    run_codex(cmd, dry_run)


def main():
    require_codex()

    if len(sys.argv) < 2:
        usage()
        sys.exit(1)

    subcommand = sys.argv[1]
    rest = sys.argv[2:]

    if subcommand in ("-h", "--help", "help"):
        usage()
        sys.exit(0)
    elif subcommand == "uncommitted":
        handle_uncommitted(rest)
    elif subcommand == "branch":
        handle_branch(rest)
    elif subcommand == "commit":
        handle_commit(rest)
    else:
        fail(f"Unknown subcommand '{subcommand}'. Use: uncommitted, branch, commit.")


if __name__ == "__main__":
    main()
