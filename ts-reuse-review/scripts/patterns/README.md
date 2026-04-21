# ast-grep pattern rules

Each `*.yml` file is an ast-grep rule. Run against a single file:

```bash
ast-grep scan -r scripts/patterns/debounce-settimeout.yml path/to/file.ts
```

Or all rules over a dir:

```bash
ast-grep scan -c scripts/patterns path/to/dir
```

## Rule fields (custom metadata)

ast-grep rule schema + our metadata extensions:

- `id`, `language`, `message`, `severity`, `rule` — standard ast-grep fields.
- `metadata.replace_native` — preferred native-API replacement (highest priority).
- `metadata.replace_catalog` — es-toolkit / date-fns / zod replacement (fallback).
- `metadata.priority` — `P1` | `P2` | `P3` — see SKILL.md Step 7 thresholds.
- `metadata.fallback_regex` — ripgrep-compatible regex used when ast-grep is unavailable. Lower confidence.
- `metadata.notes` — optional reviewer context (DST, O(n²), crypto, etc).

## Adding a new rule

1. Identify the reinvention shape. Test against 2-3 real examples.
2. Write the `rule` section using ast-grep pattern syntax. Prefer `pattern:` for single-line shapes, `all:` + `has:` for multi-constraint.
3. Add the `metadata` block with at least `replace_native` or `replace_catalog`, plus `priority` and `fallback_regex`.
4. Smoke test: `ast-grep scan -r <rule>.yml <sample-file>` — ensure zero false positives on the sample.
5. Set `priority` conservatively — P3 for stylistic, P2 for shape-preserving swaps, P1 for perf/correctness wins.

## Language

Rules declare `language: TypeScript` or `language: Tsx`. ast-grep's TypeScript parser does not match `.tsx` / `.jsx` files (separate `Tsx` parser). The `scripts/run-patterns.sh` wrapper handles this transparently:

- `.ts` / `.js` / `.mjs` / `.cjs` → rules as written.
- `.tsx` / `.jsx` → rules with `language: TypeScript` swapped to `language: Tsx` via `sed` into a tmp dir.

Rules specifically for JSX/React components (e.g., `use-previous-manual`, `use-latest-ref`) declare `language: Tsx` directly — the wrapper only runs these for `.tsx` / `.jsx` files.
