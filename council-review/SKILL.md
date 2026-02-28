---
name: council-review
description: Multi-model AI code review council. Use when reviewing code changes, auditing diffs, or assessing code quality.
---

# Council Review

## Overview

Run Codex and Gemini code reviews **in parallel** using the Task tool, then **Claude Code performs its own review** to validate and cross-check external findings before merging everything into a single unified report — like a review board where Claude is the lead reviewer who deliberates with two external experts and delivers one opinion. Individual reviewer outputs are preserved in a collapsible section but never shown as the primary structure.

## Prerequisites

- **Codex CLI**: Install with `npm i -g @openai/codex`, authenticate with `codex login`
- **Gemini CLI**: Install and authenticate, ensure `gemini` command is available in PATH

If only one CLI is installed, fall back to the available reviewer with a warning — do not fail entirely.

## Workflow

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

1. **Run `/review`** — Invoke the `/review` command on the same scope to get Claude Code's own review of the changes.
2. **Validate external findings** — For each finding from Codex and Gemini:
   - **Confirm** — Claude independently agrees the issue exists and is correctly described.
   - **Dispute** — Claude believes the finding is a false positive or incorrectly categorized. Note the reasoning.
   - **Enhance** — The issue exists but the explanation or suggested fix can be improved. Provide the improved version.
3. **Add Claude's own findings** — Include any issues from `/review` that neither Codex nor Gemini caught.

This step ensures the final report is grounded in Claude's own analysis, not just a merge of external outputs.

### Step 4: Synthesize into Unified Report

After your own review and validation are complete, **merge, deduplicate, and rewrite** all findings (from Codex, Gemini, and your own review) into the format below. Do not copy-paste or concatenate the raw outputs — synthesize them into one coherent report as if written by a single reviewer.

---

## Output Format

### Header

```
## Council Review

**Verdict: <VERDICT>** · Reviewed by Codex + Gemini + Claude

<1-2 sentence justification>
```

Verdict values:
- **Approved** — No issues or only informational notes
- **Approved with suggestions** — No critical/high issues, but improvements recommended
- **Request changes** — Critical or high-severity issues that should be fixed before merging

### Changes Walkthrough

| File | Changes |
|------|---------|
| `path/to/file.ts` | Brief description of what changed |

### Findings

All issues from all reviewers merged into a **single flat list**, deduplicated, sorted by severity (critical first). Each finding follows this format:

```
#### <EMOJI> <Short title>

**<CATEGORY>** · `file/path.ts:LINE` · Confidence: <HIGH|MEDIUM|LOW>

Explanation of the issue and why it matters.

**Suggested fix:**
\```lang
code here
\```
```

**Severity emoji mapping:**

| Emoji | Severity | Criteria |
|-------|----------|----------|
| 🔴 | Critical | Exploitable vulnerability, data loss, or crash in production |
| 🟠 | High | Likely bug or incident under realistic conditions |
| 🟡 | Medium | Incorrect behavior under edge cases or degraded performance |
| 🟢 | Low | Code quality issue that could escalate over time |
| 🔵 | Info | Observation or suggestion, no action required |

**Categories:** `Bug`, `Security`, `Performance`, `Maintainability`, `Edge Case`, `Testing`, `Style`

**Confidence is derived from reviewer agreement (3 reviewers: Codex, Gemini, Claude). Merge rules (below) take precedence over these defaults when they specify a confidence level:**
- **High** — 2+ reviewers flagged the same issue independently, or Claude confirmed an external finding
- **Medium** — One external reviewer flagged it and Claude did not dispute it, or Claude found it alone with clear evidence
- **Low** — Evidence is circumstantial or only one external reviewer flagged it with weak justification

If no issues found: "No issues found."

### Highlights

1-3 positive patterns worth calling out (good abstractions, solid error handling, thorough tests, etc.).

### Raw Outputs

Always include at the end:

```
<details>
<summary>Individual reviewer outputs</summary>

**Codex:**
<full codex output>

**Gemini:**
<full gemini output>

**Claude (/review):**
<Claude's /review output and validation notes>

</details>
```

---

## Merge Rules

When synthesizing findings from all three reviewers (Codex, Gemini, Claude):

1. **Same issue, same fix, Claude confirmed** → Merge into one finding, confidence: High
2. **Same issue, different fix** → Merge into one finding, confidence: High, present the best fix (prefer Claude's improved version if available)
3. **External finding confirmed by Claude** → Include with Claude's enhanced explanation if applicable, confidence: High
4. **External finding disputed by Claude** → Include the finding, confidence: Low, note Claude's reasoning for the dispute
5. **Contradictory assessments between external reviewers, Claude breaks the tie** → Include with Claude's assessment as the deciding factor, confidence: Medium
6. **Unique finding from one external reviewer, not disputed by Claude** → Include as-is, confidence: Medium
7. **Unique finding from Claude only** → Include as Claude's own finding, confidence: Medium

## Rules

- Always run Codex and Gemini reviews **in parallel** — never sequentially
- Use the same review scope for all reviewers
- **Always run `/review` (Step 3)** before synthesizing — never skip this step
- The report must read as **one unified opinion**, not three reports stitched together
- Never structure findings by reviewer (no "Codex found..." / "Gemini found..." sections)
- Source attribution appears only in the collapsible raw outputs section
- Sort findings strictly by severity: 🔴 → 🟠 → 🟡 → 🟢 → 🔵
- Within the same severity, High confidence findings come first
- If one external CLI is not installed, fall back to the available reviewer with a warning — Claude's own review always runs regardless
- If only Codex or only Gemini is explicitly requested, run just that one + Claude's review, then synthesize
- Always use the wrapper scripts for external reviewers — never call `codex` or `gemini` CLIs directly
