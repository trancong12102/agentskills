# Native APIs that supersede utils

Highest-priority replacements. Zero dependencies, part of the runtime. Target: Node.js ≥18, modern browsers (ES2022+).

If `target` / `lib` in `tsconfig.json` excludes a feature, downgrade the confidence to `medium` and note the tsconfig target in the report.

## Cloning

| native               | supersedes                                   | shape to flag                                     |
| -------------------- | -------------------------------------------- | ------------------------------------------------- |
| `structuredClone(x)` | `cloneDeep`, `JSON.parse(JSON.stringify(x))` | `JSON.parse(JSON.stringify(...))` literal         |
| `{ ...obj }`         | shallow clone helpers                        | only for single-level; do not flag if deep needed |
| `[...arr]`           | `arr.slice()` for cloning                    | stylistic — P3                                    |

`structuredClone` handles Map/Set/Date/RegExp/typed arrays. `JSON.parse(JSON.stringify(x))` silently drops functions, Dates become strings, Maps become `{}`. High-confidence replacement.

## Object manipulation

| native                             | supersedes                               | shape to flag                                  |
| ---------------------------------- | ---------------------------------------- | ---------------------------------------------- |
| `Object.fromEntries(arr.map(...))` | `keyBy`, `_.fromPairs`, reduce-to-object | `arr.reduce((a, x) => ({ ...a, [k]: v }), {})` |
| `Object.entries(obj).map(...)`     | `mapKeys`/`mapValues` for simple cases   | manual `for...in` building new object          |
| `Object.keys(obj).length === 0`    | `isEmpty` for objects                    | fine as-is                                     |
| `{ ...a, ...b }`                   | shallow `merge`                          | P3 stylistic                                   |

Performance note: `arr.reduce((a, x) => ({ ...a, [k]: v }), {})` is O(n²) due to spread. `Object.fromEntries(arr.map(...))` is O(n). High-priority replacement for hot paths.

## Array

| native                                     | supersedes                      | shape to flag                                  |
| ------------------------------------------ | ------------------------------- | ---------------------------------------------- |
| `arr.at(-1)`                               | `last`, `arr[arr.length - 1]`   | exact index access from end                    |
| `arr.at(0)`                                | `first`, `arr[0]`               | P3, stylistic                                  |
| `arr.flat(1)`                              | `flatten`                       | `[].concat(...arr)`                            |
| `arr.flat(Infinity)`                       | `flattenDeep`                   | recursive flatten                              |
| `arr.flatMap(fn)`                          | `flatMap`, `map + flat`         | `arr.map(fn).flat()`                           |
| `Array.from({ length: n }, (_, i) => ...)` | `range(n)`                      | for-loop push                                  |
| `Array.fromAsync(iter)`                    | manual for-await + push         | ES2024 — collect async iterable into array     |
| `new Set(arr)` / `[...new Set(arr)]`       | `uniq`                          | manual de-dup loop                             |
| `arr.findLast(pred)`                       | lodash `findLast`               | reverse + find                                 |
| `arr.findLastIndex(pred)`                  | lodash `findLastIndex`          | reverse + findIndex                            |
| `arr.includes(x)`                          | `contains`, `indexOf !== -1`    | `arr.indexOf(x) !== -1` on simple arrays       |
| `Object.groupBy(arr, fn)`                  | lodash `groupBy`                | ES2024 — prefer es-toolkit if target < Node 21 |
| `Map.groupBy(arr, fn)`                     | `groupBy` when keys are objects | ES2024 — keyFn returning non-strings           |

`Object.groupBy(arr, fn)` and `Map.groupBy(arr, fn)` are ES2024. Node 21+. If tsconfig target is lower, prefer es-toolkit `groupBy`.

## Array — immutable variants (ES2023)

Non-mutating counterparts to the classic in-place methods. Use instead of the copy-then-mutate `[...arr].method()` idiom.

| native                          | supersedes                                                   | shape to flag                  |
| ------------------------------- | ------------------------------------------------------------ | ------------------------------ |
| `arr.toSorted(cmp)`             | `[...arr].sort(cmp)` / `arr.slice().sort(cmp)`               | copy-then-sort idioms          |
| `arr.toReversed()`              | `[...arr].reverse()` / `arr.slice().reverse()`               | copy-then-reverse              |
| `arr.toSpliced(s, d, ...items)` | `[...arr].splice(...)` (awkward) / manual slice-concat-slice | range edit returning new array |
| `arr.with(i, value)`            | `arr.map((v, j) => j === i ? value : v)`                     | positional replace             |

Node ≥20, modern browsers. Downgrade to `medium` if tsconfig target < ES2023. Matched by `array-tosorted` + `array-toreversed` rules.

## Object (ES2022+)

| native                    | supersedes                                                                   | shape to flag                                                           |
| ------------------------- | ---------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| `Object.hasOwn(obj, key)` | `Object.prototype.hasOwnProperty.call(obj, key)` / `obj.hasOwnProperty(key)` | Always safe on null-prototype objects. Matched by `object-hasown` rule. |

## String

| native                 | supersedes                          | shape to flag                     |
| ---------------------- | ----------------------------------- | --------------------------------- |
| `str.replaceAll(a, b)` | `str.split(a).join(b)`              | split/join replacement            |
| `str.padStart(n, '0')` | manual zero-padding loops           | string repeat + slice             |
| `str.matchAll(re)`     | manual iteration over regex matches | while-loop calling regex over str |
| `str.normalize('NFD')` | diacritic strip libs                | for accent-insensitive search     |

## URL

| native                                         | supersedes                | shape to flag                                       |
| ---------------------------------------------- | ------------------------- | --------------------------------------------------- |
| `new URL(str).searchParams`                    | query-string parsing libs | manual `str.split('?')[1].split('&')`               |
| `new URLSearchParams(obj).toString()`          | `qs.stringify`            | manual `Object.entries` + `encodeURIComponent` join |
| `url.pathname` / `url.hostname` / `url.origin` | regex URL parsing         | hand-rolled URL regex                               |

## Intl

| native                                                                            | supersedes                        | shape to flag                                                   |
| --------------------------------------------------------------------------------- | --------------------------------- | --------------------------------------------------------------- |
| `new Intl.DateTimeFormat(locale, opts).format(date)`                              | simple date-fns `format`          | `${d.getFullYear()}-${pad(d.getMonth()+1)}-${pad(d.getDate())}` |
| `new Intl.RelativeTimeFormat(locale).format(-5, 'minute')`                        | `formatDistance` for simple cases | manual "5 minutes ago" logic                                    |
| `new Intl.NumberFormat(locale, { style: 'currency', currency: 'USD' }).format(n)` | number-formatting libs            | `$${n.toFixed(2)}` hardcoded                                    |
| `new Intl.Collator(locale).compare(a, b)`                                         | locale-aware sort libs            | `a.localeCompare(b)` also works                                 |
| `new Intl.ListFormat(locale).format(['a','b','c'])`                               | manual `'a, b, and c'` joining    | `arr.join(', ').replace(/, ([^,]*)$/, ', and $1')`              |

## Promise / async

| native                                        | supersedes                                                                | shape to flag                                                         |
| --------------------------------------------- | ------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| `Promise.allSettled(...)`                     | `settle` wrappers                                                         | try/catch around each `await`                                         |
| `Promise.any(...)`                            | first-success helpers                                                     | rejections loop                                                       |
| `Promise.withResolvers()`                     | `let resolve, reject; new Promise((r, j) => { resolve = r; reject = j })` | ES2024 — Node ≥22, Bun ≥1.1. Matched by `promise-withresolvers` rule. |
| `AbortController` + `AbortSignal.timeout(ms)` | manual timeout race                                                       | `Promise.race([fn, timeout])`                                         |
| `AbortSignal.any([a, b])`                     | manual merge of multiple signals                                          | ES2024 — combine timeouts with user cancellation                      |
| `signal.throwIfAborted()`                     | manual abort checks                                                       | reading `signal.aborted` in loops                                     |

## Set operations (ES2025)

Native set math — Node ≥22, Chrome 122+, Safari 17+. Supersedes the hand-rolled or es-toolkit variants on `T[]` when the data is already `Set<T>`.

| native                           | supersedes                        | shape to flag                |
| -------------------------------- | --------------------------------- | ---------------------------- |
| `setA.union(setB)`               | `new Set([...a, ...b])`           | spread-into-Set pattern      |
| `setA.intersection(setB)`        | `[...a].filter((x) => b.has(x))`  | array filter with Set lookup |
| `setA.difference(setB)`          | `[...a].filter((x) => !b.has(x))` | same                         |
| `setA.symmetricDifference(setB)` | xor-equivalent on Sets            | manual double filter         |
| `setA.isSubsetOf(setB)`          | `[...a].every((x) => b.has(x))`   | every + has                  |
| `setA.isSupersetOf(setB)`        | mirror of above                   |                              |
| `setA.isDisjointFrom(setB)`      | `![...a].some((x) => b.has(x))`   | some + has                   |

Downgrade confidence to `medium` if tsconfig target < ES2025 or if the values are held in arrays (converting to Set just to call these is rarely a win unless the Set already exists).

## Iterators (ES2025)

| native                  | supersedes                         | shape to flag                                                  |
| ----------------------- | ---------------------------------- | -------------------------------------------------------------- |
| `iter.map(fn)`          | `[...iter].map(fn)` on large iters | Full materialization before transform — O(n) extra allocation. |
| `iter.filter(pred)`     | `[...iter].filter(pred)`           | Same.                                                          |
| `iter.take(n)`          | `[...iter].slice(0, n)`            | `take` is lazy — stops pulling after n.                        |
| `iter.reduce(fn, init)` | `[...iter].reduce(fn, init)`       | No intermediate array.                                         |

Node ≥22 under `--js-explicit-resource-management` (transitional), stable in 24+. Downgrade confidence on older targets.

## Misc

| native                                                | supersedes                             | shape to flag                                                                                                                                  |
| ----------------------------------------------------- | -------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| `crypto.randomUUID()`                                 | `uuid` v4 dep                          | `nanoid` for non-v4 is still valid                                                                                                             |
| `crypto.getRandomValues(buf)`                         | secure random libs                     | `Math.random()` for tokens                                                                                                                     |
| `btoa(str)` / `atob(b64)`                             | base64 libs for strings                | `Buffer.from(str).toString('base64')` is Node-only; `btoa` works both                                                                          |
| `Number.isInteger(x)`                                 | custom integer checks                  | `x === Math.floor(x)`                                                                                                                          |
| `Number.isFinite(x)`                                  | `isFinite` (global coerces)            | global `isFinite` with non-number input                                                                                                        |
| `Number.parseFloat(str)` / `Number.parseInt(str, 10)` | globals                                | globals are fine but strict form preferred                                                                                                     |
| `URL.canParse(str)`                                   | `try { new URL(str) } catch { false }` | ES2023. Matched by `url-validate-try` rule.                                                                                                    |
| `URL.parse(str)`                                      | same throwing behavior as constructor  | ES2025. Returns null on invalid input instead of throwing.                                                                                     |
| `String.prototype.isWellFormed` / `toWellFormed`      | manual surrogate-pair checks           | ES2024 — detect/fix lone surrogates in user input.                                                                                             |
| `RegExp` `v` flag                                     | ad-hoc unicode set arithmetic          | ES2024 — `/[\p{Letter}--[aeiou]]/v` for intersection/subtraction.                                                                              |
| `RegExp.escape(str)`                                  | hand-rolled regex escape helpers       | ES2025 — escapes regex metacharacters for inclusion in patterns.                                                                               |
| `Temporal.Now.plainDateTimeISO()`                     | `new Date()` + date-fns for TZ work    | Stage 3 — stable timezone handling. Polyfill: `@js-temporal/polyfill`. Suggest only if polyfill installed or tsconfig target hits stable spec. |

## When NOT to recommend native

- Target environment is old (React Native Hermes without Intl, IE, Node <14) — check tsconfig + `package.json` engines.
- Polyfill is already imported elsewhere in the file.
- Performance-critical hot path where the lib's optimized implementation is faster (rare — usually the opposite).

If unsure, mark confidence `medium` and note "verify runtime target".
