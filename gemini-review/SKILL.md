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

### Step 3: Present Results

The script outputs a structured review with:
- **Header line**: Quality score (1-5), Effort estimate (1-5), and Verdict — scannable at a glance
- **Summary**: 2-3 sentence high-level overview
- **Changes Walkthrough**: Table summarizing what changed in each file
- **Findings**: All issues grouped under one heading, each tagged with a category (`Bug`, `Security`, `Performance`, `Maintainability`, `Edge Case`, `Testing`, `Style`), file:line reference, explanation, and suggested fix — sorted by severity
- **Highlights**: Positive patterns worth calling out (good abstractions, solid error handling, etc.)
- **Verdict**: Final recommendation (Approved / Approved with suggestions / Request Changes)

### Step 4: Cleanup (Remote PRs only)

After reviewing a remote PR, ask the user if they want to switch back to the original branch.

## Common Options

| Option | Description |
| --- | --- |
| `--base <BRANCH>` | Base branch for comparison (default: `main`) |
| `--focus <TEXT>` | Narrow the review to specific concerns |
| `--context-file <PATH>` | Add extra context file (repeatable) |
| `--dry-run` | Print the prompt without calling Gemini |
| `--interactive` | Keep Gemini chat open for follow-up questions |

## Rules

- Default to reviewing the current branch diff against `main` unless the user specifies otherwise
- Always use the wrapper script — it enforces `gemini-3.1-pro-preview` model and read-only mode
- Sort findings by severity: Bug/Security first, then Performance/Maintainability/Edge Case/Testing, then Style
- If Gemini CLI is not installed, instruct the user to install and authenticate it
