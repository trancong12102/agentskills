# Prompt Recipes

Use these recipes with `scripts/gemini-consult.sh` to get reliable, high-signal output from Gemini.

## Core Rules

- Keep prompts concrete and tied to actual files, logs, and constraints.
- Use `--context-file` for key evidence (requirements, failing tests, stack traces, target source files).
- Use `followup --resume ...` to refine the same conversation instead of starting from scratch.
- Keep Gemini read-only; this skill is for planning/decision support, not direct code edits.
- Encourage Gemini to use read-only tools for codebase exploration and web/documentation verification before finalizing recommendations.
- Frontend tasks require full implementation packages by default.
- Non-frontend tasks can request full code references on demand via `--implementation-package`.
- Require evidence-first outputs:
  - include concrete codebase evidence (paths + symbols/literals)
  - include official web/doc URLs in format `URL (accessed YYYY-MM-DD)` for external/version-sensitive claims
  - mark unknowns explicitly as `UNVERIFIED`
  - when evidence is insufficient, return `UNVERIFIED` findings and next evidence-gathering steps instead of definitive conclusions
  - avoid tool-control chatter in final output

## Mode Matrix

| Mode | Use For | Expected Output |
| --- | --- | --- |
| `decision` | Choosing between options | Tradeoff table + recommendation |
| `plan` | Execution sequencing | Ordered steps + checkpoints |
| `debug` | Investigating hard bugs | Ranked hypotheses + isolation plan |
| `problem-solving` | Ambiguous/high-complexity tasks | Decomposition + selected strategy |
| `pre-implement` | Before coding | Architecture + implementation strategy (or full package with `--implementation-package`) |
| `frontend` | UI/UX and FE implementation planning | Full implementation package: file tree + full file contents + test + runbook |

## Examples

```bash
# 1) Decision checkpoint before implementation
scripts/gemini-consult.sh ask \
  --mode decision \
  --task "Choose state management strategy for checkout flow: URL state vs context vs store" \
  --context-file docs/checkout-requirements.md \
  --context-file src/pages/checkout.tsx

# 2) Debug consultation with evidence
scripts/gemini-consult.sh ask \
  --mode debug \
  --task "Investigate intermittent 401 after token refresh" \
  --context-file logs/auth-errors.log \
  --context-file src/lib/auth-client.ts

# 3) Frontend full implementation package (required for FE tasks)
scripts/gemini-consult.sh ask \
  --mode frontend \
  --task "Design and implement a mobile-first product listing page with filters and skeleton loading" \
  --context-file src/pages/products.tsx \
  --context-file src/components/ProductCard.tsx

# 4) Non-frontend: request full code package only when needed
scripts/gemini-consult.sh ask \
  --mode pre-implement \
  --implementation-package \
  --task "Refactor auth token refresh flow and provide copy-paste-ready files" \
  --context-file src/auth/middleware.ts \
  --context-file src/auth/token-service.ts

# 5) Continue and force full code package if response is too high-level
scripts/gemini-consult.sh followup \
  --resume latest \
  --prompt "Regenerate as full implementation package with complete file contents and one runnable test file."
```

## Frontend Quality Checklist

For `--mode frontend`, ensure Gemini output includes:

1. Component hierarchy with clear ownership.
2. State/data flow and async/error/loading behavior.
3. Accessibility rules (keyboard, focus, semantics, contrast).
4. Responsive strategy with mobile and desktop behavior.
5. Styling/theming direction and motion guidance.
6. Complete file tree for the feature/page.
7. Full copy-paste-ready code for core files (no pseudocode placeholders).
8. At least one runnable test file.
9. Short runbook for install/wire/test.
10. Evidence section with codebase references and any external sources used.
11. Explicit `UNVERIFIED` list for unresolved gaps.
