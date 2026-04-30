---
name: effect-advanced
description: "Advanced Effect-TS patterns for typed errors, dependency injection, concurrency, resource management, schema validation, and streaming. Use when building Effect programs — not simple Effect.succeed/fail questions, but multi-concern tasks like designing service layers with Layer composition, handling typed error hierarchies with tagged errors, managing concurrent fibers with structured concurrency, scoped resource lifecycles, schema-driven API contracts, or integrating Effect with existing Express/Hono/database stacks. Do not use for basic TypeScript or general functional programming questions."
---

# Effect Advanced: Patterns, Conventions & Pitfalls

This skill defines the rules, conventions, and architectural decisions for building
production Effect-TS applications. It is intentionally opinionated to prevent common
pitfalls and enforce patterns that scale.

For detailed API documentation, use other appropriate tools (documentation lookup,
web search, etc.) — this skill focuses on **how** and **why** to use Effect idiomatically,
not the full API surface.

## Core Conventions

### Use `Effect.gen` for business logic

Generators read like synchronous code and are strongly preferred over long `.pipe` /
`.flatMap` chains for anything beyond trivial composition:

```typescript
const program = Effect.gen(function* () {
  const config = yield* ConfigService;
  const user = yield* UserRepo.findById(config.userId);
  return user;
});
```

Reserve `pipe` for data transformation pipelines and short combinator chains.

### Never throw — use Effect's error channel

| Instead of...           | Use                                 |
| ----------------------- | ----------------------------------- |
| `throw new Error()`     | `Effect.fail(new MyError())`        |
| `try/catch` on promises | `Effect.tryPromise({ try, catch })` |
| Callback APIs           | `Effect.async((resume) => ...)`     |
| Unrecoverable crashes   | `Effect.die(defect)`                |

### Functions over methods

Prefer `Effect.map(e, f)` over `e.pipe(Effect.map(f))` for composability and
tree-shaking. Flat imports (`import { Effect } from "effect"`) are fine for
applications; namespace imports (`import * as Effect from "effect/Effect"`) are
better for libraries.

### `@effect/schema` is deprecated

Schema has been merged into core `effect`. Import from `"effect"` directly:

```typescript
import { Schema } from "effect";
// NOT: import { Schema } from "@effect/schema"
```

### Use `NodeRuntime.runMain` in production

`Effect.runPromise` does not handle `SIGINT`/`SIGTERM` gracefully:

```typescript
import { NodeRuntime } from "@effect/platform-node";
NodeRuntime.runMain(program.pipe(Effect.provide(AppLayer)));
```

---

## Error Handling Philosophy

### Failures vs defects — the fundamental distinction

| Aspect        | Failure (expected)                   | Defect (unexpected)            |
| ------------- | ------------------------------------ | ------------------------------ |
| API           | `Effect.fail(new MyError())`         | `Effect.die(new Error())`      |
| Type channel  | Tracked in `E`                       | Never appears in `E` (`never`) |
| Recovery      | `catchTag`, `catchAll`, `retry`      | Only at system boundaries      |
| Rule of thumb | You intend to handle it at call site | Bug or impossible state        |

### Always use tagged errors

Plain `Error` or string failures miss the value of Effect's typed error channel:

```typescript
class UserNotFound extends Data.TaggedError("UserNotFound")<{
  readonly id: string;
}> {}

// Tagged errors are yieldable — no Effect.fail wrapper needed
const program = Effect.gen(function* () {
  const user = yield* db.findUser(id);
  if (!user) yield* new UserNotFound({ id });
  return user;
});
```

### `catchAll` does NOT catch defects

This is the #1 error handling mistake:

```typescript
Effect.catchAll(program, handler); // catches E only — NOT defects
Effect.catchAllCause(program, handler); // catches everything (E + defects + interrupts)
```

Only use `catchAllCause` / `catchAllDefect` at system boundaries (top-level error
handlers, HTTP response mappers).

---

## Dependency Injection Architecture

### Service → Layer → Provide (once)

```text
1. Define services with Context.Tag  →  "what do I need?"
2. Implement via Layers              →  "how is it built?"
3. Provide once at entry point       →  "wire it all together"
```

### Service methods must have `R = never`

Dependencies belong in Layer composition, not method signatures:

```typescript
// WRONG: leaks dependency to callers
findById: (id: string) => Effect.Effect<User, UserNotFound, Database>;

// RIGHT: Database is wired in the Layer
findById: (id: string) => Effect.Effect<User, UserNotFound>;
```

### Layer composition — know the operators

| Operation                             | When                 | Behavior                |
| ------------------------------------- | -------------------- | ----------------------- |
| `Layer.merge(A, B)`                   | Independent services | Both build concurrently |
| `Layer.provide(downstream, upstream)` | A feeds B            | upstream builds first   |
| `Layer.fresh(layer)`                  | Force new instance   | Bypasses memoization    |

**Critical:** `Layer.merge` does NOT sequence construction. If B depends on A, use
`Layer.provide`, not `Layer.merge`.

### One `Effect.provide` at the entry point

Scattered `provide` calls create hidden dependencies and layer duplication:

```typescript
// WRONG: provide scattered throughout codebase
const getUser = UserRepo.findById(id).pipe(Effect.provide(DbLayer));

// RIGHT: compose and provide once
const main = program.pipe(Effect.provide(AppLayer));
NodeRuntime.runMain(main);
```

---

## Resource & Scope Rules

### `Effect.scoped` is mandatory for `acquireRelease`

Forgetting `Effect.scoped` is the #1 resource management pitfall — resources
accumulate until the program exits:

```typescript
// WRONG: scope never closes, connection leaks
const result = yield * getDbConnection;

// RIGHT: scope closes when block completes
const result =
  yield *
  Effect.scoped(
    Effect.gen(function* () {
      const conn = yield* getDbConnection;
      return yield* conn.query("SELECT 1");
    }),
  );
```

### Release finalizers always run

On success, failure, AND interruption — guaranteed. The finalizer receives the
`Exit` value for conditional cleanup.

### Multiple resources in one scope

```typescript
Effect.scoped(
  Effect.gen(function* () {
    const conn = yield* Effect.acquireRelease(openConn(), closeConn);
    const file = yield* Effect.acquireRelease(openFile(), closeFile);
    // both released when scope closes, in REVERSE acquisition order
  }),
);
```

---

## Concurrency Model

### Prefer high-level APIs over raw fork

| API                                             | Use case                                   |
| ----------------------------------------------- | ------------------------------------------ |
| `Effect.all([], { concurrency: N })`            | Bounded parallel execution                 |
| `Effect.forEach(items, fn, { concurrency: N })` | Worker pool pattern                        |
| `Effect.race(a, b)`                             | First to complete wins, others interrupted |
| `Effect.timeout(e, dur)`                        | Deadline on any effect                     |

Only reach for `Effect.fork` / `Fiber` when high-level APIs are insufficient.

### Fork variants — know the lifecycle

| Function            | Scope          | Cleanup                         |
| ------------------- | -------------- | ------------------------------- |
| `Effect.fork`       | Parent's scope | Auto-interrupted with parent    |
| `Effect.forkDaemon` | Global scope   | Nothing cleans it up — you must |
| `Effect.forkScoped` | Nearest Scope  | Tied to resource lifecycle      |

**Gotcha:** `forkDaemon` leaks fibers if you forget to interrupt them.

---

## Common Pitfalls

1. **Floating effects** — creating an Effect without yielding or running it is a silent
   bug. `Effect.log("msg")` inside a generator does nothing unless `yield*`-ed.

2. **`catchAll` won't catch defects** — use `catchAllCause` at system boundaries for
   full failure visibility.

3. **Missing `Effect.scoped`** — `acquireRelease` without a scope boundary leaks resources
   until program exit.

4. **Scattered `Effect.provide`** — compose all layers and provide once at the entry point.

5. **Point-free on overloaded functions** — `Effect.map(myOverloadedFn)` silently erases
   generics. Use explicit lambdas: `Effect.map((x) => myOverloadedFn(x))`.

6. **`Effect.async` resume called multiple times** — resume must be called exactly once.
   Multiple calls cause undefined behavior.

7. **`orDie` silences errors** — converts typed failures to untyped defects. Handle errors
   properly instead.

8. **`Layer.merge` for dependent services** — merge doesn't sequence construction. Use
   `Layer.provide` when one layer needs another's output.

9. **`Fiber.join` vs `Fiber.await`** — `join` can cause premature finalizer execution in
   edge cases. Prefer `await` when resource safety matters.

10. **`runCollect` on infinite streams** — never call without a prior `take`. It will
    never terminate and consume unbounded memory.

11. **Using `it.effect` for scoped tests** — effects requiring `Scope` must use `it.scoped`,
    not `it.effect`, or you get a type error.

---

## Reference Files

Read the relevant reference file when working with a specific concern:

| File                                 | When to read                                                       |
| ------------------------------------ | ------------------------------------------------------------------ |
| `references/error-handling.md`       | Tagged errors, Cause, defect recovery, error mapping patterns      |
| `references/dependency-injection.md` | Services, Layers, composition, memoization, provide patterns       |
| `references/concurrency.md`          | Fibers, fork variants, Deferred, Semaphore, structured concurrency |
| `references/resource-management.md`  | Scope, acquireRelease, Layer resources, fork + scope interaction   |
| `references/schema.md`               | Schema definition, transforms, branded types, recursive schemas    |
| `references/stream.md`               | Stream operators, chunking, backpressure, resourceful streams      |
| `references/testing.md`              | @effect/vitest, TestClock, Layer mocking, Config mocking           |
| `references/platform.md`             | HTTP client, FileSystem, Command, runtime, framework integration   |
