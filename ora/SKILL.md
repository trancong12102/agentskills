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

**Do not Edit/Write in main after plan mode exits.** All implementation runs through Hephaestus — dispatch a new wave for any change touching 2+ files or >10 lines. Exceptions: typo/comment fixes, removing an unused import, or a 1-line syntax error pointed out explicitly by Aletheia/Hephaestus output. Any code change that requires reading surrounding context first goes to Hephaestus.

## Verification — do not skip

6. **Aletheia per Hephaestus task.** Do not squash-merge a worktree without ora:Aletheia verification first.
   - GAPS_FOUND → resume Hephaestus via SendMessage (do not respawn).
   - Still failing after 2 retries → ask user.
7. Squash-merge each verified worktree.

**After Aletheia GREEN, do not re-Read worktree files in main.** Aletheia already verified. If you need to inspect something specific, dispatch ora:Ariadne with a targeted question — do not verify-by-Read. **Do not Read the same file via both worktree path and merged main path** — after squash-merge the content is identical.

## Finalize

8. Run /council-review or /simplify against the squash-merged wave commits (from merge base to HEAD).
9. **Dispatch a new Hephaestus fix-wave for any review issue touching 2+ files or >10 lines. Severity does not matter — the threshold is scope. Default to 1 fix-wave; split into parallel waves only when issues cluster into ≥3 independent modules with no file overlap.**
10. `git reset --soft <merge-base>` + `git restore --staged` — uncommit and unstage all wave commits so changes appear as working directory modifications.
11. Stop and let human review all changes and commit.

## Rules

- All agents use resume via SendMessage — do not respawn when the same session can continue.
- Ariadne for local codebase. Clio for external sources. Do not mix.
- Hephaestus for implementation only. Do not use for exploration or research.
- Aletheia for goal-backward verification only. Do not use for code quality review.
