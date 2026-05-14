# Error Handling — Tagged Errors, Cause & Recovery Patterns

## Tagged Errors — The Required Pattern

Always use `Data.TaggedError` or `Schema.TaggedError`. Plain `Error` or string failures
miss the value of Effect's typed error channel.

### `Data.TaggedError` — simple, no runtime validation

```typescript
import { Data } from "effect";

class UserNotFound extends Data.TaggedError("UserNotFound")<{
  readonly id: string;
}> {}

class DatabaseError extends Data.TaggedError("DatabaseError")<{
  readonly cause: unknown;
}> {}
```

### `Schema.TaggedError` — with runtime validation and serialization

```typescript
import { Schema } from "effect";

class ValidationError extends Schema.TaggedError<ValidationError>()(
  "ValidationError",
  {
    field: Schema.String,
    message: Schema.String,
  },
) {}
```

Use `Schema.TaggedError` when errors cross process boundaries (HTTP responses,
message queues) or when you need schema-based encoding/decoding.

### Tagged errors are yieldable

No `Effect.fail` wrapper needed in generators:

```typescript
const program = Effect.gen(function* () {
  const user = yield* db.findUser(id);
  if (!user) yield* new UserNotFound({ id }); // direct yield
  return user;
});
```

---

## Catching Errors

### `catchTag` — the primary pattern

```typescript
Effect.catchTag("UserNotFound", (err) => Effect.succeed(defaultUser));
```

### `catchTags` — multiple tags in one call

```typescript
Effect.catchTags({
  UserNotFound: (err) => Effect.succeed(defaultUser),
  DatabaseError: (err) => Effect.fail(new ServiceUnavailable()),
});
```

### `catchAll` — all typed failures

```typescript
Effect.catchAll((err) => {
  // err is the union of all E types
  return Effect.succeed(fallback);
});
```

**Critical:** `catchAll` does NOT catch defects. Only failures in the `E` channel.

---

## Cause — The Lossless Failure Representation

`Cause<E>` captures all failure information without loss:

| Variant                    | Meaning             |
| -------------------------- | ------------------- |
| `Cause.fail(e)`            | Typed failure       |
| `Cause.die(defect)`        | Unhandled defect    |
| `Cause.interrupt(fiberId)` | Fiber interruption  |
| `Cause.parallel(c1, c2)`   | Concurrent failures |
| `Cause.sequential(c1, c2)` | Sequential failures |

### Full pattern matching with `Effect.exit`

```typescript
const exit = yield * Effect.exit(program);
if (Exit.isSuccess(exit)) {
  // exit.value
}
if (Exit.isFailure(exit)) {
  const cause = exit.cause;
  // inspect Cause<E>
}
```

---

## Defect Recovery — System Boundaries Only

```typescript
// Only at HTTP response mappers, top-level handlers, etc.
Effect.catchAllDefect(program, (defect) => {
  if (Cause.isRuntimeException(defect)) {
    return logAndReturn500(defect);
  }
  return Effect.die(defect); // re-throw unknown defects
});
```

### `catchAllCause` — everything

```typescript
Effect.catchAllCause(program, (cause) => {
  // handles failures + defects + interrupts
  if (Cause.isFailure(cause)) {
    /* typed error */
  }
  if (Cause.isDie(cause)) {
    /* defect */
  }
  if (Cause.isInterrupt(cause)) {
    /* fiber was interrupted */
  }
});
```

---

## Error Mapping — Handle Library Errors at the Source

Library errors (e.g., `HttpClientError`, `SqlError`) are generic. Catch and map them
to your domain errors immediately:

```typescript
const findUser = (id: string) =>
  db.query(`SELECT * FROM users WHERE id = $1`, [id]).pipe(
    Effect.catchTag("SqlError", (e) =>
      Effect.fail(new DatabaseError({ cause: e })),
    ),
    Effect.flatMap((rows) =>
      rows.length === 0
        ? Effect.fail(new UserNotFound({ id }))
        : Effect.succeed(rows[0]),
    ),
  );
```

---

## Common Pitfalls

1. **`catchAll` won't catch defects** — this is the #1 mistake. Use `catchAllCause`
   when you need full visibility.

2. **Using plain `Error` instead of tagged errors** — loses type-level discrimination.
   `catchTag` requires a `_tag` field.

3. **`orDie` silences typed errors** — converts failures to defects, discarding type
   information. Only use when you're absolutely sure the error is unrecoverable.

4. **Not mapping library errors** — letting generic `SqlError` or `HttpClientError`
   leak into your domain makes error handling fragile and coupled to implementation.

5. **Catching defects too eagerly** — `catchAllDefect` deep in the call stack hides
   bugs. Reserve it for system boundaries.
