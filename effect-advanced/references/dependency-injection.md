# Dependency Injection — Services, Layers & Composition

## Defining Services

```typescript
import { Context, Effect, Layer } from "effect";

class UserRepository extends Context.Tag("app/UserRepository")<
  UserRepository,
  {
    readonly findById: (id: string) => Effect.Effect<User, UserNotFound>;
    readonly save: (user: User) => Effect.Effect<void>;
  }
>() {}
```

### Tag naming convention

Use file path style: `"my-app/users/UserRepository"`. Prevents tag collisions
across packages.

### Service methods must return `Effect<A, E, never>`

Dependencies belong in the Layer, not in method signatures:

```typescript
// WRONG: leaks Database requirement to all callers
readonly findById: (id: string) => Effect.Effect<User, UserNotFound, Database>

// RIGHT: Database is wired in the Layer
readonly findById: (id: string) => Effect.Effect<User, UserNotFound>
```

---

## Accessing Services

```typescript
const getUser = (id: string) =>
  Effect.gen(function* () {
    const repo = yield* UserRepository; // access via yield*
    return yield* repo.findById(id);
  });
// Type: Effect<User, UserNotFound, UserRepository>
```

The `R` parameter carries the requirement — TypeScript enforces that you provide it
before running.

---

## Creating Layers

### `Layer.succeed` — synchronous, no dependencies

```typescript
const UserRepositoryLive = Layer.succeed(UserRepository, {
  findById: (id) => Effect.tryPromise(() => db.findUser(id)),
  save: (user) => Effect.tryPromise(() => db.save(user)),
});
```

### `Layer.effect` — effectful, may depend on other services

```typescript
const UserRepositoryLive = Layer.effect(
  UserRepository,
  Effect.gen(function* () {
    const config = yield* Config;
    const db = yield* DbConnection;
    return {
      findById: (id) => Effect.tryPromise(() => db.query(id)),
      save: (user) => Effect.tryPromise(() => db.insert(user)),
    };
  }),
);
```

### `Layer.scoped` — with resource lifecycle

```typescript
const DatabaseLive = Layer.scoped(
  Database,
  Effect.acquireRelease(connectToDb(), (db) => db.disconnect()),
);
```

---

## Layer Composition

### `Layer.merge` — independent services (concurrent construction)

```typescript
const InfraLayer = Layer.merge(LoggerLive, MetricsLive);
// Both build concurrently, neither depends on the other
```

### `Layer.provide` — dependency chain (sequential construction)

```typescript
// LoggerLive needs ConfigLive → provide it
const LoggerWithConfig = Layer.provide(LoggerLive, ConfigLive);
```

### Composing a full application layer

```typescript
const AppLayer = Layer.provide(
  UserRepositoryLive,
  Layer.merge(
    Layer.provide(LoggerLive, ConfigLive),
    Layer.provide(DatabaseLive, ConfigLive),
  ),
);
```

### Memoization

Layers are memoized by object identity. If `ConfigLive` appears twice in the
dependency graph as the same reference, it is constructed only once.

Use `Layer.fresh(layer)` to bypass memoization (useful in tests for isolated instances).

---

## Providing Dependencies

### One `Effect.provide` at the entry point

```typescript
const main = program.pipe(Effect.provide(AppLayer));
NodeRuntime.runMain(main);
```

### Per-test providing

```typescript
it.effect("test", () => program.pipe(Effect.provide(TestLayers)));
```

---

## `Effect.Service` — Shorthand for Simple Services

For services that don't need separate interface/implementation split:

```typescript
class UserRepo extends Effect.Service<UserRepo>()("app/UserRepo", {
  effect: Effect.gen(function* () {
    const db = yield* Database;
    return {
      findById: (id: number) => db.query(users).where(eq(users.id, id)),
    };
  }),
  dependencies: [DatabaseLive],
}) {}
```

---

## Common Pitfalls

1. **`Layer.merge` for dependent services** — merge doesn't sequence construction.
   If B needs A's output, use `Layer.provide`.

2. **Scattered `Effect.provide`** — providing layers deep in the call tree creates
   hidden dependencies and can cause duplicate layer construction.

3. **Service methods with `R` requirements** — leaks implementation details to
   callers. Wire dependencies in the Layer.

4. **Tag collision across packages** — use namespaced tag strings like
   `"my-app/module/ServiceName"`.

5. **Forgetting `Layer.fresh` in tests** — shared memoized layers between tests
   can leak state. Use `Layer.fresh` for stateful services in test suites.
