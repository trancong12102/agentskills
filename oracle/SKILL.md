---
name: oracle
description: "Use this skill when the user needs deep analysis, reasoning, or expert guidance on complex problems. Trigger on: 'oracle', 'second opinion', architecture analysis, elusive bug debugging, impact assessment, security reasoning, refactoring strategy, or trade-off evaluation — problems that benefit from a separate model's deep reasoning. Delegates to Codex CLI. Do NOT use for simple factual questions, code generation, code review (use council-review), or tasks needing file modifications."
---

# Oracle

Delegate deep analysis to Codex CLI — launch it with full context, wait for it to finish, then present the results.

## Prerequisites

- **Codex CLI** (required): Install with `npm i -g @openai/codex`, authenticate with `codex login`

If Codex CLI is not installed, **stop and tell the user** to install it. Do not fall back to Claude-only analysis.

## When to Use

- Architecture analysis and design decisions
- Debugging elusive or complex bugs
- Impact assessment of proposed changes
- Security reasoning and threat modeling
- Refactoring strategy evaluation
- Trade-off analysis between approaches
- Complex "why" or "how" questions about a codebase

## When NOT to Use

- Simple factual questions (just answer directly)
- Code generation tasks (just write the code)
- Code review (use `council-review` instead)
- Tasks that require file modifications (oracle is read-only analysis)

## Workflow

**DO NOT read script source code.** Run scripts directly and use `--help` for usage.

### Step 1: Gather Context

Before launching Codex, gather relevant context because Codex CLI only sees what's explicitly passed to it.

1. Understand the user's question and identify what parts of the codebase are relevant.
2. Use Read, Grep, and Glob tools to read relevant files and code snippets.
3. **Research official sources when relevant** — When the question involves libraries, frameworks, or evolving best practices:
   - Use the `context7` skill to fetch up-to-date official documentation.
   - Use WebSearch to find official blog posts, best practices guides, RFCs, or authoritative references.
   - Include the findings as additional context files so Codex benefits from accurate, current information.
4. Write the gathered context to temporary files that can be passed to the script via `--context-file`.
5. Formulate a clear, specific question that captures the user's intent.

### Step 2: Run Codex

Scripts are in `scripts/` relative to this skill's directory. Run `<script> --help` for full usage.

Launch `scripts/codex-oracle.py` as a background Bash task (`run_in_background: true`). **Codex CLI thinks deeply and may take up to 30 minutes** — do not treat a long wait as a failure. You will be notified automatically when it completes.

```bash
python3 scripts/codex-oracle.py --question "..." --context-file path1 --context-file path2 [--focus text] [--dry-run]
```

Wait for the Codex background task notification before moving to Step 3.

### Step 3: Present Results

1. Read the Codex output from the completed background task.
2. Verify that cited file paths actually exist in the codebase.
3. Present the results to the user — use your own judgment on formatting and what to highlight.

## Rules

- **Do not start your own parallel analysis while Codex runs** — Codex is the analyst. Your role is to gather context, launch Codex, and present the results.
- **Wait for Codex to complete before presenting results** — the oracle's value depends on Codex's deep reasoning output.
- **Organize findings by theme** — group related insights together, not by severity alone. Structure adapts to question type (architecture -> components/trade-offs, bug -> root cause hypotheses, security -> threat model, etc.).
- **Research before reasoning** — Check official documentation (via `context7`) and search for authoritative references (via WebSearch) when the question involves libraries, frameworks, or evolving best practices.
- **Always use the wrapper script** for Codex — do not call `codex` CLI directly, because the script sets the correct model and read-only mode.
