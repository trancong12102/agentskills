---
name: gemini
description: Gemini CLI consultation workflow for coding agents. Use when technical tasks need Gemini consultation for decisions, planning, debugging, problem-solving, or pre-implementation guidance.
---

# Gemini

## Overview

Run Gemini CLI as a required decision and planning partner before coding actions.
This skill enforces `--approval-mode plan` (read-only) and structured prompts, and supports continuing the same Gemini conversation with `--resume`.

## Mandatory Workflow

1. Gather context first from local files, logs, and constraints.
2. Call Gemini before major decisions, planning, debugging, and hard problem solving:
   - `scripts/gemini-consult.sh ask --mode <mode> --task "<task>" ...`
3. If the answer is incomplete, continue the same conversation:
   - `scripts/gemini-consult.sh followup --resume latest --prompt "<follow-up question>"`
4. Implement only after synthesizing Gemini's recommendations with local repo evidence.

## Evidence-First Quality Rule

- Gemini must inspect relevant codebase artifacts before giving decisions or plans.
- Output must include explicit evidence:
  - codebase paths and key symbols/literals used for conclusions
  - external source URLs in format `URL (accessed YYYY-MM-DD)` when external/version-sensitive claims are made
- Unknowns must be explicitly marked as `UNVERIFIED`; no unstated assumptions.
- If evidence is insufficient, return `UNVERIFIED` findings and next evidence-gathering steps instead of definitive conclusions.
- Output must not include tool-control chatter (for example: "submitting plan", "exit plan mode").

## Code Reference Rule

- `frontend` tasks: consultation is mandatory and must return full implementation package output.
- Non-frontend tasks: before editing code, request Gemini code references when needed.
- For non-frontend modes, force full implementation package output with:
  - `scripts/gemini-consult.sh ask --mode <mode> --implementation-package --task "<task>"`

## Mode Selection

Use the right mode when running `ask`:

- `decision`: Compare options, tradeoffs, and make a recommendation.
- `plan`: Build a step-by-step execution plan with risk controls.
- `debug`: Produce a root-cause-first debugging strategy.
- `problem-solving`: Decompose hard/ambiguous problems and pick an approach.
- `pre-implement`: Produce implementation strategy before writing code.
- `frontend`: Produce implementation-ready FE plan with code-level detail.

## Frontend Rule (Strict)

For FE work, always use `--mode frontend` and require output that is immediately implementable, including:

- Component/page hierarchy and responsibilities
- State and data flow design
- API contract expectations and error/loading states
- Responsive behavior (mobile + desktop)
- Accessibility requirements
- Styling/theming strategy
- Complete file tree for the target feature/page
- Full copy-paste-ready code for core files (not pseudocode)
- At least one test file with runnable test logic
- Short runbook (deps, wiring steps, run command)

If Gemini returns only high-level guidance or partial snippets, continue with:

- `scripts/gemini-consult.sh followup --resume latest --prompt "Regenerate as full implementation package with complete file contents."`

## Commands

```bash
# New consultation
scripts/gemini-consult.sh ask \
  --mode pre-implement \
  --task "Implement optimistic UI for comment posting" \
  --context-file docs/requirements.md \
  --context-file src/features/comments/api.ts

# Frontend-specific consultation (must include code-level plan)
scripts/gemini-consult.sh ask \
  --mode frontend \
  --task "Redesign checkout page for mobile-first UX and keep desktop parity" \
  --context-file src/pages/checkout.tsx

# Non-frontend: request full code reference package when needed
scripts/gemini-consult.sh ask \
  --mode pre-implement \
  --implementation-package \
  --task "Refactor token refresh flow in auth middleware" \
  --context-file src/auth/middleware.ts

# Continue the same Gemini conversation
scripts/gemini-consult.sh followup \
  --resume latest \
  --prompt "Refine step 3 with concrete React component code"

# Inspect available sessions
scripts/gemini-consult.sh sessions
```

## Hard Constraints

- Allow Gemini to use read-only tools for codebase exploration and web/documentation lookup.
- Never use Gemini to modify codebase files in this workflow.
- Keep Gemini in read-only mode via `--approval-mode plan` (enforced by script).
- Default model is fixed to `gemini-3.1-pro-preview`.
- Treat Gemini output as decision support; verify against local code before implementing.

## Resources

- `scripts/gemini-consult.sh`: Wrapper for read-only Gemini consultation and session continuation.
- `references/prompt-recipes.md`: Prompt recipes and quality checklist by mode.
