#!/usr/bin/env python3
"""
Run a deep analysis using Codex CLI.

Usage:
    python3 codex-oracle.py --question "..." [options]

Options:
    --question <text>       The question or analysis request (required)
    --context-file <path>   Add context file content to the prompt (repeatable)
    --focus <text>          Narrow the analysis to specific concerns
    --dry-run               Print the command without running Codex

Notes:
    - Model is fixed to gpt-5.3-codex
    - Always runs in read-only sandbox mode (codex exec --sandbox read-only)
"""

import os
import shutil
import subprocess
import sys

MODEL = "gpt-5.3-codex"


def fail(msg):
    print(f"Error: {msg}", file=sys.stderr)
    sys.exit(1)


def usage():
    print(__doc__.strip())


def require_codex():
    if not shutil.which("codex"):
        fail("Codex CLI not found in PATH. Install with: npm i -g @openai/codex && codex login")


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


def build_prompt(question, focus, context_block):
    focus_block = ""
    if focus:
        focus_block = f"""
ANALYSIS FOCUS:
Pay special attention to: {focus}
"""

    context_section = ""
    if context_block:
        context_section = f"""
CONTEXT:
{context_block}
"""

    return f"""\
You are a senior software architect performing deep analysis and reasoning.

INSTRUCTIONS:
Analyze the following question thoroughly. Consider multiple angles, weigh trade-offs,
and provide actionable insights grounded in the codebase context provided.

QUESTION:
{question}
{focus_block}
CONSTRAINTS:
- You are in read-only mode. Do not modify any files.
- You may read files in the codebase to gather additional context.
- IMPORTANT: Before making recommendations, use the `context7` skill to fetch official documentation for the relevant libraries, frameworks, and tools. Also use web search to find best practices, official blog posts, and authoritative references. Ground your analysis in these official sources and cite them in your findings.
- Cite specific file paths and line numbers when referencing code.
- Be concrete and actionable — avoid vague or generic advice.
- If you are uncertain about something, say so explicitly with your confidence level.
{context_section}
OUTPUT FORMAT:
Structure your analysis as follows. Use these exact section headers:

SUMMARY:
2-3 sentence high-level answer to the question.

KEY FINDINGS:
For each finding:
- Finding: Short title
  Detail: Detailed explanation with evidence from the codebase.
  Confidence: high | medium | low
  Category: architecture | bug | security | performance | maintainability | design | testing | other

RECOMMENDATIONS:
For each recommendation:
- Action: What to do
  Rationale: Why this is recommended and what impact it will have.
  Priority: critical | high | medium | low

RISKS:
For each risk:
- Description: What could go wrong
  Likelihood: high | medium | low
  Mitigation: How to mitigate this risk"""


def run_codex(prompt, dry_run):
    cmd = ["codex", "exec", "--sandbox", "read-only", "-c", f"model={MODEL}", "-"]

    if dry_run:
        print("=== DRY RUN ===")
        print(f"Command: {' '.join(cmd)}")
        print()
        print("----- BEGIN PROMPT (stdin) -----")
        print(prompt)
        print("----- END PROMPT (stdin) -----")
        return

    result = subprocess.run(cmd, input=prompt, text=True)
    sys.exit(result.returncode)


def parse_args(args):
    question = ""
    focus = ""
    dry_run = False
    context_files = []
    i = 0
    while i < len(args):
        if args[i] == "--question":
            if i + 1 >= len(args):
                fail("--question requires a value")
            question = args[i + 1]
            i += 2
        elif args[i] == "--focus":
            if i + 1 >= len(args):
                fail("--focus requires a value")
            focus = args[i + 1]
            i += 2
        elif args[i] == "--context-file":
            if i + 1 >= len(args):
                fail("--context-file requires a value")
            context_files.append(args[i + 1])
            i += 2
        elif args[i] == "--dry-run":
            dry_run = True
            i += 1
        elif args[i] in ("-h", "--help"):
            usage()
            sys.exit(0)
        else:
            fail(f"Unknown option: {args[i]}")
    return question, focus, dry_run, context_files


def main():
    require_codex()

    if len(sys.argv) < 2:
        usage()
        sys.exit(1)

    question, focus, dry_run, context_files = parse_args(sys.argv[1:])

    if not question:
        fail("--question is required")

    context_block = build_context_block(context_files) if context_files else ""
    prompt = build_prompt(question, focus, context_block)
    run_codex(prompt, dry_run)


if __name__ == "__main__":
    main()
