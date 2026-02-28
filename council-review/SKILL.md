---
name: council-review
description: "Use this skill for multi-model AI code review. Trigger whenever the user asks to review code changes, audit a diff, check code quality, review a PR, review commits, or review uncommitted changes before pushing or merging. Also trigger when they say 'code review', 'review my changes', 'check this before I merge', or want multiple perspectives on code. Runs Codex, Gemini, and Claude reviews in parallel, then synthesizes a unified report. Do NOT use for reviewing documentation, markdown, or non-code files, or for trivial single-line changes."
---

# Council Review

Run Codex and Gemini code reviews in parallel, then Claude performs its own review to cross-validate findings — like a review board where Claude is the lead reviewer who deliberates with two external experts and delivers one opinion.

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

### Step 2: Run Reviews in Parallel

Launch **two Task agents simultaneously** in a single message. Both scripts are in `scripts/` relative to this skill's directory and enforce the correct model and read-only mode internally. Run `<script> --help` for full usage.

#### Codex — `scripts/codex-review.py`

```bash
python3 scripts/codex-review.py uncommitted
python3 scripts/codex-review.py branch --base main
python3 scripts/codex-review.py commit <SHA>
```

#### Gemini — `scripts/gemini-review.py`

Output is always structured YAML. Supports `pr` scope (Codex does not) and additional options `--context-file` and `--interactive`.

```bash
python3 scripts/gemini-review.py uncommitted
python3 scripts/gemini-review.py branch --base main
python3 scripts/gemini-review.py commit <SHA>
python3 scripts/gemini-review.py pr <PR_NUMBER>
```

Shared options: `--base <BRANCH>`, `--focus <TEXT>`, `--dry-run`.

### Step 3: Claude Code Review & Validation

After both external reviewers return, trigger the **`/review`** command to perform Claude Code's own independent review, then cross-validate all findings:

1. **Run `/review`** — Invoke the `/review` command on the same scope to get Claude Code's own review.
2. **Validate external findings** — For each finding from Codex and Gemini:
   - **Confirm** — Claude independently agrees the issue exists and is correctly described.
   - **Dispute** — Claude believes the finding is a false positive or incorrectly categorized. Note the reasoning.
   - **Enhance** — The issue exists but the explanation or suggested fix can be improved. Provide the improved version.
3. **Add Claude's own findings** — Include any issues from `/review` that neither Codex nor Gemini caught.

### Step 4: Synthesize into Unified Report

After your own review and validation are complete, **merge, deduplicate, and rewrite** all findings into one coherent report as if written by a single reviewer. Do not copy-paste or concatenate raw outputs.

Load `references/output-format.md` for the report template. Load `references/merge-rules.md` for how to reconcile findings across reviewers.

## Rules

- **Run Codex and Gemini in parallel** — sequential execution doubles the wait for no benefit; both agents are independent.
- **Use the same review scope for all reviewers** — comparing different scopes would make deduplication meaningless.
- **Always run `/review` (Step 3) before synthesizing** — Claude's own analysis is what turns three outputs into one trustworthy report, not just a merge.
- **Write one unified opinion** — the report should read as a single reviewer's assessment. Never structure findings by reviewer (no "Codex found..." sections). Source attribution belongs only in the collapsible raw outputs.
- **Sort findings by severity** — 🔴 → 🟠 → 🟡 → 🟢 → 🔵, with higher confidence first within the same severity.
- **Always use the wrapper scripts** for external reviewers — never call `codex` or `gemini` CLIs directly, because the scripts set the correct model and read-only mode.
- If one external CLI is missing, run the available one + Claude's review and synthesize normally.
- If only Codex or only Gemini is explicitly requested, run just that one + Claude's review.
