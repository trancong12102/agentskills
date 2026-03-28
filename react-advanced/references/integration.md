# Ecosystem Integration — Combining TanStack + XState

## Wiring the Providers

The router receives `QueryClient` as context — the single integration point:

```typescript
// src/main.tsx
export const queryClient = new QueryClient({
  defaultOptions: { queries: { staleTime: 60_000 } },
})

const router = createRouter({
  routeTree,
  context: { queryClient },
  defaultPreload: 'intent',
  defaultPreloadStaleTime: 0,  // Let React Query manage staleness
})

declare module '@tanstack/react-router' {
  interface Register { router: typeof router }
}

root.render(
  <QueryClientProvider client={queryClient}>
    <RouterProvider router={router} />
  </QueryClientProvider>
)
```

Root route:
```typescript
interface RouterContext { queryClient: QueryClient }

export const Route = createRootRouteWithContext<RouterContext>()({
  component: () => <Outlet />,
})
```

`defaultPreloadStaleTime: 0` is intentional — without it, the router caches loader
results independently, causing React Query's staleTime to be ignored during preloads.

---

## Data Flow: Server -> Route -> Component

### 1. Define queryOptions (single source of truth)

```typescript
// queries/posts.ts
export const postQueryOptions = (postId: string) =>
  queryOptions({
    queryKey: ['posts', postId],
    queryFn: () => fetchPost(postId),
    staleTime: 30_000,
  })
```

### 2. Prefetch in route loader

```typescript
export const Route = createFileRoute('/posts/$postId')({
  loader: async ({ context: { queryClient }, params }) => {
    queryClient.prefetchQuery(commentsQueryOptions(params.postId))  // non-blocking
    await queryClient.ensureQueryData(postQueryOptions(params.postId))  // blocking
  },
  component: PostDetail,
})
```

### 3. Consume in component

```typescript
function PostDetail() {
  const { postId } = Route.useParams()
  const { data: post } = useSuspenseQuery(postQueryOptions(postId))
  return (
    <>
      <h1>{post.title}</h1>
      <Suspense fallback={<CommentsSkeleton />}>
        <Comments postId={postId} />
      </Suspense>
    </>
  )
}
```

### With TanStack Start server functions

```typescript
// serverFns/posts.ts
export const getPost = createServerFn({ method: 'GET' })
  .validator((data: { postId: string }) => data)
  .handler(async ({ data }) => db.posts.findUnique({ where: { id: data.postId } }))

// queries/posts.ts
export const postQueryOptions = (postId: string) =>
  queryOptions({
    queryKey: ['posts', postId],
    queryFn: () => getPost({ data: { postId } }),
  })
```

---

## XState Integration Patterns

### Pattern 1: Component-scoped machine

For self-contained UI flows (checkout wizard, multi-step form):
```typescript
function CheckoutPage() {
  const [snapshot, send] = useMachine(checkoutMachine)
  return (
    <div>
      {snapshot.matches('cart') && <CartStep onNext={() => send({ type: 'NEXT' })} />}
      {snapshot.matches('shipping') && <ShippingStep onNext={() => send({ type: 'NEXT' })} />}
    </div>
  )
}
```

### Pattern 2: Shared machine via createActorContext

For app-wide state (auth, notifications):
```typescript
export const AuthMachineContext = createActorContext(authMachine)

// Wrap app
<AuthMachineContext.Provider>
  <RouterProvider router={router} />
</AuthMachineContext.Provider>

// Consume anywhere
const user = AuthMachineContext.useSelector((s) => s.context.user)
```

### Pattern 3: Bridge React Query into XState

```typescript
function CheckoutFlow() {
  const [snapshot, send] = useMachine(checkoutMachine)
  const { data: cart } = useSuspenseQuery(cartQueryOptions)

  useEffect(() => {
    if (cart) send({ type: 'CART_LOADED', cart })
  }, [cart, send])

  const submitOrder = useMutation({
    mutationFn: createOrder,
    onSuccess: () => send({ type: 'ORDER_CONFIRMED' }),
    onError: (err) => send({ type: 'ORDER_FAILED', error: err.message }),
  })
}
```

---

## TanStack Form + XState

TanStack Form handles field values and validation. XState handles what happens with
submitted data (multi-step flows, confirmation dialogs):

```typescript
function ProfileForm() {
  const [snapshot, send] = useMachine(profileFormMachine)
  const updateProfile = useMutation({ mutationFn: updateProfileApi })

  const form = useForm({
    defaultValues: { name: '', email: '' },
    onSubmit: async ({ value }) => {
      send({ type: 'SUBMIT_START' })
      try {
        await updateProfile.mutateAsync(value)
        send({ type: 'SUBMIT_SUCCESS' })
      } catch (err) {
        send({ type: 'SUBMIT_ERROR', error: err.message })
      }
    },
  })

  if (snapshot.matches('success')) return <SuccessScreen />
  // ...form rendering
}
```

---

## TanStack Table + Virtual + React Query

```typescript
function UsersTable() {
  const { data: users } = useSuspenseQuery(usersQueryOptions)
  const [sorting, setSorting] = useState<SortingState>([])
  const containerRef = useRef<HTMLDivElement>(null)

  const table = useReactTable({
    data: users,
    columns,
    state: { sorting },
    onSortingChange: setSorting,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
  })

  const { rows } = table.getRowModel()
  const rowVirtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => containerRef.current,
    estimateSize: () => 48,
    overscan: 10,
  })

  return (
    <div ref={containerRef} style={{ height: '600px', overflow: 'auto' }}>
      {/* ... render virtual rows */}
    </div>
  )
}
```

---

## Suspense & Error Boundary Placement

### Route-level (automatic via TanStack Router)

```typescript
export const Route = createFileRoute('/dashboard')({
  loader: async ({ context: { queryClient } }) => {
    await queryClient.ensureQueryData(dashboardStatsQueryOptions)     // blocks
    queryClient.prefetchQuery(activityFeedQueryOptions)               // non-blocking
  },
  pendingComponent: () => <DashboardSkeleton />,
  errorComponent: ({ error }) => <DashboardError message={error.message} />,
  component: Dashboard,
})
```

### Within-route (granular)

```typescript
function Dashboard() {
  const { data: stats } = useSuspenseQuery(dashboardStatsQueryOptions)
  return (
    <div>
      <StatsPanel stats={stats} />
      <Suspense fallback={<ActivitySkeleton />}>
        <ActivityFeed />
      </Suspense>
    </div>
  )
}
```

### Error recovery with React Query

```typescript
errorComponent: ({ error, reset }) => {
  const router = useRouter()
  const queryErrorResetBoundary = useQueryErrorResetBoundary()
  useEffect(() => { queryErrorResetBoundary.reset() }, [queryErrorResetBoundary])
  return <button onClick={() => router.invalidate()}>Retry</button>
}
```

---

## Avoiding Waterfalls

- Route loaders: `Promise.all` for blocking, bare calls for non-blocking
- Parent + child route loaders run concurrently by default in TanStack Router
- Never fetch in `useEffect` what could go in a loader

---

## Testing Strategies

### React Query tests

Fresh `QueryClient` per test, retries disabled:
```typescript
function createTestQueryClient() {
  return new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
}
```

Use MSW for network-level mocking instead of mocking `queryFn`.

### XState tests

Pure TypeScript — test without React:
```typescript
const actor = createActor(checkoutMachine)
actor.start()
actor.send({ type: 'NEXT' })
expect(actor.getSnapshot().matches('shipping')).toBe(true)
```

### TanStack Router tests

Memory history for testing navigation:
```typescript
const router = createRouter({
  routeTree,
  history: createMemoryHistory({ initialEntries: ['/posts/1'] }),
  context: { queryClient: createTestQueryClient() },
})
```

---

## Common Anti-Patterns

1. **React Query AND XState for same data** — RQ owns server data. XState orchestrates
   UI flows only. Machine context holds UI state, not raw server data.

2. **Duplicating URL state in useState** — use `Route.useSearch()` directly.

3. **React hooks inside XState machines** — hooks cannot run outside React tree.

4. **Over-abstracting query keys** — centralize in `queryOptions` objects.

5. **XState for simple toggles** — `useState` for single booleans.

6. **SSR hydration mismatch with XState** — machine initial context must be deterministic.
   Pass initial data as `input` from route props, not from `window` or browser-only APIs.
