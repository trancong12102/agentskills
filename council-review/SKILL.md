---
name: council-review
description: "Multi-model AI code review — runs Codex, Claude, and Simplify reviews in parallel, then synthesizes a unified report. Use when the user asks to review code changes, audit a diff, check code quality, review a PR, review commits, or review uncommitted changes. Also covers 'code review', 'review my changes', 'check this before I merge', or wanting multiple perspectives on code. Do NOT use for documentation/markdown review or trivial single-line changes."
---

# Council Review

Run Codex, Claude's own `/review`, and `/simplify` in parallel, then cross-validate and synthesize into one unified report — like a review board where three reviewers examine the code from different angles, and Claude as lead reviewer delivers the final opinion.

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

### Step 2: Run All Three Reviews in Parallel

All three reviewers read the same diff independently — none depends on another's output. Launch all three at once in a single message to eliminate sequential wait time.

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

#### Simplify — `/simplify` skill (background Agent)

Launch a background Agent (`run_in_background: true`) to run `/simplify` on the same scope. Prompt the agent to invoke the `/simplify` skill (via the Skill tool), then **return only its analysis and findings as text** — do not apply any code fixes. The agent's output arrives directly in its completion notification.

After launching all three background tasks, **end your turn immediately**. Do not output anything else, do not proceed to Step 3, and do not check on task progress. You will be notified automatically when each task completes.

### Step 3: Cross-Validate Findings

Once you have received completion notifications for **all three** tasks, cross-validate:

1. **Validate external findings** — For each finding from Codex and `/simplify`:
   - **Confirm** — Claude independently agrees the issue exists and is correctly described.
   - **Dispute** — Claude believes the finding is a false positive or incorrectly categorized. Note the reasoning.
   - **Enhance** — The issue exists but the explanation or suggested fix can be improved. Provide the improved version.
2. **Add Claude's own findings** — Include any issues from `/review` that the other reviewers didn't catch.
3. **Note cross-reviewer agreement** — Track which findings were flagged by multiple reviewers (higher confidence).

### Step 4: Synthesize into Unified Report

After your own review and validation are complete, **merge, deduplicate, and rewrite** all findings into one coherent report as if written by a single reviewer. Do not copy-paste or concatenate raw outputs.

Load `references/output-format.md` for the report template. Load `references/merge-rules.md` for how to reconcile findings across reviewers.

## Rules

- **Run all three reviewers in parallel** — Codex, `/review`, and `/simplify` are independent reads of the same diff. Launch all three in a single message.
- **Use the same review scope for all reviewers** — comparing different scopes would make deduplication meaningless.
- **Wait for all three reviews to complete before cross-validation** — the council's value depends on comparing complete outputs.
- **Run `/simplify` agent as report-only** — the agent must return findings as text, not apply edits to the workspace. Instruct the agent explicitly to skip code fixes.
- **Write one unified opinion** — the report should read as a single reviewer's assessment. Never structure findings by reviewer (no "Codex found..." or "Simplify found..." sections).
- **Sort findings by priority** — P0 → P1 → P2 → P3 → P4.
- **Exclude low-confidence findings** — If Claude disputes an external finding or evidence is purely circumstantial, omit it from the report. The council's value is cross-validation; findings that fail it are noise.
- **Always use the wrapper script** for Codex — do not call `codex` CLI directly, because the script sets the correct model and read-only mode.
- **Suppress intermediate outputs** — Do not display raw reviewer outputs to the user. Running each skill in a subagent keeps its output out of the main conversation naturally. The only review output the user should see is the final unified report.
- **Never use `TaskOutput` for background tasks** — `TaskOutput` cannot find background Bash task IDs and will fail. Use the `Read` tool on the `output-file` path from the completion notification instead. For background Agents, read the result directly from the completion notification.
- **If a reviewer fails at runtime** — fall back to the remaining reviewers if at least two succeed. If fewer than two reviewers succeed, stop the review and report the error — a single-reviewer result lacks cross-validation.
