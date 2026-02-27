---
name: council-review
description: Multi-model AI code review council that runs Codex (OpenAI) and Gemini (Google) reviews in parallel, then synthesizes findings into a single unified report. Use when the user wants a thorough multi-model code review, asks for "council review", "full review", "review with both models", or wants maximum review coverage before merging critical changes.
---

# Council Review (Codex + Gemini)

## Overview

Run Codex and Gemini code reviews **in parallel** using the Task tool, then merge all findings into a single unified report — like a review board that deliberates and delivers one opinion. Individual reviewer outputs are preserved in a collapsible section but never shown as the primary structure.

## Workflow

### Step 1: Determine Review Scope

Ask the user what they want reviewed if not already clear:

| Scope | When to use | Codex | Gemini |
| --- | --- | --- | --- |
| Branch diff | Before opening or merging a PR | Yes | Yes |
| Uncommitted changes | During active development | Yes | Yes |
| Specific commit | Auditing a single changeset | Yes | Yes |
| Remote PR | Reviewing a GitHub Pull Request by number or URL | No | Yes |
| Custom | User provides specific review instructions | Yes | Yes |

> **Note:** Remote PR scope is only supported by Gemini. When user selects PR, run Gemini review only and skip Codex.

Default to reviewing the current branch diff against `main` unless the user specifies otherwise.

### Step 2: Run Both Reviews in Parallel

Launch **two Task agents simultaneously** in a single message:

- **Agent 1**: Activate the `codex-review` skill and run it with the determined scope. Capture the full review output.
- **Agent 2**: Activate the `gemini-review` skill with `--format structured` to get YAML output optimized for merging. Example: `scripts/gemini-review.sh branch --base main --format structured`

Tell each agent to capture and return the **full review output**. The structured YAML from Gemini makes it easier to extract and merge findings reliably.

### Step 3: Synthesize into Unified Report

Once both agents return, **merge, deduplicate, and rewrite** all findings into the format below. Do NOT copy-paste or concatenate the raw outputs — synthesize them into one coherent report as if written by a single reviewer.

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
- The synthesized report must be **shorter** than both raw outputs combined
- If one CLI is not installed, warn the user and fall back to the available reviewer — do not fail entirely
