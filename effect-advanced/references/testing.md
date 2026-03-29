# Testing — @effect/vitest, TestClock & Layer Mocking

## Test Variants

```typescript
import { it } from "@effect/vitest";
import { Effect, TestClock, Duration, Fiber } from "effect";

// Auto-injects TestContext (TestClock at 0, deterministic TestRandom)
it.effect("test name", () =>
  Effect.gen(function* () {
    // ...assertions
  }),
);

// For effects requiring Scope (acquireRelease, Command.start, etc.)
it.scoped("resource test", () =>
  Effect.gen(function* () {
    const conn = yield* Effect.acquireRelease(open(), close);
    // ...
  }),
);

// Uses real system clock and live services
it.live("real time test", () =>
  Effect.gen(function* () {
    // real network, real clock
  }),
);
```

**Critical:** Use `it.scoped` — not `it.effect` — for tests with scoped resources.
Using `it.effect` causes a type error because the effect still has `Scope` in its
requirements.

---

## Controlling Time with TestClock

The canonical **fork -> adjust -> verify** pattern:

```typescript
it.effect("timeout fires after 1 minute", () =>
  Effect.gen(function* () {
    // 1. Fork the effect under test
    const fiber = yield* Effect.fork(
      Effect.sleep(Duration.minutes(5)).pipe(
        Effect.timeout(Duration.minutes(1)),
      ),
    );
    // 2. Advance virtual clock
    yield* TestClock.adjust(Duration.minutes(1));
    // 3. Observe outcome
    const result = yield* Fiber.join(fiber);
    expect(Option.isNone(result)).toBe(true);
  }),
);
```

**Critical:** You must fork first, then adjust. If you adjust before forking, the
sleep is already past and the test is nondeterministic.

---

## Mocking Services via Layers

Prefer Layer-based mocking over `vi.mock`:

```typescript
const UserRepositoryTest = Layer.succeed(UserRepository, {
  findById: (id) => Effect.succeed({ id, name: "Test User" }),
  save: (_user) => Effect.succeed(undefined),
});

it.effect("creates a user", () =>
  Effect.gen(function* () {
    const result = yield* createUser("test@example.com");
    expect(result.name).toBe("Test User");
  }).pipe(Effect.provide(UserRepositoryTest)),
);
```

---

## `it.layer` — Shared Layer Across Suite

Constructs the layer **once** and shares it across all tests:

```typescript
const { it } = await import("@effect/vitest");

const TestLayer = Layer.merge(UserRepositoryTest, ConfigTest);

describe("UserService", () => {
  it.layer(TestLayer)("test 1", () =>
    Effect.gen(function* () {
      /* ... */
    }),
  );
  it.layer(TestLayer)("test 2", () =>
    Effect.gen(function* () {
      /* ... */
    }),
  );
});
```

**Gotcha:** Layers provided via `it.layer` do NOT automatically receive `TestContext`.
If your layer needs `TestClock`, you must explicitly compose:
`Layer.merge(MyLayer, TestContext.TestContext)`.

For stateful services needing per-test isolation, use `.pipe(Effect.provide(testLayer))`
on each test individually instead of `it.layer`.

---

## Mocking Config

```typescript
import { ConfigProvider, Layer } from "effect";

const testConfig = Layer.setConfigProvider(
  ConfigProvider.fromMap(
    new Map([
      ["DATABASE_URL", "postgres://localhost/test"],
      ["API_KEY", "test-key"],
    ]),
  ),
);

it.effect("reads config", () => program.pipe(Effect.provide(testConfig)));
```

---

## Schema Round-Trip Testing

Always test non-trivial schemas for roundtrip consistency:

```typescript
test("DateFromString roundtrip", () => {
  const date = new Date();
  const encoded = Schema.encodeSync(DateFromString)(date);
  const decoded = Schema.decodeSync(DateFromString)(encoded);
  expect(decoded.getTime()).toBe(date.getTime());
});
```

---

## Common Pitfalls

1. **`it.effect` for scoped tests** — use `it.scoped` when the effect requires `Scope`.

2. **Adjusting TestClock before forking** — the sleep is already past and the test is
   nondeterministic. Always fork first, then adjust.

3. **`it.layer` without TestContext** — layers provided via `it.layer` don't get
   `TestClock` automatically. Wire it explicitly if needed.

4. **Shared memoized layers** — `Layer.fresh` prevents state leakage between tests
   for stateful services.

5. **Using `vi.mock` for services** — Layer-based mocking is idiomatic, type-safe,
   and doesn't pollute the module system.
