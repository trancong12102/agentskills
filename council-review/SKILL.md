---
name: council-review
description: "Use this skill for multi-model AI code review. Trigger whenever the user asks to review code changes, audit a diff, check code quality, review a PR, review commits, or review uncommitted changes before pushing or merging. Also trigger when they say 'code review', 'review my changes', 'check this before I merge', or want multiple perspectives on code. Runs Codex, Gemini, and Claude reviews in parallel, then synthesizes a unified report. Do NOT use for reviewing documentation, markdown, or non-code files, or for trivial single-line changes."
---

# Council Review

Run Codex, Gemini, and Claude's own `/review` all in parallel, then cross-validate and synthesize into one unified report — like a review board where three reviewers examine the code independently, and Claude as lead reviewer delivers the final opinion.

## Prerequisites

- **Codex CLI**: Install with `npm i -g @openai/codex`, authenticate with `codex login`
- **Gemini CLI**: Install and authenticate, ensure `gemini` command is available in PATH

If only one CLI is installed, fall back to the available reviewer with a warning — the review still has value with fewer perspectives, so don't fail entirely.

## When to Use

- Reviewing uncommitted changes before committing
- Auditing a branch diff before opening a PR
- Reviewing a specific commit for regressions
- Checking a remote PR (Gemini only supports this scope)

## When NOT to Use

- Reviewing documentation, markdown, or non-code files
- Trivial single-line changes where a full council review would be overkill

## Workflow

**DO NOT read script source code.** Run scripts directly and use `--help` for usage.

### Step 1: Determine Review Scope

If the scope is not already clear, use AskUserQuestion to ask:

- **Uncommitted changes** (default) — staged, unstaged, and untracked changes
- **Branch diff** — compare current branch against a base branch
- **Specific commit** — audit a single changeset
- **Remote PR** — review a GitHub PR by number or URL (Gemini only, skip Codex)

### Step 2: Run All Three Reviews in Parallel

All three reviewers read the same diff independently — none depends on another's output. Launch them all at once in a single message to eliminate sequential wait time.

Scripts are in `scripts/` relative to this skill's directory and enforce the correct model and read-only mode internally. Run `<script> --help` for full usage.

#### Codex — `scripts/codex-review.py` (background Bash task)

Launch as a background Bash task (`run_in_background: true`). **Codex CLI thinks deeply and may take up to 30 minutes** — do not treat a long wait as a failure. You will be notified automatically when it completes.

```bash
python3 scripts/codex-review.py uncommitted
python3 scripts/codex-review.py branch --base main
python3 scripts/codex-review.py commit <SHA>
```

#### Gemini — `scripts/gemini-review.py` (background Bash task)

Launch as a background Bash task (`run_in_background: true`). Output is always structured YAML. Supports `pr` scope (Codex does not) and additional options `--context-file` and `--interactive`.

```bash
python3 scripts/gemini-review.py uncommitted
python3 scripts/gemini-review.py branch --base main
python3 scripts/gemini-review.py commit <SHA>
python3 scripts/gemini-review.py pr <PR_NUMBER>
```

Shared options: `--base <BRANCH>`, `--focus <TEXT>`, `--dry-run`.

#### Claude — `/review` command

While Codex and Gemini run in the background, trigger the `/review` command immediately on the same scope. This runs Claude Code's own independent review concurrently with the external reviewers, so by the time Codex and Gemini finish, Claude's review is already done too.

### Step 3: Cross-Validate Findings

Once all three reviews have returned, cross-validate:

1. **Validate external findings** — For each finding from Codex and Gemini:
   - **Confirm** — Claude independently agrees the issue exists and is correctly described.
   - **Dispute** — Claude believes the finding is a false positive or incorrectly categorized. Note the reasoning.
   - **Enhance** — The issue exists but the explanation or suggested fix can be improved. Provide the improved version.
2. **Add Claude's own findings** — Include any issues from `/review` that neither Codex nor Gemini caught.

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

This applies to both `codex-review.py` and `gemini-review.py`.

## Rules

- **Run all three reviewers in parallel** — Codex, Gemini, and `/review` are independent reads of the same diff. Running them concurrently instead of sequentially saves the entire `/review` execution time.
- **Use the same review scope for all reviewers** — comparing different scopes would make deduplication meaningless.
- **Wait for all three reviews before synthesizing** — Claude's own analysis is what turns three outputs into one trustworthy report, not just a merge. All three must complete before cross-validation begins.
- **Write one unified opinion** — the report should read as a single reviewer's assessment. Never structure findings by reviewer (no "Codex found..." sections).
- **Sort findings by severity** — 🔴 → 🟠 → 🟡 → 🟢 → 🔵, with higher confidence first within the same severity.
- **Always use the wrapper scripts** for external reviewers — never call `codex` or `gemini` CLIs directly, because the scripts set the correct model and read-only mode.
- If one external CLI is missing, run the available one + Claude's review and synthesize normally.
- If only Codex or only Gemini is explicitly requested, run just that one + Claude's review.
