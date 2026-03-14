---
name: oracle
description: "Deep analysis and expert reasoning via a separate model (Codex CLI). Use when the user asks for 'oracle', 'second opinion', architecture analysis, elusive bug debugging, impact assessment, security reasoning, refactoring strategy, or trade-off evaluation — problems that benefit from deep, independent reasoning. Do NOT use for simple factual questions, code generation, code review (use council-review), or tasks needing file modifications."
---

# Oracle

Delegate deep analysis to Codex CLI — launch it with a clear question, wait for it to finish, then present the results. Codex runs in a read-only sandbox with full codebase access, so it gathers its own context.

## Prerequisites

- **Codex CLI** (required): Install with `npm i -g @openai/codex`, authenticate with `codex login`

If Codex CLI is not installed, **stop and tell the user** to install it.

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

Do not read script source code. Run scripts directly and use `--help` for usage.

### Step 1: Formulate Question

Codex CLI has full read access to the codebase and can explore files, grep code, and web search on its own. Your job is to craft a clear, specific question — not to gather context for it.

1. Understand the user's question and what they need analyzed.
2. Formulate a clear, specific question that captures the user's intent — include relevant file paths, function names, or architectural areas to point Codex in the right direction.
3. Optionally use `--context-file` for truly external context that Codex cannot access on its own (e.g., user-provided files outside the repo, paste content).

### Step 2: Run Codex

Scripts are in `scripts/` relative to this skill's directory. Run `<script> --help` for full usage.

Launch `scripts/codex-oracle.py` as a background Bash task (`run_in_background: true`). Codex CLI may take up to 30 minutes.

```bash
python3 scripts/codex-oracle.py --question "..." --context-file path1 --context-file path2 [--focus text] [--dry-run]
```

After launching the background task, **end your response immediately** and wait. Do not poll, read output files, or check process status. You will be notified automatically when Codex completes.

### Step 3: Present Results

1. Use the `Read` tool on the `output-file` path from the completion notification to retrieve the Codex analysis.
2. Present the results to the user — use your own judgment on formatting and what to highlight.

## Rules

- **Do not start your own parallel analysis while Codex runs** — Codex is the analyst. Your role is to formulate the question, launch Codex, and present the results.
- **Wait for Codex to complete before presenting results** — the oracle's value depends on Codex's deep reasoning output.
- **Organize findings by theme** — group related insights together, not by severity alone. Structure adapts to question type (architecture -> components/trade-offs, bug -> root cause hypotheses, security -> threat model, etc.).
- **Never use `TaskOutput` for background tasks** — `TaskOutput` cannot find background Bash task IDs and will fail. Use the `Read` tool on the `output-file` path from the completion notification instead.
- **Do not poll or probe background tasks** — Do not read output files, check process status, or run any commands while waiting. End your response after launching. You will be notified automatically when Codex completes.
- **Always use the wrapper script** for Codex — do not call `codex` CLI directly, because the script sets the correct model and read-only mode.
