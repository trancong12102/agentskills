#!/usr/bin/env python3
"""
Run a deep analysis using Gemini CLI.

Usage:
    python3 gemini-oracle.py --question "..." [options]

Options:
    --question <text>       The question or analysis request (required)
    --context-file <path>   Add context file content to the prompt (repeatable)
    --focus <text>          Narrow the analysis to specific concerns
    --dry-run               Print the prompt without calling Gemini
    --interactive           Keep Gemini chat open after analysis

Notes:
    - Model is fixed to gemini-3.1-pro-preview
    - Always runs in read-only mode (--approval-mode plan)
"""

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
- You may use read-only tools to explore the codebase for additional context.
- Cite specific file paths and line numbers when referencing code.
- Be concrete and actionable — avoid vague or generic advice.
- If you are uncertain about something, say so explicitly with your confidence level.
{context_section}
OUTPUT FORMAT:
Return your analysis as a YAML document. This output will be consumed by another LLM
for synthesis, so strict adherence to the schema is critical.

Return ONLY the YAML block below — no prose, no markdown fences, no explanation outside the YAML.

summary: |
  2-3 sentence high-level answer to the question.
key_findings:
  - finding: Short title of the finding
    detail: |
      Detailed explanation with evidence from the codebase.
    confidence: high | medium | low
    category: architecture | bug | security | performance | maintainability | design | testing | other
recommendations:
  - action: What to do
    rationale: |
      Why this is recommended and what impact it will have.
    priority: critical | high | medium | low
risks:
  - description: What could go wrong
    likelihood: high | medium | low
    mitigation: How to mitigate this risk

Field definitions:
- confidence: high = strong evidence supports this, medium = reasonable inference, low = speculative
- category: architecture, bug, security, performance, maintainability, design, testing, other
- priority: critical = do this now, high = do this soon, medium = do this eventually, low = nice to have
- key_findings: empty list [] if no specific findings
- recommendations: empty list [] if no recommendations
- risks: empty list [] if no risks identified"""


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


def parse_args(args):
    question = ""
    focus = ""
    interactive = False
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
            fail(f"Unknown option: {args[i]}")
    return question, focus, interactive, dry_run, context_files


def main():
    require_gemini()

    if len(sys.argv) < 2:
        usage()
        sys.exit(1)

    question, focus, interactive, dry_run, context_files = parse_args(sys.argv[1:])

    if not question:
        fail("--question is required")

    context_block = build_context_block(context_files) if context_files else ""
    prompt = build_prompt(question, focus, context_block)
    run_gemini(prompt, interactive, dry_run)


if __name__ == "__main__":
    main()
