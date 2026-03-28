# Testing TanStack + XState with Vitest

This reference focuses on testing patterns specific to this stack. For general Vitest,
Testing Library, or MSW usage, consult their official docs.

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

### Testing loading → success → error states

```typescript
test('shows loading then data', async () => {
  render(
    <QueryClientProvider client={createTestQueryClient()}>
      <UserList />
    </QueryClientProvider>
  )

  expect(screen.getByText('Loading...')).toBeInTheDocument()
  await screen.findByText('Alice')
  expect(screen.queryByText('Loading...')).not.toBeInTheDocument()
})

test('shows error state', async () => {
  server.use(
    http.get('/api/users', () =>
      HttpResponse.json({ message: 'Error' }, { status: 500 })
    )
  )

  render(
    <QueryClientProvider client={createTestQueryClient()}>
      <UserList />
    </QueryClientProvider>
  )

  await screen.findByText(/error/i)
})
```

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

## Testing TanStack Router

### `renderWithRouter` utility

```typescript
import {
  createRouter, createRootRoute, createRoute,
  RouterProvider, Outlet, createMemoryHistory,
} from '@tanstack/react-router'

const testRootRoute = createRootRoute({ component: () => <Outlet /> })

function renderWithRouter(
  routes: any[],
  { initialLocation = '/', context = {} } = {},
) {
  const routeTree = testRootRoute.addChildren(routes)
  const router = createRouter({
    routeTree,
    history: createMemoryHistory({ initialEntries: [initialLocation] }),
    context,
  })

  return { ...render(<RouterProvider router={router} />), router }
}
```

### Testing params, search params, and navigation

```typescript
test('reads route params', async () => {
  const userRoute = createRoute({
    getParentRoute: () => testRootRoute,
    path: '/users/$userId',
    component: function UserPage() {
      const { userId } = userRoute.useParams()
      return <div>User: {userId}</div>
    },
  })

  renderWithRouter([userRoute], { initialLocation: '/users/42' })
  await screen.findByText('User: 42')
})

test('navigates on click', async () => {
  const user = userEvent.setup()

  const homeRoute = createRoute({
    getParentRoute: () => testRootRoute,
    path: '/',
    component: function Home() {
      const navigate = homeRoute.useNavigate()
      return <button onClick={() => navigate({ to: '/about' })}>Go</button>
    },
  })
  const aboutRoute = createRoute({
    getParentRoute: () => testRootRoute,
    path: '/about',
    component: () => <h1>About</h1>,
  })

  const { router } = renderWithRouter([homeRoute, aboutRoute])
  await user.click(screen.getByRole('button', { name: /go/i }))

  await waitFor(() => {
    expect(router.state.location.pathname).toBe('/about')
  })
})
```

---

## Testing TanStack Form

### Test submission and validation through the UI

```typescript
test('submits with correct values', async () => {
  const user = userEvent.setup()
  const handleSubmit = vi.fn()

  render(<ContactForm onSubmit={handleSubmit} />)

  await user.type(screen.getByRole('textbox', { name: /name/i }), 'Alice')
  await user.type(screen.getByRole('textbox', { name: /email/i }), 'alice@test.com')
  await user.click(screen.getByRole('button', { name: /submit/i }))

  await waitFor(() => {
    expect(handleSubmit).toHaveBeenCalledWith({
      name: 'Alice',
      email: 'alice@test.com',
    })
  })
})

test('shows validation error then clears it', async () => {
  const user = userEvent.setup()
  render(<ContactForm onSubmit={vi.fn()} />)

  const input = screen.getByRole('textbox', { name: /name/i })
  await user.type(input, 'ab')
  await user.tab()  // trigger onBlur
  await screen.findByRole('alert')

  await user.clear(input)
  await user.type(input, 'alice')
  expect(screen.queryByRole('alert')).not.toBeInTheDocument()
})
```

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

## Shared Test Wrapper — Combining Providers

```typescript
// src/test/test-utils.tsx
export function renderWithProviders(
  routes: any[],
  { initialLocation = '/' } = {},
) {
  const queryClient = createTestQueryClient()
  const rootRoute = createRootRoute({ component: () => <Outlet /> })
  const routeTree = rootRoute.addChildren(routes)

  const router = createRouter({
    routeTree,
    history: createMemoryHistory({ initialEntries: [initialLocation] }),
    context: { queryClient },
  })

  return {
    ...render(
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    ),
    router,
    queryClient,
  }
}
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
