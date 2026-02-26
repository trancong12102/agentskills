---
name: codex-review
description: Run code reviews using the OpenAI Codex CLI. Use when the user asks to review code, review a pull request, review a branch, review uncommitted changes, review a commit, or wants AI-powered code review via Codex.
---

# Codex Code Review

## Overview

Use the OpenAI Codex CLI to perform AI-powered code reviews. Supports reviewing branches, uncommitted changes, specific commits, and custom-scoped reviews. Reviews are read-only and never modify the working tree.

## Prerequisites

- Codex CLI installed (`npm i -g @openai/codex`)
- Authenticated (`codex login`)

## Workflow

### Step 1: Determine Review Scope

Ask the user what they want reviewed if not already clear:

| Scope | When to use |
| --- | --- |
| Branch diff | Before opening or merging a PR |
| Uncommitted changes | During active development |
| Specific commit | Auditing a single changeset |
| Custom | User provides specific review instructions |

### Step 2: Run the Review

**Branch review** (compare current branch against base):

```bash
codex review --base main -c model="gpt-5.3-codex"
```

**Uncommitted changes:**

```bash
codex review --uncommitted -c model="gpt-5.3-codex"
```

**Specific commit:**

```bash
codex review --commit <SHA> -c model="gpt-5.3-codex"
```

**Custom review with focused instructions:**

```bash
codex review -c model="gpt-5.3-codex" "Review the code for accessibility regressions and WCAG compliance."
```

## Common Flags

| Flag | Description |
| --- | --- |
| `--base <BRANCH>` | Review changes against a base branch |
| `--uncommitted` | Review staged, unstaged, and untracked changes |
| `--commit <SHA>` | Review changes introduced by a specific commit |
| `--title <TITLE>` | Optional commit title for the review summary |
| `-c model="gpt-5.3-codex"` | Override the model (preferred: `gpt-5.3-codex`) |

## Rules

- Default to reviewing the current branch diff against `main` unless the user specifies otherwise
- Always pass `-c model="gpt-5.3-codex"` to use the preferred model
- Use `--base`, `--uncommitted`, or `--commit` flags to set the review scope instead of describing it in the prompt
- Present findings sorted by priority (highest first)
- Summarize the overall assessment before listing individual findings
- If Codex CLI is not installed, instruct the user to run `npm i -g @openai/codex` and `codex login`
