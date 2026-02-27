---
name: gemini-review
description: AI-powered code review using the Gemini CLI with gemini-3.1-pro-preview. Use when reviewing branch diffs before merging a PR, auditing uncommitted changes during development, inspecting a specific commit, performing custom-scoped reviews, or whenever the user asks Gemini to review code.
---

# Gemini Code Review

## Overview

Use the Gemini CLI to perform AI-powered code reviews with `gemini-3.1-pro-preview` — Google's most capable reasoning model. Supports reviewing branches, uncommitted changes, specific commits, remote PRs, and custom-scoped reviews. Reviews are read-only and never modify the working tree.

## Prerequisites

- Gemini CLI installed and authenticated
- `gemini` command available in PATH

## Workflow

### Step 1: Determine Review Scope

Ask the user what they want reviewed if not already clear:

| Scope | When to use |
| --- | --- |
| Branch diff | Before opening or merging a PR |
| Uncommitted changes | During active development |
| Specific commit | Auditing a single changeset |
| Remote PR | Reviewing a GitHub Pull Request by number or URL |
| Custom | User provides specific review instructions |

### Step 2: Run the Review

**Branch review** (compare current branch against base):

```bash
scripts/gemini-review.sh branch --base main
```

**Uncommitted changes:**

```bash
scripts/gemini-review.sh uncommitted
```

**Specific commit:**

```bash
scripts/gemini-review.sh commit <SHA>
```

**Remote PR** (checks out and reviews a GitHub PR):

```bash
scripts/gemini-review.sh pr <PR_NUMBER>
```

**Custom review with focused instructions:**

```bash
scripts/gemini-review.sh branch --base main --focus "Review for accessibility regressions and WCAG compliance"
```

**Structured output for LLM consumption** (used by council-review):

```bash
scripts/gemini-review.sh branch --base main --format structured
```

### Step 3: Present Results

The script outputs a review in one of two formats depending on `--format`:

#### Markdown format (default)

Human-readable output with:
- **Verdict**: Approved / Approved with suggestions / Request Changes
- **Summary**: 2-3 sentence high-level overview
- **Changes Walkthrough**: Table summarizing what changed in each file
- **Findings**: All issues with explicit severity emoji (🔴🟠🟡🟢🔵), category, file:line reference, explanation, and suggested fix — sorted by severity
- **Highlights**: Positive patterns worth calling out
- **Verdict**: Final recommendation restated with justification

#### Structured format (`--format structured`)

YAML output optimized for LLM parsing and synthesis (e.g., by the `council-review` skill):

```yaml
verdict: approved | approved_with_suggestions | request_changes
summary: |
  2-3 sentence overview.
changes:
  - file: path/to/file.ts
    description: Brief description of changes
findings:
  - severity: critical | high | medium | low | info
    category: bug | security | performance | maintainability | edge_case | testing | style
    file: path/to/file.ts
    line: 42
    title: Short title
    description: |
      Explanation of the issue.
    suggestion: |
      code fix here
highlights:
  - Short description of a positive pattern
```

### Step 4: Cleanup (Remote PRs only)

After reviewing a remote PR, ask the user if they want to switch back to the original branch.

## Severity Levels

| Emoji | Level | Criteria |
|-------|-------|----------|
| 🔴 | Critical | Exploitable vulnerability, data loss, or crash in production |
| 🟠 | High | Likely bug or incident under realistic conditions |
| 🟡 | Medium | Incorrect behavior under edge cases or degraded performance |
| 🟢 | Low | Code quality issue that could escalate over time |
| 🔵 | Info | Observation or suggestion, no action required |

## Common Options

| Option | Description |
| --- | --- |
| `--base <BRANCH>` | Base branch for comparison (default: `main`) |
| `--focus <TEXT>` | Narrow the review to specific concerns |
| `--format <FORMAT>` | Output format: `markdown` (default) or `structured` (YAML for LLM consumption) |
| `--context-file <PATH>` | Add extra context file (repeatable) |
| `--dry-run` | Print the prompt without calling Gemini |
| `--interactive` | Keep Gemini chat open for follow-up questions |

## Rules

- Default to reviewing the current branch diff against `main` unless the user specifies otherwise
- Always use the wrapper script — it enforces `gemini-3.1-pro-preview` model and read-only mode
- Sort findings by severity: Critical/High first, then Medium, then Low/Info
- Every finding must include an explicit severity level — never rely on implicit ordering alone
- If Gemini CLI is not installed, instruct the user to install and authenticate it
- When called from `council-review`, always use `--format structured` for reliable LLM parsing
