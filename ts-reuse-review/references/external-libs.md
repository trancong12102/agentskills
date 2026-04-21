# External lib catalog — es-toolkit, date-fns, zod

Fixed targets. Always eligible for suggestion even when not installed. Install command detected from lockfile: `bun.lockb|bun.lock → bun add`, `pnpm-lock.yaml → pnpm add`, `yarn.lock → yarn add`, else `npm i`.

## es-toolkit

Modern lodash alternative. ESM, tree-shakeable, TypeScript native. Install: `bun add es-toolkit`.

Import form: `import { <fn> } from 'es-toolkit'` (core) or `'es-toolkit/compat'` (lodash-compat subset).

### Collection

| util            | signature                           | reinvention shape                                                                                      |
| --------------- | ----------------------------------- | ------------------------------------------------------------------------------------------------------ |
| `chunk`         | `(arr, size) => T[][]`              | `for (let i=0; i<arr.length; i+=n) arr.slice(i, i+n)`                                                  |
| `groupBy`       | `(arr, keyFn) => Record<K, T[]>`    | `arr.reduce((acc, x) => ({ ...acc, [k]: [...(acc[k] ?? []), x] }), {})`                                |
| `keyBy`         | `(arr, keyFn) => Record<K, T>`      | `arr.reduce((acc, x) => ({ ...acc, [x.id]: x }), {})` or `Object.fromEntries(arr.map(x => [x.id, x]))` |
| `partition`     | `(arr, pred) => [T[], T[]]`         | two filters with opposite predicates                                                                   |
| `uniqBy`        | `(arr, keyFn) => T[]`               | Map/Set dedup loops keyed on property                                                                  |
| `uniq`          | `(arr) => T[]`                      | `[...new Set(arr)]` is native; `uniq` equivalent for deep equality                                     |
| `sortBy`        | `(arr, keyFn) => T[]`               | `[...arr].sort((a,b) => a[k] > b[k] ? 1 : -1)`                                                         |
| `orderBy`       | `(arr, keyFns, orders) => T[]`      | multi-key sort with mixed asc/desc — manual sort with chained comparators                              |
| `zip`           | `(a, b) => [T, U][]`                | `a.map((x, i) => [x, b[i]])`                                                                           |
| `zipObject`     | `(keys, values) => object`          | `Object.fromEntries(keys.map((k, i) => [k, values[i]]))` — native preferred                            |
| `unzip`         | `(pairs) => [T[], U[]]`             | two maps over pairs                                                                                    |
| `countBy`       | `(arr, keyFn) => Record<K, number>` | reduce with `++acc[k]`                                                                                 |
| `pick`          | `(obj, keys) => Partial`            | destructure + rebuild object                                                                           |
| `pickBy`        | `(obj, pred) => Partial`            | `Object.fromEntries(Object.entries(obj).filter(([k,v]) => pred(v,k)))`                                 |
| `omit`          | `(obj, keys) => Partial`            | destructure rest                                                                                       |
| `omitBy`        | `(obj, pred) => Partial`            | `Object.fromEntries(Object.entries(obj).filter(([k,v]) => !pred(v,k)))`                                |
| `mapValues`     | `(obj, fn) => obj`                  | `Object.fromEntries(Object.entries(obj).map(([k,v]) => [k, fn(v)]))` — native preferred                |
| `mapKeys`       | `(obj, fn) => obj`                  | same shape as mapValues on keys                                                                        |
| `invert`        | `(obj) => obj`                      | `Object.fromEntries(Object.entries(obj).map(([k,v]) => [v,k]))`                                        |
| `flatten`       | `(arr) => T[]`                      | native `arr.flat()` preferred                                                                          |
| `flattenDeep`   | `(arr) => T[]`                      | native `arr.flat(Infinity)` preferred                                                                  |
| `difference`    | `(a, b) => T[]`                     | `a.filter(x => !b.includes(x))` — O(n\*m) manual vs Set-based                                          |
| `differenceBy`  | `(a, b, keyFn) => T[]`              | same with keyFn reshape                                                                                |
| `intersection`  | `(a, b) => T[]`                     | `a.filter(x => b.includes(x))`                                                                         |
| `union`         | `(...arrs) => T[]`                  | `[...new Set(arrs.flat())]` native-ish                                                                 |
| `xor`           | `(a, b) => T[]`                     | symmetric difference — `[...a.filter(x => !b.includes(x)), ...b.filter(x => !a.includes(x))]`          |
| `shuffle`       | `(arr) => T[]`                      | Fisher-Yates loop reimplemented                                                                        |
| `sample`        | `(arr) => T`                        | `arr[Math.floor(Math.random() * arr.length)]`                                                          |
| `sampleSize`    | `(arr, n) => T[]`                   | shuffle + slice                                                                                        |
| `take` / `drop` | `(arr, n) => T[]`                   | `arr.slice(0, n)` / `arr.slice(n)` — stylistic                                                         |
| `takeWhile`     | `(arr, pred) => T[]`                | manual loop breaking on first false                                                                    |
| `dropWhile`     | `(arr, pred) => T[]`                | manual loop starting after last false                                                                  |

### Timing / async

| util              | signature                             | reinvention shape                                                                                                  |
| ----------------- | ------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `debounce`        | `(fn, ms) => debounced`               | `let timer; return (...args) => { clearTimeout(timer); timer = setTimeout(() => fn(...args), ms); }`               |
| `throttle`        | `(fn, ms) => throttled`               | `let last = 0; return (...args) => { const now = Date.now(); if (now - last >= ms) { last = now; fn(...args); } }` |
| `delay` / `sleep` | `(ms) => Promise<void>`               | `new Promise(r => setTimeout(r, ms))`                                                                              |
| `retry`           | `(fn, { retries, delay }) => Promise` | loop with try/catch + sleep                                                                                        |
| `once`            | `(fn) => fn`                          | `let called=false, result; return (...a) => called ? result : (called=true, result=fn(...a))`                      |
| `memoize`         | `(fn) => fn`                          | `const cache = new Map(); return (k) => { if (!cache.has(k)) cache.set(k, fn(k)); return cache.get(k); }`          |

### Equality / clone

| util          | signature                   | reinvention shape                                                    |
| ------------- | --------------------------- | -------------------------------------------------------------------- |
| `isEqual`     | `(a, b) => boolean`         | recursive key-by-key compare                                         |
| `isEqualWith` | `(a, b, cmp) => boolean`    | custom deep compare                                                  |
| `cloneDeep`   | `(x) => T`                  | `JSON.parse(JSON.stringify(x))` — native `structuredClone` preferred |
| `merge`       | `(target, ...sources) => T` | recursive object spread                                              |

### Math / misc

| util               | signature                   | reinvention shape                      |
| ------------------ | --------------------------- | -------------------------------------- |
| `clamp`            | `(n, min, max) => n`        | `Math.min(Math.max(n, min), max)`      |
| `inRange`          | `(n, lo, hi) => bool`       | `n >= lo && n < hi`                    |
| `random`           | `(lo, hi) => n`             | `Math.random() * (hi-lo) + lo`         |
| `range`            | `(start, end, step) => n[]` | for-loop pushing into array            |
| `sum` / `mean`     | `(arr) => n`                | `arr.reduce((a,b) => a+b, 0)`          |
| `sumBy` / `meanBy` | `(arr, keyFn) => n`         | `arr.reduce((a,x) => a + keyFn(x), 0)` |
| `minBy` / `maxBy`  | `(arr, keyFn) => T`         | reduce tracking min/max                |
| `round`            | `(n, precision) => n`       | `Math.round(n * 10**p) / 10**p`        |
| `floor`/`ceil`     | `(n, precision) => n`       | same idiom, different rounding         |

### String

| util         | signature               | reinvention shape                                        |
| ------------ | ----------------------- | -------------------------------------------------------- |
| `camelCase`  | `(str) => str`          | split on spaces/dashes/underscores + capitalize + join   |
| `kebabCase`  | `(str) => str`          | regex replace `[A-Z]` with `-lc`                         |
| `snakeCase`  | `(str) => str`          | regex replace `[A-Z]` with `_lc`                         |
| `pascalCase` | `(str) => str`          | camelCase with first letter upper                        |
| `capitalize` | `(str) => str`          | `str[0].toUpperCase() + str.slice(1)`                    |
| `startCase`  | `(str) => str`          | camelCase → space-separated capitalized words            |
| `truncate`   | `(str, opts) => str`    | `str.length > n ? str.slice(0, n) + '…' : str`           |
| `escape`     | `(str) => str`          | HTML escape: `&` / `<` / `>` / `"` / `'` replacements    |
| `unescape`   | `(str) => str`          | reverse of escape                                        |
| `words`      | `(str) => str[]`        | split on non-word chars                                  |
| `pad`        | `(str, n, char) => str` | center-pad — native `padStart`/`padEnd` only go one side |

## date-fns

Functional date manipulation. No `moment`-style mutation. Install: `bun add date-fns`.

Import form: `import { <fn> } from 'date-fns'`.

| util                                                                | signature                      | reinvention shape                                                  |
| ------------------------------------------------------------------- | ------------------------------ | ------------------------------------------------------------------ |
| `addDays`                                                           | `(date, n) => Date`            | `new Date(date.getTime() + n * 86400000)`                          |
| `addHours` / `addMinutes` / `addSeconds` / `addMonths` / `addYears` | `(date, n) => Date`            | manual ms arithmetic or `setMonth/setFullYear`                     |
| `subDays` etc.                                                      | `(date, n) => Date`            | negative of add                                                    |
| `differenceInDays` / `differenceInHours` / `differenceInMinutes`    | `(a, b) => n`                  | `Math.floor((a - b) / 86400000)`                                   |
| `startOfDay` / `startOfWeek` / `startOfMonth`                       | `(date) => Date`               | `new Date(d.getFullYear(), d.getMonth(), d.getDate())`             |
| `endOfDay` / `endOfWeek` / `endOfMonth`                             | `(date) => Date`               | set hrs/mins/secs to 23:59:59.999                                  |
| `format`                                                            | `(date, pattern) => string`    | template string with `getFullYear()`, `getMonth()+1`, zero-padding |
| `parseISO`                                                          | `(str) => Date`                | `new Date(str)` — works for ISO but date-fns validates             |
| `isBefore` / `isAfter` / `isEqual`                                  | `(a, b) => bool`               | `a.getTime() < b.getTime()`                                        |
| `isValid`                                                           | `(date) => bool`               | `!isNaN(date.getTime())`                                           |
| `formatDistance`                                                    | `(a, b) => string`             | manual ms → "5 minutes ago" logic                                  |
| `formatDistanceToNow`                                               | `(date) => string`             | same vs `new Date()`                                               |
| `formatRelative`                                                    | `(date, base) => string`       | "yesterday at 5:30pm" locale-aware                                 |
| `eachDayOfInterval` / `eachWeekOfInterval` / `eachMonthOfInterval`  | `({start, end}) => Date[]`     | manual `while (d <= end) push; d = addDays(d, 1)`                  |
| `isWithinInterval`                                                  | `(date, {start, end}) => bool` | `d >= start && d <= end`                                           |
| `intervalToDuration`                                                | `({start, end}) => Duration`   | manual hour/minute/sec decomposition from ms                       |
| `getUnixTime` / `fromUnixTime`                                      | `(date) => n` / `(n) => date`  | `Math.floor(date.getTime() / 1000)` / `new Date(n * 1000)`         |
| `nextMonday` / `nextSaturday` / ... / `previousFriday`              | `(date) => Date`               | manual `setDate(d.getDate() + ...)` until `getDay()` matches       |
| `startOfToday` / `endOfToday` / `startOfYesterday`                  | `() => Date`                   | `startOfDay(new Date())`                                           |
| `lastDayOfMonth` / `lastDayOfWeek`                                  | `(date) => Date`               | manual `new Date(year, month+1, 0)`                                |
| `setDefaultOptions({ locale, weekStartsOn })`                       | `() => void`                   | passing `{ locale }` to every date-fns call                        |

### Native supersedes for date

- `new Intl.DateTimeFormat(locale, opts).format(date)` → locale-aware formatting without date-fns for simple cases.
- `new Intl.RelativeTimeFormat(locale).format(-5, 'minute')` → `formatDistance` for simple cases.

If the match is locale-aware formatting and `Intl` suffices, prefer native over date-fns.

## zod

TypeScript-first schema validation. Install: `bun add zod`.

Import form: `import { z } from 'zod'`.

Reinvention shapes to flag:

- Manual type guards: `function isUser(x: unknown): x is User { return typeof x === 'object' && x !== null && 'id' in x && typeof x.id === 'string' && ... }` → `const User = z.object({ id: z.string(), ... }); const isUser = (x: unknown) => User.safeParse(x).success;`
- Hand-rolled API response validation: `if (typeof data.user?.email !== 'string') throw new Error(...)` → `UserResponse.parse(data)`.
- Boolean chains checking required fields on objects from `JSON.parse` / `fetch`.
- Assertion functions throwing on missing keys.
- Regex checks on strings already covered: `z.string().email()`, `z.string().url()`, `z.string().uuid()`, `z.string().datetime()`, `z.string().regex(/.../)`, `z.string().ip()`, `z.string().cuid()`, `z.string().cidr()`.
- `Number.isInteger(x) && x >= 0` → `z.number().int().nonnegative()`.
- Enum membership check `['a', 'b', 'c'].includes(x)` on typed input → `z.enum(['a', 'b', 'c'])`.
- `try { new URL(s) } catch { ... }` → `z.string().url()` at API boundary (matched by `url-validate-try` rule — prefer native `URL.canParse` for non-validation contexts).
- Hand-rolled email regex → `z.string().email()` (matched by `email-regex` rule).

### zod v4 / zod-mini (2025+)

| form        | when to suggest                                                            |
| ----------- | -------------------------------------------------------------------------- |
| `zod`       | default; full API, ~50kb.                                                  |
| `@zod/mini` | bundle-constrained targets (Workers, mobile); same inference, reduced API. |
| `@zod/core` | library authors wanting to expose zod types without shipping the runtime.  |

Only suggest `@zod/mini` when the project has a bundle-size budget evident (`size-limit`, `bundlesize`, `esbuild --metafile` in CI) or is an edge/worker deployment.

Confidence is `medium` for zod suggestions unless the reinvention is >10 lines or covers an API boundary (fetch/parse). Type guards under 5 lines may be intentional performance choices — do not flag.

## Install suggestion template

When a fixed-catalog lib is missing and no native / installed alternative covers the match, emit:

```text
    install: bun add <lib>    # or: pnpm add, yarn add, npm i — detected from lockfile
```

Do not suggest install for libs the user has already explicitly avoided (e.g., `.cursorrules` / `CLAUDE.md` forbids adding lib X — honor that). If the skill has no way to know, include the suggestion.
