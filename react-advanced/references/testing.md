# Testing TanStack + XState with Vitest

This reference focuses on testing patterns specific to this stack. For general Vitest,
Testing Library, or MSW usage, consult their official docs.

For platform-specific testing:

- **Web:** see `react-web-advanced`'s `references/testing.md` (TanStack Router testing)
- **React Native:** see `react-native-advanced`'s `references/testing-rn.md` (RNTL, Expo Router)

---

## Testing React Query

### Fresh QueryClient per test — the #1 rule

```typescript
function createTestQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,       // fail fast — don't retry 3x in tests
        gcTime: Infinity,   // prevent garbage collection mid-test
      },
      mutations: { retry: false },
    },
  })
}

function createWrapper() {
  const queryClient = createTestQueryClient()
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  )
}
```

Never create `QueryClient` at module scope — it leaks cache between tests.

Use `findBy*` queries (async) for data states, `getBy*` for immediate loading states.
Override MSW handlers with `server.use()` for error scenarios.

### Testing custom hooks with `renderHook`

```typescript
test("useUsers returns data", async () => {
  const { result } = renderHook(() => useUsers(), {
    wrapper: createWrapper(),
  });

  expect(result.current.isPending).toBe(true); // v5: isPending, not isLoading

  await waitFor(() => expect(result.current.isSuccess).toBe(true));
  expect(result.current.data).toHaveLength(2);
});
```

---

## Testing TanStack Form

Use `userEvent.setup()` for realistic event simulation. Key pattern: `user.tab()` triggers
`onBlur` validation — use this to test field-level validators without submitting.

---

## Testing XState Machines

### Pure machine testing — no React, no DOM

Test machines as pure TypeScript. This is the primary testing strategy for XState:

```typescript
import { createActor, fromPromise } from "xstate";

test("transitions correctly", () => {
  const actor = createActor(toggleMachine);
  actor.start();

  expect(actor.getSnapshot().value).toBe("inactive");
  actor.send({ type: "TOGGLE" });
  expect(actor.getSnapshot().value).toBe("active");
});

test("guard blocks transition on empty cart", () => {
  const actor = createActor(checkoutMachine);
  actor.start();

  actor.send({ type: "CHECKOUT" });
  expect(actor.getSnapshot().value).toBe("cart"); // guard blocked
});
```

### `.provide()` — mock actions, guards, and invoked actors

```typescript
test("mock invoked actor", async () => {
  const actor = createActor(
    fetchMachine.provide({
      actors: {
        fetchUser: fromPromise(() =>
          Promise.resolve({ id: "1", name: "Alice" }),
        ),
      },
    }),
  );

  actor.start();
  actor.send({ type: "FETCH" });

  await vi.waitFor(() => {
    expect(actor.getSnapshot().value).toBe("success");
  });
});

test("override guard", () => {
  const actor = createActor(
    checkoutMachine.provide({
      guards: { hasItems: () => true },
    }),
  );

  actor.start();
  actor.send({ type: "CHECKOUT" });
  expect(actor.getSnapshot().value).toBe("checkout");
});
```

### XState `waitFor` for async machines

```typescript
import { waitFor } from "xstate";

const snapshot = await waitFor(actor, (s) => s.matches("success"), {
  timeout: 5000,
});
```

---

## Common Pitfalls

1. **Shared QueryClient across tests** — leaks cache. Always create fresh per test.

2. **Using `fireEvent` for user interactions** — `fireEvent.change` dispatches a single
   event. Use `userEvent.setup()` + `user.type()` for realistic behavior.

3. **Not awaiting async operations** — test passes before data loads (false positive).
   Use `findBy*` or `waitFor` for async content.

4. **Mocking `queryFn` directly** — use MSW for network-level mocking. Tests the full
   request/response pipeline including status codes and error shapes.

5. **Using `act()` manually** — `userEvent`, `waitFor`, and `findBy*` wrap `act()`
   automatically. Only needed for direct state updates outside React's event system.

6. **Not resetting MSW handlers** — `server.use()` overrides persist. Always have
   `afterEach(() => server.resetHandlers())` in setup.

7. **Testing implementation details** — test what users see (roles, text), not internal
   state or component structure.
