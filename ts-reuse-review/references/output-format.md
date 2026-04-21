# Report output format

Template for the final emitted report. Plain text, no JSON/YAML wrapper. Human-readable, but parseable by grep.

## Header

```text
ts-reuse-review — <scope-description>
scope: <uncommitted | branch <base>..HEAD | commit <sha>>
scanned: <N> file(s), <M> hunk(s)
detected libs: <comma-separated list from detect-libs.sh — group labels>
runtime: <ast-grep | ripgrep-fallback>
```

If ast-grep is missing, the `runtime` line must say `ripgrep-fallback (install @ast-grep/cli for higher precision)`.

## Findings block

Each finding is a 4-5 line stanza:

```text
<P?>  <relative-path>:<line[,line...]>
      pattern: <short one-line description of the reinvention shape>
      replace: <canonical replacement — import + call form>
      confidence: <high | medium | low>
      why: <one-line trigger source>
      [install: <command>]    # only when fixed-catalog lib is missing
```

Field rules:

- **priority** (`P1` / `P2` / `P3`) — see SKILL.md Step 7 for thresholds. No P0 tier — this skill does not flag critical issues; it flags reuse. Never use P4 either.
- **line(s)** — when the same reinvention appears multiple times in one file, list all lines comma-separated: `src/utils/array.ts:12,34,56`.
- **pattern** — describe the _shape_, not the variable names. "manual Array chunk via for-loop + slice" beats "chunks `items` into pages".
- **replace** — exact import + call. Include named imports, not default. Example: `import { chunk } from 'es-toolkit'; chunk(items, 50)`.
- **confidence** — `high` only when ast-grep matched a structural rule AND the catalog signature matches. `medium` when one of those holds. `low` when only regex heuristic matched.
- **why** — one of: `catalog: es-toolkit` / `catalog: date-fns` / `catalog: zod` / `installed: <lib-name>` / `native: <feature>` / `internal: <path>:<line>`. The tag makes report filterable.
- **install** — only present when `why: catalog: <lib>` AND the lib is not in detected deps. Command matches lockfile (bun/pnpm/yarn/npm).

## Sort order

1. Priority ascending (P1 first).
2. Within same priority, group by file, then by line ascending.
3. Within same file+line, put `internal:` replacements before `native:` before `installed:` before `catalog:` — internal reuse is the highest-value signal.

## Summary

After all findings, one summary line:

```text
summary: <total> findings — <P1 count> P1, <P2 count> P2, <P3 count> P3
  by source: <N> internal, <N> native, <N> installed, <N> catalog
```

If zero findings:

```text
ts-reuse-review — no reuse opportunities found in the reviewed scope.
```

## Full example

```text
ts-reuse-review — uncommitted changes
scope: uncommitted
scanned: 4 files, 6 hunks
detected libs: general (es-toolkit), date (date-fns), schema (zod), async (effect)
runtime: ast-grep

P1  src/api/batch.ts:23
    pattern: manual Array chunk via for-loop + slice
    replace: import { chunk } from 'es-toolkit'; chunk(items, 50)
    confidence: high
    why: catalog: es-toolkit

P1  src/hooks/useSearch.ts:14,42
    pattern: setTimeout-based debounce wrapper
    replace: import { debounce } from 'es-toolkit'; debounce(onSearch, 300)
    confidence: high
    why: catalog: es-toolkit

P2  src/utils/sleep.ts:5
    pattern: setTimeout Promise wrapper
    replace: use existing — packages/shared/timer.ts:12 sleep()
    confidence: high
    why: internal: packages/shared/timer.ts:12

P2  src/api/client.ts:88
    pattern: Promise.race-based timeout
    replace: AbortSignal.timeout(5000) + pass signal to fetch
    confidence: medium
    why: native: AbortSignal.timeout

P3  src/forms/validate.ts:40
    pattern: hand-rolled object-shape type guard (8 lines)
    replace: import { z } from 'zod'; const User = z.object({ id: z.string(), email: z.string().email() }); const isUser = (x: unknown) => User.safeParse(x).success
    confidence: medium
    why: catalog: zod

summary: 5 findings — 2 P1, 2 P2, 1 P3
  by source: 1 internal, 1 native, 0 installed, 3 catalog
```

## Do not include

- Raw diff snippets — the user already has the diff.
- File contents beyond the one-line pattern description.
- Explanations of why the lib exists ("es-toolkit is a modern lodash alternative...") — assume the reader knows.
- Cross-reviewer comparison — this skill runs standalone; never references other reviewers.
- Recommendations about code quality, types, or style beyond reuse — out of scope.
