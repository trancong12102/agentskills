# Testing Web — TanStack Router & Integration

For React Query, TanStack Form, and XState testing patterns, see `react-advanced`'s
`references/testing.md`. This file covers web-specific testing with TanStack Router.

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

## Shared Test Wrapper — Router + React Query

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

1. **Not using `createMemoryHistory`** — browser history requires a real DOM.
   Always use `createMemoryHistory` with `initialEntries` in tests.

2. **Not awaiting navigation** — route transitions are async. Wrap assertions
   with `waitFor` or use `findBy*` queries.

3. **Shared QueryClient across tests** — always create fresh per test via
   `createTestQueryClient()`. See `react-advanced` testing reference.
