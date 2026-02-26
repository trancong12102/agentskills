---
name: comprehensive-review
description: Comprehensive AI code review that runs both Codex (OpenAI) and Gemini (Google) reviews in parallel, then synthesizes findings into a single consolidated report. Use when the user wants a thorough multi-model code review, asks for "comprehensive review", "full review", "review with both models", or wants maximum review coverage before merging critical changes.
---

# Comprehensive Code Review (Codex + Gemini)

## Overview

Run Codex and Gemini code reviews **in parallel** using the Task tool, then synthesize both results into a single consolidated report. This provides broader coverage by leveraging two independent AI models with different strengths.

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

Launch **two Task agents simultaneously** in a single message. Both agents must run in the **same message** to ensure true parallelism.

- **Agent 1**: Activate the `codex-review` skill and run it with the determined scope.
- **Agent 2**: Activate the `gemini-review` skill and run it with the same scope.

Pass the user's review scope (branch, uncommitted, commit SHA, PR number, custom focus) to both agents so they review the exact same changes.

Tell each agent to capture and return the **full review output**.

### Step 3: Synthesize Results

Once both agents return, produce a **single consolidated report** using this structure:

---

## Comprehensive Review

**Quality: X/5** · **Confidence: X/5** · **Verdict: <verdict>**

Where:
- Quality (1-5): Combined quality assessment across both reviewers
- Confidence (1-5): How much the reviewers agree (5 = full consensus, 1 = major contradictions)
- Verdict: Approved / Approved with suggestions / Request Changes

### Summary
3-5 sentence overview combining both reviewers' assessments. Note where they agree and disagree.

### Changes Walkthrough

| File | Changes |
|------|---------|
| `path/to/file.ts` | Brief description of what changed |

### Findings

All issues merged, deduplicated, and sorted by severity. Each finding must follow this format:

> **[Category]** `file/path.ts:LINE` — Short title `[Codex]` `[Gemini]` `[Both]`
>
> Explanation of the issue and why it matters.
>
> **Suggested fix:**
> ```lang
> code suggestion here
> ```

Where:
- Category: `Bug`, `Security`, `Performance`, `Maintainability`, `Edge Case`, `Testing`, `Style`
- Source tag: `[Codex]`, `[Gemini]`, or `[Both]` — indicates which reviewer(s) flagged it
- Findings tagged `[Both]` should appear first within their severity group (highest confidence)
- If both flagged the same issue with different suggestions, present both and note the difference

If none: "No issues found by either reviewer."

### Highlights
1-3 positive patterns that one or both reviewers called out, attributed by source.

### Reviewer Agreement

| Metric | Details |
|--------|---------|
| **Consensus** | Issues both flagged independently (list count or summary) |
| **Codex only** | Findings unique to Codex |
| **Gemini only** | Findings unique to Gemini |
| **Conflicts** | Contradictory recommendations, if any (explain both perspectives) |

### Verdict
Restate the verdict with a 1-2 sentence justification referencing the combined findings.

<details>
<summary>Raw reviewer outputs</summary>

Include both full outputs here for reference, clearly labeled:
- **Codex output**: full text
- **Gemini output**: full text

</details>

---

## Rules

- Always run both reviews **in parallel** using two Task agents in a single message — never sequentially
- Use the same review scope and options for both reviewers to ensure a fair comparison
- Deduplicate findings that both reviewers flagged — mark them as `[Both]` with highest confidence and list first within their severity group
- When reviewers contradict each other, present both perspectives and let the user decide
- Attribute every finding to its source (`[Codex]`, `[Gemini]`, or `[Both]`)
- If one CLI is not installed, warn the user and fall back to running only the available reviewer — do not fail entirely
- Sort findings by severity: Bug/Security first, then Performance/Maintainability/Edge Case/Testing, then Style
- The consolidated report should be **shorter** than both individual reports combined — synthesize, don't concatenate
- Always include raw reviewer outputs in a collapsible `<details>` section at the end for transparency
