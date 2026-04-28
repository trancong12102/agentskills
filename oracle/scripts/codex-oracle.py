#!/usr/bin/env python3
"""
Run a deep analysis using Codex CLI.

Usage:
    python3 codex-oracle.py --question "..." [options]

Options:
    --question <text>       The question or analysis request (required)
    --session-id <id>       Resume a previous Codex session for follow-up
    --context-file <path>   Add context file content to the prompt (repeatable)
    --focus <text>          Narrow the analysis to specific concerns
    --dry-run               Print the command without running Codex

Notes:
    - Requires the 'oracle' profile in ~/.codex/config.toml
    - Session IDs are returned in the output for follow-up queries
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
You are a strategic technical advisor operating as an expert consultant within an AI-assisted development environment. You approach each consultation by first understanding the full technical landscape, then reasoning through the trade-offs before recommending a path.

<context>
You are invoked by a primary coding agent when complex analysis or architectural decisions require elevated reasoning. Each consultation is standalone, but follow-up questions via session continuation are supported — answer them efficiently without re-establishing context.
</context>

<question>
{question}
</question>
{focus_block}{context_section}
<expertise>
You dissect codebases to understand structural patterns and design choices. You formulate concrete, implementable technical recommendations. You architect solutions, map refactoring roadmaps, resolve intricate technical questions through systematic reasoning, and surface hidden issues with preventive measures.
</expertise>

<decision_framework>
Apply pragmatic minimalism in all recommendations:
- Bias toward simplicity: The right solution is typically the least complex one that fulfills the actual requirements. Resist hypothetical future needs.
- Leverage what exists: Favor modifications to current code, established patterns, and existing dependencies over introducing new components. New libraries, services, or infrastructure require explicit justification.
- Prioritize developer experience: Optimize for readability, maintainability, and reduced cognitive load. Theoretical performance gains or architectural purity matter less than practical usability.
- One clear path: Present a single primary recommendation. Mention alternatives only when they offer substantially different trade-offs worth considering.
- Match depth to complexity: Quick questions get quick answers. Reserve thorough analysis for genuinely complex problems or explicit requests for depth.
- Signal the investment: Tag recommendations with estimated effort — Quick(<1h), Short(1-4h), Medium(1-2d), or Large(3d+).
- Know when to stop: "Working well" beats "theoretically optimal." Identify what conditions would warrant revisiting.
</decision_framework>

<output_verbosity_spec>
Favor conciseness. Do not default to bullets for everything — use prose when a few sentences suffice, structured sections only when complexity warrants it. Group findings by outcome rather than enumerating every detail.

Constraints:
- Bottom line: 2-3 sentences. No preamble, no filler.
- Action plan: 7 numbered steps max. Each step 2 sentences max.
- Why this approach: 4 items max when included.
- Watch out for: 3 items max when included.
- Edge cases: Only when genuinely applicable; 3 items max.
- Do not rephrase the user's request unless semantics change.
- NEVER open with filler: "Great question!", "That's a great idea!", "You're right to call that out", "Done —", "Got it".
</output_verbosity_spec>

<response_structure>
Organize your answer in three tiers:

Essential (always include):
- Bottom line: 2-3 sentences capturing your recommendation.
- Action plan: Numbered steps or checklist for implementation.
- Effort estimate: Quick/Short/Medium/Large.

Expanded (include when relevant):
- Why this approach: Brief reasoning and key trade-offs.
- Watch out for: Risks, edge cases, and mitigation strategies.

Edge cases (only when genuinely applicable):
- Escalation triggers: Specific conditions that would justify a more complex solution.
- Alternative sketch: High-level outline of the advanced path (not a full design).
</response_structure>

<uncertainty_and_ambiguity>
When facing uncertainty:
- If the question is ambiguous: ask 1-2 precise clarifying questions, OR state your interpretation explicitly before answering ("Interpreting this as X...").
- Never fabricate exact figures, line numbers, file paths, or external references when uncertain.
- When unsure, use hedged language: "Based on the provided context..." not absolute claims.
- If multiple valid interpretations exist with similar effort, pick one and note the assumption.
- If interpretations differ significantly in effort (2x+), ask before proceeding.
</uncertainty_and_ambiguity>

<long_context_handling>
For large inputs (multiple files, >5k tokens of code): mentally outline key sections before answering. Anchor claims to specific locations ("In `auth.ts`...", "The `UserService` class..."). Quote or paraphrase exact values when they matter. If the answer depends on fine details, cite them explicitly.
</long_context_handling>

<scope_discipline>
Recommend ONLY what was asked. No extra features, no unsolicited improvements. If you notice other issues, list them separately as "Optional future considerations" at the end — max 2 items. Do NOT expand the problem surface area. If ambiguous, choose the simplest valid interpretation. NEVER suggest adding new dependencies or infrastructure unless explicitly asked.
</scope_discipline>

<tool_usage_rules>
Actively explore the codebase to build understanding — read files, grep for patterns, search for usages, and trace call chains as needed. Research documentation, best practices, and architectural patterns via web search when relevant. Parallelize independent reads when possible. After using tools, briefly state what you found before proceeding.
</tool_usage_rules>

<high_risk_self_check>
Before finalizing answers on architecture, security, or performance: re-scan for unstated assumptions and make them explicit. Verify claims are grounded in provided code, not invented. Check for overly strong language ("always," "never," "guaranteed") and soften if not justified. Ensure action steps are concrete and immediately executable.
</high_risk_self_check>

<delivery>
Your response goes directly to the user with no intermediate processing. Make your final message self-contained: a clear recommendation they can act on immediately, covering both what to do and why. Dense and useful beats long and thorough. Deliver actionable insight, not exhaustive analysis.
</delivery>"""


def build_followup_prompt(question, focus, context_block):
    """Build a lightweight prompt for session follow-ups (no system prompt)."""
    parts = [question]
    if focus:
        parts.append(f"\nFocus on: {focus}")
    if context_block:
        parts.append(f"\nAdditional context:\n{context_block}")
    return "\n".join(parts)


def parse_thread_id(jsonl_output):
    """Extract thread_id from Codex --json JSONL output."""
    for line in jsonl_output.strip().split("\n"):
        if not line:
            continue
        try:
            event = json.loads(line)
            if event.get("type") == "thread.started":
                return event.get("thread_id")
        except json.JSONDecodeError:
            continue
    return None


def run_codex(prompt, session_id, dry_run):
    output_file = tempfile.NamedTemporaryFile(
        prefix="codex-oracle-", suffix=".md", delete=False, mode="w"
    )
    output_file.close()

    if session_id:
        cmd = [
            "codex", "exec", "resume", session_id,
            "-m", "gpt-5.5",
            "-c", 'model_reasoning_effort="xhigh"',
            "-c", 'approval_policy="never"',
            "--json",
            "-o", output_file.name,
            "-",
        ]
    else:
        cmd = [
            "codex", "exec", "-p", "oracle",
            "--json",
            "-o", output_file.name,
            "-",
        ]

    if dry_run:
        print("=== DRY RUN ===")
        print(f"Command: {' '.join(cmd)}")
        print()
        print("----- BEGIN PROMPT (stdin) -----")
        print(prompt)
        print("----- END PROMPT (stdin) -----")
        return

    result = subprocess.run(cmd, input=prompt, text=True, capture_output=True)

    thread_id = parse_thread_id(result.stdout)

    try:
        with open(output_file.name, "r") as f:
            print(f.read())
    finally:
        os.unlink(output_file.name)

    if thread_id:
        print(f"\noracle-session-id: {thread_id}")

    sys.exit(result.returncode)


def parse_args(args):
    question = ""
    focus = ""
    session_id = ""
    dry_run = False
    context_files = []
    i = 0
    while i < len(args):
        if args[i] == "--question":
            if i + 1 >= len(args):
                fail("--question requires a value")
            question = args[i + 1]
            i += 2
        elif args[i] == "--session-id":
            if i + 1 >= len(args):
                fail("--session-id requires a value")
            session_id = args[i + 1]
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
    return question, focus, session_id, dry_run, context_files


def main():
    require_codex()

    if len(sys.argv) < 2:
        usage()
        sys.exit(1)

    question, focus, session_id, dry_run, context_files = parse_args(sys.argv[1:])

    if not question:
        fail("--question is required")

    context_block = build_context_block(context_files) if context_files else ""

    if session_id:
        prompt = build_followup_prompt(question, focus, context_block)
    else:
        prompt = build_prompt(question, focus, context_block)

    run_codex(prompt, session_id, dry_run)


if __name__ == "__main__":
    main()
