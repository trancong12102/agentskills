#!/usr/bin/env python3
"""
Run a code review using Gemini CLI.

Usage:
    python3 gemini-review.py branch [--base <branch>] [options]
    python3 gemini-review.py uncommitted [options]
    python3 gemini-review.py commit <SHA> [options]
    python3 gemini-review.py pr <PR_NUMBER> [options]

Subcommands:
    branch       Review current branch diff against a base branch
    uncommitted  Review staged, unstaged, and untracked changes
    commit       Review changes introduced by a specific commit
    pr           Checkout and review a GitHub Pull Request

Options:
    --base <branch>         Base branch for comparison (default: main)
    --focus <text>          Narrow the review to specific concerns
    --context-file <path>   Add extra context file (repeatable)
    --dry-run               Print the prompt without calling Gemini
    --interactive           Keep Gemini chat open after review

Notes:
    - Model is fixed to gemini-3.1-pro-preview
    - Always runs in read-only mode (--approval-mode plan)
"""

import json
import os
import shutil
import subprocess
import sys

MODEL = "gemini-3.1-pro-preview"
APPROVAL_MODE = "plan"


def fail(msg):
    print(f"Error: {msg}", file=sys.stderr)
    sys.exit(1)


def usage():
    print(__doc__.strip())


def require_gemini():
    if not shutil.which("gemini"):
        fail("Gemini CLI not found in PATH. Install and authenticate Gemini CLI before using this skill.")


def require_gh():
    if not shutil.which("gh"):
        fail("GitHub CLI (gh) not found in PATH. Install it to review PRs.")


def run_cmd(args, fail_msg=None):
    result = subprocess.run(args, capture_output=True, text=True)
    if result.returncode != 0:
        if fail_msg:
            fail(fail_msg)
        return ""
    return result.stdout


def is_binary_file(path):
    """Two-tier binary detection: try `file --brief`, fall back to null-byte heuristic."""
    if shutil.which("file"):
        result = subprocess.run(
            ["file", "--brief", path], capture_output=True, text=True
        )
        if result.returncode == 0:
            return "text" not in result.stdout.lower()
    # Fallback: check for null bytes in first 8KB
    try:
        with open(path, "rb") as f:
            chunk = f.read(8192)
            return b"\x00" in chunk
    except (OSError, IOError):
        return True


def get_branch_diff(base):
    diff = run_cmd(
        ["git", "diff", f"{base}...HEAD"],
        fail_msg=f"Failed to get diff against '{base}'. Is '{base}' a valid branch?",
    )
    if not diff:
        fail(f"No changes found between '{base}' and HEAD.")
    return diff


def get_uncommitted_diff():
    diff = run_cmd(["git", "diff", "HEAD"])
    staged = run_cmd(["git", "diff", "--staged"])

    untracked_result = subprocess.run(
        ["git", "ls-files", "--others", "--exclude-standard"],
        capture_output=True, text=True,
    )
    untracked_files = [
        f for f in untracked_result.stdout.splitlines() if f.strip()
    ]

    result = ""
    if staged:
        result += f"=== STAGED CHANGES ===\n{staged}\n\n"
    if diff:
        result += f"=== UNSTAGED CHANGES ===\n{diff}\n\n"
    if untracked_files:
        result += "=== UNTRACKED FILES (NEW) ===\n"
        for f in untracked_files:
            result += f"--- new file: {f} ---\n"
            if is_binary_file(f):
                result += "(binary file, skipped)\n"
            else:
                try:
                    with open(f, "r", errors="replace") as fh:
                        result += fh.read() + "\n"
                except (OSError, IOError):
                    result += "(could not read file)\n"
            result += "\n"

    if not result:
        fail("No uncommitted changes found.")
    return result


def get_commit_diff(sha):
    diff = run_cmd(
        ["git", "show", "--format=%H %s%n%b", sha],
        fail_msg=f"Failed to get commit '{sha}'.",
    )
    return diff


def get_pr_diff(pr_number):
    require_gh()

    result = subprocess.run(
        ["gh", "pr", "checkout", pr_number], capture_output=True, text=True
    )
    if result.returncode != 0:
        fail(f"Failed to checkout PR #{pr_number}.")

    # Get base branch from PR metadata using --json + json.loads
    base = "main"
    pr_json_result = subprocess.run(
        ["gh", "pr", "view", pr_number, "--json", "baseRefName"],
        capture_output=True, text=True,
    )
    if pr_json_result.returncode == 0:
        try:
            pr_data = json.loads(pr_json_result.stdout)
            base = pr_data.get("baseRefName", "main")
        except (json.JSONDecodeError, KeyError):
            pass

    diff = run_cmd(
        ["git", "diff", f"{base}...HEAD"],
        fail_msg="Failed to get PR diff.",
    )

    # Get PR title and body using --json + json.loads
    pr_body = ""
    pr_info_result = subprocess.run(
        ["gh", "pr", "view", pr_number, "--json", "number,title,body"],
        capture_output=True, text=True,
    )
    if pr_info_result.returncode == 0:
        try:
            info = json.loads(pr_info_result.stdout)
            number = info.get("number", "")
            title = info.get("title", "")
            body = info.get("body", "")
            pr_body = f"PR #{number}: {title}\n{body}"
        except (json.JSONDecodeError, KeyError):
            pass

    result_text = ""
    if pr_body:
        result_text += f"=== PR DESCRIPTION ===\n{pr_body}\n\n"
    result_text += diff
    return result_text


def build_context_block(context_files):
    context = ""
    for path in context_files:
        if not os.path.isfile(path):
            fail(f"Context file not found: {path}")
        context += f"\n----- BEGIN: {path} -----\n"
        with open(path, "r", errors="replace") as f:
            context += f.read()
        context += f"\n----- END: {path} -----\n"
    return context


def build_common_preamble(focus, context_block):
    focus_block = ""
    if focus:
        focus_block = f"""
REVIEW FOCUS:
Pay special attention to: {focus}
"""

    context_section = ""
    if context_block:
        context_section = f"""
ADDITIONAL CONTEXT:
{context_block}
"""

    return f"""\
You are a senior code reviewer performing a thorough, professional code review.

INSTRUCTIONS:
Analyze the code changes below. Evaluate each change against the following criteria:

1. **Correctness**: Does the code achieve its stated purpose without bugs or logical errors?
2. **Security**: Are there potential security vulnerabilities (injection, XSS, auth bypass, secrets exposure)?
3. **Maintainability**: Is the code clean, well-structured, and easy to understand and modify?
4. **Efficiency**: Are there obvious performance bottlenecks or resource inefficiencies?
5. **Edge Cases**: Does the code handle edge cases and errors appropriately?
6. **Testability**: Is the code adequately testable? Suggest missing test cases if relevant.
{focus_block}
CONSTRAINTS:
- You are in read-only mode. Do not modify any files.
- You may use read-only tools to explore the codebase for additional context.
- Cite specific file paths and line numbers when referencing issues.
- Be constructive and explain *why* a change is needed, not just *what* to change.
{context_section}"""


def build_structured_prompt(preamble, diff):
    return f"""\
{preamble}
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
{diff}"""


def build_review_prompt(diff, focus, context_block):
    preamble = build_common_preamble(focus, context_block)
    return build_structured_prompt(preamble, diff)


def run_gemini(prompt, interactive, dry_run):
    if dry_run:
        print("=== DRY RUN ===")
        print(f"Model: {MODEL}")
        print(f"Approval mode: {APPROVAL_MODE}")
        print()
        print("----- BEGIN PROMPT -----")
        print(prompt)
        print("----- END PROMPT -----")
        return

    cmd = ["gemini", "--model", MODEL, "--approval-mode", APPROVAL_MODE]
    if interactive:
        cmd.extend(["--prompt-interactive", prompt])
    else:
        cmd.extend(["--prompt", prompt])

    result = subprocess.run(cmd)
    sys.exit(result.returncode)


def parse_common_opts(args):
    focus = ""
    interactive = False
    dry_run = False
    context_files = []
    rest = []
    i = 0
    while i < len(args):
        if args[i] == "--focus":
            if i + 1 >= len(args):
                fail("--focus requires a value")
            focus = args[i + 1]
            i += 2
        elif args[i] == "--context-file":
            if i + 1 >= len(args):
                fail("--context-file requires a value")
            context_files.append(args[i + 1])
            i += 2
        elif args[i] == "--interactive":
            interactive = True
            i += 1
        elif args[i] == "--dry-run":
            dry_run = True
            i += 1
        elif args[i] in ("-h", "--help"):
            usage()
            sys.exit(0)
        else:
            rest.append(args[i])
            i += 1
    return focus, interactive, dry_run, context_files, rest


def handle_branch(args):
    base = "main"
    focus, interactive, dry_run, context_files, rest = parse_common_opts(args)
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

    diff = get_branch_diff(base)
    context_block = build_context_block(context_files) if context_files else ""
    prompt = build_review_prompt(diff, focus, context_block)
    run_gemini(prompt, interactive, dry_run)


def handle_uncommitted(args):
    focus, interactive, dry_run, context_files, rest = parse_common_opts(args)
    if rest:
        fail(f"Unknown option: {rest[0]}")

    diff = get_uncommitted_diff()
    context_block = build_context_block(context_files) if context_files else ""
    prompt = build_review_prompt(diff, focus, context_block)
    run_gemini(prompt, interactive, dry_run)


def handle_commit(args):
    if not args:
        fail("Usage: gemini-review.py commit <SHA> [options]")
    sha = args[0]
    focus, interactive, dry_run, context_files, rest = parse_common_opts(args[1:])
    if rest:
        fail(f"Unknown option: {rest[0]}")

    diff = get_commit_diff(sha)
    context_block = build_context_block(context_files) if context_files else ""
    prompt = build_review_prompt(diff, focus, context_block)
    run_gemini(prompt, interactive, dry_run)


def handle_pr(args):
    if not args:
        fail("Usage: gemini-review.py pr <PR_NUMBER> [options]")
    pr_number = args[0]
    focus, interactive, dry_run, context_files, rest = parse_common_opts(args[1:])
    if rest:
        fail(f"Unknown option: {rest[0]}")

    diff = get_pr_diff(pr_number)
    context_block = build_context_block(context_files) if context_files else ""
    prompt = build_review_prompt(diff, focus, context_block)
    run_gemini(prompt, interactive, dry_run)


def main():
    require_gemini()

    if len(sys.argv) < 2:
        usage()
        sys.exit(1)

    subcommand = sys.argv[1]
    rest = sys.argv[2:]

    if subcommand in ("-h", "--help", "help"):
        usage()
        sys.exit(0)
    elif subcommand == "branch":
        handle_branch(rest)
    elif subcommand == "uncommitted":
        handle_uncommitted(rest)
    elif subcommand == "commit":
        handle_commit(rest)
    elif subcommand == "pr":
        handle_pr(rest)
    else:
        fail(f"Unknown subcommand '{subcommand}'. Use: branch, uncommitted, commit, pr.")


if __name__ == "__main__":
    main()
