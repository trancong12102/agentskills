# Resource Management — Scope, acquireRelease & Lifecycle

## `acquireRelease` Semantics

Release finalizers run on **success, failure, and interruption** — guaranteed.
The finalizer receives the `Exit` value for conditional cleanup:

```typescript
const resource = Effect.acquireRelease(
  // acquire:
  Effect.tryPromise(() => openConnection()),
  // release (always runs):
  (conn, exit) => Effect.sync(() => conn.close()),
);
```

---

## `Effect.scoped` — Mandatory Scope Boundary

**Forgetting `Effect.scoped` is the #1 resource management pitfall.** Without it,
resources accumulate until the program exits:

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

---

## Multiple Resources in One Scope

Use multiple `acquireRelease` calls inside a single `Effect.scoped`. Resources are
released in **reverse acquisition order**:

```typescript
Effect.scoped(
  Effect.gen(function* () {
    const conn = yield* Effect.acquireRelease(openConn(), closeConn);
    const file = yield* Effect.acquireRelease(openFile(), closeFile);
    // file released first, then conn
    return yield* processData(conn, file);
  }),
);
```

---

## Layers and Scoped Resources

Use `Layer.scoped` for services with resource lifecycles:

```typescript
const DatabaseLive = Layer.scoped(
  Database,
  Effect.acquireRelease(connectToDb(), (db) => db.disconnect()),
);
```

The layer's scope is managed by the runtime — the resource is acquired when the
layer is built and released when the program exits.

---

## `Effect.ensuring` — Always-Run Finalizer

For simple "run this cleanup regardless of outcome" without the acquire/release
pattern:

```typescript
const program = doWork().pipe(Effect.ensuring(Effect.sync(() => cleanup())));
```

---

## Fork + Scope Interaction

`Effect.fork` attaches the child fiber to the parent scope. When the parent scope
closes, the child fiber is interrupted.

**Gotcha:** If you fork a long-running background task with plain `fork` inside a
short-lived scope, it will be interrupted when the scope closes.

Solutions:

- `Effect.forkDaemon` — detach from all scopes (but you must manage cleanup)
- `Effect.forkIn(scope)` — attach to a specific longer-lived scope
- `Effect.forkScoped` — attach to the nearest enclosing `Scope`

---

## Common Pitfalls

1. **Missing `Effect.scoped`** — the most common resource leak. Every
   `acquireRelease` needs a scope boundary.

2. **Using `it.effect` for scoped tests** — effects requiring `Scope` must use
   `it.scoped` from `@effect/vitest`, not `it.effect`.

3. **Assuming `Layer.scoped` auto-scopes** — the layer is scoped to the runtime,
   not to individual requests. For per-request resources, use `acquireRelease`
   inside your service methods with explicit `Effect.scoped`.

4. **Not checking `Exit` in finalizers** — if cleanup differs between success
   and failure, inspect the `exit` parameter in the release function.

5. **Long-lived forks in short-lived scopes** — background tasks forked with
   `Effect.fork` die when their parent scope closes. Use `forkDaemon` or
   `forkIn` with an appropriate scope.
