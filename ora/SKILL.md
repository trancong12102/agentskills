---
name: ora
description: "Ora workflow for planning and executing multi-step tasks with agents. Use when entering plan mode, starting a non-trivial implementation, or orchestrating Ariadne/Clio/Hephaestus/Aletheia agents. Do NOT use for trivial single-file edits."
---

# Ora Workflow

## Planning (in plan mode)

1. **Landscape scan**: ora:Ariadne (always) + ora:Clio (if task involves externals) — understand what exists, not how to change it.
2. **Classify ambiguity**: vague request, multiple systems, unclear acceptance criteria → /brainstorm first. Clear and well-scoped → skip to 3.
3. **Deep targeted exploration**: Ariadne/Clio on the clarified scope.
4. **Write plan** informed by the analysis.

## Execution (after plan mode)

5. Spawn ora:Hephaestus in worktrees (parallel for independent tasks). Research in this phase is rare — only when execution reveals something the plan could not have anticipated.

## Verification — do not skip

6. **Aletheia per Hephaestus task.** Do not squash-merge a worktree without ora:Aletheia verification first.
   - GAPS_FOUND → resume Hephaestus via SendMessage (do not respawn).
   - Still failing after 2 retries → ask user.
7. Squash-merge each verified worktree.

## Finalize — do not commit

8. `git reset --soft` + `git restore --staged` — uncommit and unstage all changes so they appear as working directory modifications.
9. Run /council-review or /simplify on the unstaged changes. Apply fixes but do not commit.
10. Stop and let human review all changes and commit.

## Rules

- All agents use resume via SendMessage — do not respawn when the same session can continue.
- Ariadne for local codebase. Clio for external sources. Do not mix.
- Hephaestus for implementation only. Do not use for exploration or research.
- Aletheia for goal-backward verification only. Do not use for code quality review.
