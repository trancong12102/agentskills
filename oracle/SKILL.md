---
name: oracle
description: "Deep analysis and expert reasoning. Use when the user asks for 'oracle', 'second opinion', architecture analysis, elusive bug debugging, impact assessment, security reasoning, refactoring strategy, or trade-off evaluation — problems that benefit from deep, independent reasoning. Do not use for simple factual questions, code generation, code review (use council-review), or tasks needing file modifications."
---

# Oracle

Delegate deep analysis to Codex CLI — launch it with a clear question, wait for it to finish, then present the results. Codex runs in a read-only sandbox with full codebase access, so it gathers its own context.

## Prerequisites

- **Codex CLI** (required): Install with `npm i -g @openai/codex`, authenticate with `codex login`

If Codex CLI is not installed, **stop and tell the user** to install it.

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
# New session
python3 scripts/codex-oracle.py --question "..." [--context-file path] [--focus text] [--dry-run]

# Resume a previous session for follow-up
python3 scripts/codex-oracle.py --session-id <id> --question "follow-up question..." [--context-file path] [--focus text]
```

After launching the background task, **end your response immediately** and wait. Do not poll, read output files, or check process status. You will be notified automatically when Codex completes.

### Step 3: Review Response

1. Read the Codex output from the background task completion notification.
2. Capture the `oracle-session-id` from the last line — store it internally for follow-ups.
3. Review the response yourself. Decide whether it fully answers the user's question or needs clarification/deeper analysis.

### Step 4: Follow-up Loop (if needed)

If the Codex response is incomplete, ambiguous, or you need it to drill deeper — send follow-ups before presenting anything to the user. Repeat as many times as needed.

1. Use the stored `oracle-session-id` with `--session-id` to resume the session. Codex retains the full conversation history.
2. Only send the new follow-up question — do not repeat prior questions or the system prompt.
3. Launch as a background task, wait for completion, and review the new response.
4. Loop back to decide: sufficient, or another follow-up needed?

The session accumulates context with each round, making subsequent answers more informed. Start a new session (Step 1) only when the topic changes entirely.

### Step 5: Present Results

Once you have a complete, clear answer from the oracle (after one or more rounds):

1. Synthesize all Codex responses into a single coherent answer for the user.
2. Use your own judgment on formatting and what to highlight — you do not need to echo every detail from every round.

## Rules

- **Codex is the analyst — your role is to formulate, launch, and present.** Do not start your own parallel analysis while Codex runs.
- **Organize findings by theme, not severity.** Group related insights together. Structure adapts to question type (architecture → components/trade-offs, bug → root cause hypotheses, security → threat model, etc.).
- **Read background-task output via the `Read` tool on the `output-file` path** from the completion notification. `TaskOutput` cannot find background Bash task IDs and will fail.
- **Always use the wrapper script** for Codex. The script sets the correct model and read-only mode; calling `codex` CLI directly bypasses these.
