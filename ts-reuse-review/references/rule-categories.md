# Rule categories

The pattern scan in Step 4 runs ~80 ast-grep rules grouped by domain. This file documents what each category covers — load when you need to know which rules fire for a given diff or want to add new rules.

## Categories

### collection shape

Reshape, filter, group, partition primitives.

`chunk`, `groupBy`, `keyBy`, `partition`, `uniqBy`, `sortBy`, `mapValues`, `pick`, `omit`, `zip`, `maxBy`, `minBy`, `sum`, `range`, `compact`, `findindex-splice-remove`

### timing / async

Throttle, debounce, retry, semaphore, abort.

`debounce`, `throttle`, `sleep`, `retry`, `timeout`, `once`, `memoize`, `abort-controller-flag`, `concurrency-chunk`, `manual-semaphore`, `console-timer-manual`

### equality / clone / update

Deep equality, deep clone, immutable update.

`deepClone (structuredClone)`, `isEqual`, `merge`, `nested-spread-update → immer`

### date

Day arithmetic, comparisons, relative time.

`addDays`, `startOfDay`, `differenceInDays`, `isSameDay`, `relative-time`, `date-getday-weekday`

### schema / validation

Hand-rolled validators replaced by zod/valibot.

Hand-rolled object-shape assertions → zod/valibot; `email-regex`, `url-validate-try`, `regex-uuid`, `regex-ipv4`, `regex-iso-datetime`, `zod-discriminated-union`

### effect-specific

Idiomatic Effect-TS replacements.

`switch-on-._tag` → `Match.tag`, `Option.match` → `getOrUndefined`, `Duration.hours/minutes`, `exhaust(never)` → `Match.exhaustive`, identity `Effect.map`, `Promise.all` inside `Effect.gen`, custom Error with `_tag` → `Data.TaggedError`

### web / runtime (Workers/Edge)

Browser/runtime APIs supersede Node-style code.

`URL/URLSearchParams` parsing, HTML escape, `btoa`, `TextDecoder`, `crypto-createhash-node` → `subtle.digest`

### node fs / path / crypto

Modern Node API replacements.

`fs-readfile-callback`, `path-concat-string`, `crypto-createhash-node`

### native supersedes

ES2020+ natives replacing hand-rolled code.

`JSON.parse(JSON.stringify(x))`, `Object.fromEntries`, `arr.at(-1)`, `Array.from({length})`, `crypto.randomUUID`, `Object.hasOwn`, `arr.toSorted`/`arr.toReversed`, `Promise.withResolvers`, typeof-undefined-compare, nullish-chain-coerce, `new Map(chained set)`

### correctness bugs (P1 auto)

Always P1 — replacement also fixes a latent bug.

`array-fill-same-init` (shared ref), `react-object-literal-dep` (defeats memoization)

### React hooks

Common React hook reinventions.

`usePrevious`, `useLatest-ref`, `react-fetch-useeffect`, `react-object-literal-dep`

### event / pub-sub

Manual event emitters → Node `EventEmitter` / `mitt`.

### i18n / format

Native `Intl` for locale sort, plural, currency.

`Intl.Collator`, `Intl.PluralRules`, `Intl.NumberFormat`

### stringly

Template literal abuse via `.replace({key})`.

## Fallback mode

When `ast-grep` is unavailable, the wrapper falls back to `fallback_regex` heuristics embedded in each rule's metadata. Matches in fallback mode are tagged `confidence: low` so the report header can warn the reader.

## Adding rules

New rules live in `scripts/patterns/` as `*.yml` files. Match the existing rule schema (id, language, rule body, metadata fields including `fallback_regex` and category). Update this file whenever a new category is introduced.
