---
name: council-review
description: "Use this skill for multi-model AI code review. Trigger whenever the user asks to review code changes, audit a diff, check code quality, review a PR, review commits, or review uncommitted changes before pushing or merging. Also trigger when they say 'code review', 'review my changes', 'check this before I merge', or want multiple perspectives on code. Runs Codex and Claude reviews in parallel, then synthesizes a unified report. Do NOT use for reviewing documentation, markdown, or non-code files, or for trivial single-line changes."
---

# Council Review

Run Codex and Claude's own `/review` in parallel, then cross-validate and synthesize into one unified report — like a review board where two reviewers examine the code independently, and Claude as lead reviewer delivers the final opinion.

## Prerequisites

- **Codex CLI**: Install with `npm i -g @openai/codex`, authenticate with `codex login`

If only one CLI is installed, fall back to the available reviewer with a warning — the review still has value with fewer perspectives, so don't fail entirely.

## When to Use

- Reviewing uncommitted changes before committing
- Auditing a branch diff before opening a PR
- Reviewing a specific commit for regressions
## When NOT to Use

- Reviewing documentation, markdown, or non-code files
- Trivial single-line changes where a full council review would be overkill

## Workflow

Do not read script source code. Run scripts directly and use `--help` for usage.

### Step 1: Determine Review Scope

If the scope is not already clear, use AskUserQuestion to ask:

- **Uncommitted changes** (default) — staged, unstaged, and untracked changes
- **Branch diff** — compare current branch against a base branch
- **Specific commit** — audit a single changeset
### Step 2: Run Both Reviews in Parallel

Both reviewers read the same diff independently — neither depends on the other's output. Launch them both at once in a single message to eliminate sequential wait time.

Scripts are in `scripts/` relative to this skill's directory and enforce the correct model and read-only mode internally. Run `<script> --help` for full usage.

#### Codex — `scripts/codex-review.py` (background Bash task)

Launch as a background Bash task (`run_in_background: true`). Codex CLI may take up to 30 minutes. When it completes, use the `Read` tool on the `output-file` path from the notification to retrieve the review.

```bash
python3 scripts/codex-review.py uncommitted
python3 scripts/codex-review.py branch --base main
python3 scripts/codex-review.py commit <SHA>
```

#### Claude — `/review` skill (background Agent)

Launch a background Agent (`run_in_background: true`) to run `/review` on the same scope. Prompt the agent to invoke the `/review` skill (via the Skill tool) and return its complete findings. The agent's output arrives directly in its completion notification.

### Step 3: Cross-Validate Findings

Once both reviews have returned, cross-validate:

1. **Validate external findings** — For each finding from Codex:
   - **Confirm** — Claude independently agrees the issue exists and is correctly described.
   - **Dispute** — Claude believes the finding is a false positive or incorrectly categorized. Note the reasoning.
   - **Enhance** — The issue exists but the explanation or suggested fix can be improved. Provide the improved version.
2. **Add Claude's own findings** — Include any issues from `/review` that Codex didn't catch.

### Step 4: Synthesize into Unified Report

After your own review and validation are complete, **merge, deduplicate, and rewrite** all findings into one coherent report as if written by a single reviewer. Do not copy-paste or concatenate raw outputs.

Load `references/output-format.md` for the report template. Load `references/merge-rules.md` for how to reconcile findings across reviewers.

## Error Handling: Retry on Argument Errors

If a script exits with a non-zero code and stderr mentions argument conflicts (e.g. "cannot be used with", "unrecognized arguments", "invalid option"), **do not give up**. Follow this recovery sequence:

1. **Read the error message** from the failed Bash output.
2. **Run the script with `--help`** to get the correct usage.
3. **Re-run with corrected arguments.** Common fixes:
   - Drop the `--focus` flag — some CLI versions don't accept it with certain scopes.
   - Move focus text from a positional argument to a named flag, or vice versa.
   - Remove flags that conflict with the chosen subcommand.
4. If the second attempt also fails with a different argument error, repeat steps 1-3 **once more** (max 2 retries).
5. If it still fails after retries, log the error and continue with the remaining reviewers — a partial council review is better than none.

This applies to `codex-review.py`.

## Rules

- **Run both reviewers in parallel** — Codex and `/review` are independent reads of the same diff. Running them concurrently instead of sequentially saves the entire `/review` execution time.
- **Use the same review scope for both reviewers** — comparing different scopes would make deduplication meaningless.
- **Wait for both reviews to complete before cross-validation** — the council's value depends on comparing complete outputs.
- **Write one unified opinion** — the report should read as a single reviewer's assessment. Never structure findings by reviewer (no "Codex found..." sections).
- **Sort findings by priority** — P0 → P1 → P2 → P3 → P4.
- **Exclude low-confidence findings** — If Claude disputes an external finding or evidence is purely circumstantial, omit it from the report. The council's value is cross-validation; findings that fail it are noise.
- **Always use the wrapper script** for Codex — do not call `codex` CLI directly, because the script sets the correct model and read-only mode.
- **Suppress intermediate outputs** — Do not display raw Codex or `/review` outputs to the user. Running `/review` in a subagent keeps its output out of the main conversation naturally. The only review output the user should see is the final unified report.
- **Never use `TaskOutput` for background tasks** — `TaskOutput` cannot find background Bash task IDs and will fail. Use the `Read` tool on the `output-file` path from the completion notification instead. For background Agents, read the result directly from the completion notification.
