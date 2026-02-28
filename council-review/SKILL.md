---
name: council-review
description: Multi-model AI code review that runs Codex (OpenAI) and Gemini (Google) in parallel, then synthesizes findings into a unified report. Use when reviewing code changes, auditing diffs, or assessing code quality.
---

# Council Review

## Overview

Run Codex and Gemini code reviews **in parallel** using the Task tool, then merge all findings into a single unified report — like a review board that deliberates and delivers one opinion. Individual reviewer outputs are preserved in a collapsible section but never shown as the primary structure.

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

#### Codex — `scripts/codex-review.sh`

```bash
scripts/codex-review.sh uncommitted
scripts/codex-review.sh branch --base main
scripts/codex-review.sh commit <SHA>
```

#### Gemini — `scripts/gemini-review.sh`

Output is always structured YAML. Supports `pr` scope (Codex does not) and additional options `--context-file` and `--interactive`.

```bash
scripts/gemini-review.sh uncommitted
scripts/gemini-review.sh branch --base main
scripts/gemini-review.sh commit <SHA>
scripts/gemini-review.sh pr <PR_NUMBER>
```

Shared options: `--base <BRANCH>`, `--focus <TEXT>`, `--dry-run`.

### Step 3: Synthesize into Unified Report

After both agents return, **merge, deduplicate, and rewrite** all findings into the format below. Do not copy-paste or concatenate the raw outputs — synthesize them into one coherent report as if written by a single reviewer.

---

## Output Format

### Header

```
## Council Review

**Verdict: <VERDICT>** · Reviewed by Codex + Gemini

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

All issues from both reviewers merged into a **single flat list**, deduplicated, sorted by severity (critical first). Each finding follows this format:

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

**Confidence is derived from reviewer agreement:**
- **High** — Both reviewers flagged the same issue independently
- **Medium** — One reviewer flagged it with clear evidence
- **Low** — One reviewer flagged it but evidence is circumstantial

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

</details>
```

---

## Merge Rules

When synthesizing findings from both reviewers:

1. **Same issue, same fix** → Merge into one finding, confidence: High
2. **Same issue, different fix** → Merge into one finding, confidence: High, present both fixes and note the difference
3. **Contradictory assessments** (one says it's fine, the other flags it) → Include the finding, confidence: Low, briefly note the disagreement in the explanation
4. **Unique finding from one reviewer** → Include as-is, confidence: Medium

## Rules

- Always run both reviews **in parallel** — never sequentially
- Use the same review scope for both reviewers
- The report must read as **one unified opinion**, not two reports stitched together
- Never structure findings by reviewer (no "Codex found..." / "Gemini found..." sections)
- Source attribution appears only in the collapsible raw outputs section
- Sort findings strictly by severity: 🔴 → 🟠 → 🟡 → 🟢 → 🔵
- Within the same severity, High confidence findings come first
- If one CLI is not installed, fall back to the available reviewer with a warning
- If only Codex or only Gemini is explicitly requested, run just that one and skip the synthesis step
- Always use the wrapper scripts — never call `codex` or `gemini` CLIs directly
