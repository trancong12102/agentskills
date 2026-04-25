---
name: ts-reuse-review
description: "Reviews TypeScript/JavaScript diffs for reinvented utilities and missed reuse of standard libraries, native APIs, or existing workspace helpers. Use when reviewing TS/JS code changes, auditing a diff, checking a PR before merge, writing new utility helpers, or refactoring existing helpers. Detects installed project libraries and prefers those when present. Reports findings as text only — never edits files. Do not use for non-JS/TS code, documentation, configuration, or trivial single-line changes."
---

# TS Reuse Review

Scans a TypeScript/JavaScript diff for code that reinvents existing utilities. Reports a prioritized list of reuse opportunities — external libs (es-toolkit, date-fns, zod), ES2020+ native APIs, installed project libs (effect, remeda, ts-pattern, etc.), and already-existing internal helpers. Never applies edits.

## Prerequisites

- **ripgrep** — used for internal helper search. Install: `brew install ripgrep` or `cargo install ripgrep`.
- **ast-grep** — structural pattern matching. Install: `npm i -g @ast-grep/cli` or `brew install ast-grep`.

If `ast-grep` is missing, fall back to `ripgrep`-only detection and flag the degraded mode in the report header. Do not abort.

## Workflow

Do not read script source code. Run scripts directly and use `--help` for usage. Scripts live under `scripts/` relative to this skill's directory.

### Step 1: Determine review scope

If the scope is not already clear from the invocation, use AskUserQuestion:

- **Uncommitted changes** (default) — staged, unstaged, and untracked TS/JS files
- **Branch diff** — current branch vs a base branch
- **Specific commit** — one changeset by SHA

Extract the list of changed `.ts`, `.tsx`, `.js`, `.jsx`, `.mjs`, `.cjs` files. Skip `*.d.ts`, generated files under `**/dist/**`, `**/build/**`, `**/.next/**`, `**/node_modules/**`, and test fixtures under `**/fixtures/**`.

### Step 2: Detect installed project libraries

Run `scripts/detect-libs.sh` to scan `package.json` (plus workspace `packages/*/package.json` if monorepo) for relevant libs. It returns a JSON-ish list of installed libs grouped by domain:

- **general utils**: es-toolkit, lodash-es, ramda, remeda, radash
- **date**: date-fns, dayjs, luxon
- **schema**: zod, valibot, yup, superstruct, arktype
- **async/effects**: effect, neverthrow, rxjs, ts-pattern
- **http/query**: ky, ofetch, axios, @tanstack/react-query, swr
- **collections**: immer, immutable

The output decides the "prefer" tier for Step 5. Fixed targets (es-toolkit, date-fns, zod) stay in the catalog even when not installed — the report suggests installing them.

### Step 3: Extract changed code regions

For each changed file, get the new-side hunks via `git diff` and keep only added or modified lines. Discard pure deletions and context. Store as `{ file, startLine, endLine, content }` for pattern scanning.

### Step 4: Run pattern scan

Run `scripts/run-patterns.sh <file1> [<file2> ...]` on each changed file. The wrapper runs all rules in `scripts/patterns/` with the correct language per extension (`TypeScript` for .ts/.js/.mjs/.cjs, `Tsx` for .tsx/.jsx — ast-grep has separate parsers). Only keep matches whose line range overlaps the changed region.

The catalog is ~80 rules grouped into 14 categories (collection shape, timing/async, equality/clone, date, schema, effect-specific, web runtime, node APIs, native supersedes, correctness bugs, React hooks, event/pub-sub, i18n, stringly). Load `references/rule-categories.md` when you need to understand which rules fire for a diff or are adding new rules.

If ast-grep is unavailable, fall back to the ripgrep heuristic patterns embedded in each rule's `fallback_regex` metadata field and mark matches as `confidence: low`.

### Step 5: Cross-reference against catalogs

Load the references relevant to the hits from Step 4 — do not load every file:

**Always load (catalog core):**

- `references/external-libs.md` — fixed external catalog (es-toolkit + date-fns + zod).
- `references/native-apis.md` — ES2020+ through ES2025 natives (immutable array variants, `Object.hasOwn`, `Promise.withResolvers`, `URL.canParse`, Set operations, Iterator helpers).
- `references/project-libs.md` — dynamic handling of installed libs from Step 2.

**Load on match (progressive disclosure):**

- `references/react-patterns.md` — diff touches `.tsx` / `.jsx` files.
- `references/effect-patterns.md` — `effect` in installed deps.
- `references/node-vs-web.md` — `wrangler.toml`, `vercel.json` edge config, `deno.json`, or a file under `workers/` / `edge/` / `functions/` is in the diff.
- `references/testing-patterns.md` — diff includes test files AND `vitest` or `jest` is installed.
- `references/zod-patterns.md` — any zod-family rule fires (`email-regex`, `regex-uuid`, `regex-ipv4`, `regex-iso-datetime`, `zod-discriminated-union`, `url-validate-try`, `manual-schema-guard`) and zod/valibot/yup/arktype is installed.
- `references/query-patterns.md` — `react-fetch-useeffect` rule fires OR any of `@tanstack/react-query` / `swr` / `@reduxjs/toolkit/query` is installed.
- `references/form-patterns.md` — `react-hook-form` / `formik` / `@tanstack/react-form` / `@conform-to/react` installed AND diff contains a React form component.
- `references/immer-immutability.md` — `nested-spread-update` rule fires OR `immer` / `use-immer` / `mutative` installed.

This progressive-disclosure approach keeps the skill under the per-invocation context budget even as the catalog grows.

For each pattern match:

1. If the primary replacement lives in a **native API**, recommend the native (highest priority — zero deps).
2. Else if an **installed project lib** (from Step 2) has a canonical match, recommend that lib's import. Example: `effect` installed → prefer `Effect.retry` over es-toolkit `retry`.
3. Else if a **fixed external lib** matches (es-toolkit/date-fns/zod), recommend the import and — if the lib is not installed — suggest `bun add es-toolkit` (or npm/pnpm equivalent detected from the lockfile).
4. Drop the match if none apply.

### Step 6: Internal helper dedup check

Before emitting any external replacement, run `scripts/scan-internal-utils.sh <fn-name-candidates>` to grep workspace util directories (`src/**/utils/**`, `src/**/lib/**`, `packages/*/src/**`, `shared/**`, `common/**`) for existing helpers with matching names or call signatures.

If a matching internal helper exists, replace the external recommendation with `use existing: <path>:<line>`. This prevents suggesting a new import when the project already has the util.

### Step 7: Emit report

Output format — load `references/output-format.md` for the full template. Summary:

```text
ts-reuse-review findings:

<P?>  <file>:<line>
      pattern: <short description>
      replace: <suggested replacement>
      confidence: high|medium|low
      why: <trigger source — catalog entry, installed lib, native API, internal helper>

summary: <N> findings (<breakdown by priority>)
```

Priority levels:

- **P1** — clear reinvention, replacement is a one-line import swap, reduces ≥5 lines.
- **P2** — reinvention but replacement changes the shape slightly (named args vs positional, different null handling).
- **P3** — stylistic preference (native vs lib, one lib vs another already installed).
- Drop matches below P3 threshold to keep signal high.

End the report. Do not apply edits. Do not open files for modification.

## Rules

- **Report only** — Never invoke Edit, Write, or NotebookEdit. Output is text findings; the user or a downstream skill decides whether to apply changes. This makes the skill safe to run in parallel with other reviewers.
- **Scope to changed regions** — Matches outside added/modified hunks are noise. Do not flag pre-existing code the user did not touch in this diff.
- **Prefer native > installed-lib > fixed-catalog** — Order preserves dependency minimalism. A native API suggestion beats an es-toolkit suggestion even when es-toolkit is installed.
- **Check internal helpers before suggesting externals** — A workspace that already has `utils/chunk.ts` should not be told to import `chunk` from es-toolkit. Reuse trumps install.
- **Suggest install only when the fixed-catalog lib is missing and no native/internal alternative exists** — Avoid noisy "install es-toolkit" spam when a one-liner native works.
- **Do not flag trivial wrappers** — A 3-line helper around a native call is not a reuse violation unless it duplicates an installed lib's behavior. Minimum threshold: reinvention ≥5 lines OR exact signature match to a catalog entry.
- **Merge duplicate findings per file** — If the same reinvention appears on multiple lines of one file, collapse to one finding listing all line numbers.
- **Confidence calibration** — `high` when ast-grep structural match + catalog signature match; `medium` when regex-only match or catalog signature match only; `low` when fallback regex heuristic or ambiguous shape.
- **Language scope — TS/JS only** — Skip `.py`, `.rs`, `.go`, `.md`, `.json`, `.yml`. Extension enforcement happens in Step 1.
- **Never mutate workspace state** — No writes to `package.json`, no `bun install` execution. Install suggestions are text-only.
