# Effect-TS reinventions

Load when `effect` is in installed deps. Skip otherwise — Effect adoption is all-or-nothing; suggesting Effect primitives in a non-Effect codebase creates churn.

Effect's surface is big. Most reinventions fall into one of these buckets: error handling, combinators, concurrency, schedules, data types, pattern matching. Prefer Effect primitives over external libs or hand-rolled code when inside `Effect.gen`/`Effect.fn`/pipe context.

## Error handling

| reinvention                                                              | Effect replacement                                               | notes                                                                                               |
| ------------------------------------------------------------------------ | ---------------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| `class MyError extends Error { readonly _tag = 'MyError' }`              | `class MyError extends Data.TaggedError('MyError')<{}> {}`       | Structural equality + Match.tag compat + proper printing. Matched by `effect-tagged-error` rule.    |
| `Effect.catchAll(e => { log(e); return Effect.fail(e) })`                | `Effect.tapError(log)`                                           | `tap*` variants don't swallow the error channel.                                                    |
| `Effect.flatMap(x => Effect.succeed(x))` / `Effect.map(x => Effect.y)`   | `Effect.flatMap(x => Effect.y)`                                  | Map returning an Effect is a type error in 3.x but still sometimes compiles as `Effect<Effect<A>>`. |
| `if (err instanceof FooError) ... else if (err instanceof BarError) ...` | `Effect.catchTag('Foo', ...)` or `Match.tag('Foo', ...)`         | Works with `Data.TaggedError`.                                                                      |
| `try { await fn() } catch (e) { throw new DomainError(e) }`              | `Effect.tryPromise({ try: fn, catch: e => new DomainError(e) })` | Keep typed error channel instead of unknown throw.                                                  |

## Combinators

| reinvention                                                     | Effect replacement                                                                 | notes                                                              |
| --------------------------------------------------------------- | ---------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| `Promise.all([eff1, eff2])`                                     | `yield* Effect.all([eff1, eff2], { concurrency: 'unbounded' })`                    | Matched by `effect-all-promise-all` rule. Preserves error channel. |
| `Effect.map(x => x)` / identity map                             | drop the map                                                                       | Matched by `identity-map` rule.                                    |
| Manual retry loop `for (let i=0; i<n; i++) { try {} catch {} }` | `Effect.retry(eff, { times: n })` or `Effect.retry(eff, Schedule.exponential(ms))` | Matched by `retry-loop` rule.                                      |
| `await new Promise(r => setTimeout(r, ms))` inside gen          | `yield* Effect.sleep(Duration.millis(ms))`                                         | `Duration.millis/seconds/minutes/hours` is idiomatic.              |
| Manual timeout race                                             | `Effect.timeout(eff, Duration.seconds(5))`                                         | Returns `Effect<A, TimeoutException \| E>`.                        |

## Concurrency / schedule

| reinvention                                        | Effect replacement                                          | notes                                                              |
| -------------------------------------------------- | ----------------------------------------------------------- | ------------------------------------------------------------------ |
| Manual chunked `Promise.all` for concurrency limit | `Effect.forEach(arr, fn, { concurrency: N })`               | Preferred over `p-limit` / `p-map` when already in Effect context. |
| Exponential backoff via `delay * 2^i`              | `Schedule.exponential(Duration.millis(ms), factor)`         | Composable with jitter, cap, recurs.                               |
| Manual jitter via `Math.random() * base`           | `Schedule.jittered(schedule)`                               | Combines with any Schedule.                                        |
| Polling via `setInterval` + fetch                  | `Effect.repeat(fetch, Schedule.fixed(Duration.seconds(5)))` | Also composes with `Schedule.whileInput`.                          |

## Option / nullable handling

| reinvention                                                      | Effect replacement                      | notes                                                              |
| ---------------------------------------------------------------- | --------------------------------------- | ------------------------------------------------------------------ |
| `Option.match(opt, { onNone: () => undefined, onSome: x => x })` | `Option.getOrUndefined(opt)`            | Matched by `option-match-undefined` rule.                          |
| `Option.match(opt, { onNone: () => null, onSome: x => x })`      | `Option.getOrNull(opt)`                 |                                                                    |
| `Option.match(opt, { onNone: () => default, onSome: x => x })`   | `Option.getOrElse(opt, () => default)`  |                                                                    |
| `opt._tag === 'None' ? fallback : opt.value`                     | `Option.getOrElse(opt, () => fallback)` | Don't read `_tag` directly — Effect's types guide you to the util. |

## Pattern matching

| reinvention                                                            | Effect replacement                                                                 | notes                                                   |
| ---------------------------------------------------------------------- | ---------------------------------------------------------------------------------- | ------------------------------------------------------- |
| `switch (x._tag) { case 'A': ...; case 'B': ...; default: throw ... }` | `Match.type<T>().pipe(Match.tag('A', ...), Match.tag('B', ...), Match.exhaustive)` | Matched by `switch-tag-match` + `exhaust-never-helper`. |
| Helper `function absurd(x: never): never { throw new Error('...') }`   | `Match.exhaustive`                                                                 | Drop the helper entirely.                               |

## Data types

| reinvention                                   | Effect replacement                            | notes                                                                        |
| --------------------------------------------- | --------------------------------------------- | ---------------------------------------------------------------------------- |
| Manual `_tag`-based discriminated unions      | `Data.TaggedClass` / `Data.taggedEnum`        | Adds structural equality + Hash.                                             |
| Custom `HashMap`/`HashSet` via plain objects  | `HashMap.fromIterable` / `HashSet.make`       | Only suggest if performance-relevant; plain objects are idiomatic otherwise. |
| Custom immutable list via spread-only updates | `Chunk` or `List`                             | Only suggest if profiling shows spread cost.                                 |
| Manual duration arithmetic `5 * 60 * 1000`    | `Duration.minutes(5).pipe(Duration.toMillis)` | Matched by `duration-ms-arith` rule.                                         |

## State

| reinvention                               | Effect replacement                    | notes                                           |
| ----------------------------------------- | ------------------------------------- | ----------------------------------------------- |
| `let counter = 0` mutated inside gen      | `const ref = yield* Ref.make(0)`      | Ref is fiber-safe; mutable closure state isn't. |
| Module-level singleton via `let instance` | `Layer.effect(Tag, Effect.sync(...))` | Only when already using Effect layers for DI.   |
| Manual in-memory cache `Map<K, V>`        | `Cache.make` / `Cache.makeWith`       | Adds TTL + capacity + stats out of the box.     |

## HTTP / platform

| reinvention                                 | Effect replacement                                                     | notes                                    |
| ------------------------------------------- | ---------------------------------------------------------------------- | ---------------------------------------- |
| `fetch` + try/catch + JSON parse + validate | `HttpClient.get(url).pipe(HttpClient.response.schemaBodyJson(Schema))` | Only if `@effect/platform` is installed. |
| Manual request retries                      | `HttpClient.retry(policy)` or `Effect.retry` on the client effect      |                                          |

## When not to suggest Effect

- File has no Effect imports and the change is localized. Switching a single helper to Effect drags the entire call site into Effect.
- Codebase is mostly React component-level code — Effect fits well in services/backends, less so in leaf React.
- User explicitly mentioned migrating away from Effect.
- Test files — Effect in tests is fine but not a reinvention to fix.

## Import shorthand

Common imports to emit in suggestions:

```ts
import {
  Effect,
  Data,
  Option,
  Duration,
  Schedule,
  Match,
  Ref,
  Cache,
} from "effect";
```

Effect uses named namespace imports. Never suggest `import * as Effect from 'effect/Effect'` — that form still works but is discouraged in 3.x docs.
