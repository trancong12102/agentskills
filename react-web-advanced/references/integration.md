# Web Integration — Router + Query + Zustand + Virtual

## Wiring the Providers

Key setup: pass `queryClient` as router context via `createRouter({ context: { queryClient } })`.
Set `defaultPreloadStaleTime: 0` — without it, Router caches preload results independently,
causing React Query's `staleTime` to be ignored. Register the router type with
`declare module '@tanstack/react-router' { interface Register { router: typeof router } }`.

---

## Data Flow: Prefetch Decision in Loaders

The key decision in route loaders: `ensureQueryData` (blocking — waits for data before
rendering) vs `prefetchQuery` (non-blocking — fire-and-forget, let component show fallback).

```typescript
loader: async ({ context: { queryClient }, params }) => {
  queryClient.prefetchQuery(commentsQueryOptions(params.postId)); // non-blocking
  await queryClient.ensureQueryData(postQueryOptions(params.postId)); // blocking
};
```

Use `Promise.all` for multiple blocking queries to avoid waterfalls. With TanStack Start,
wire server functions as `queryFn` in `queryOptions` — the integration is transparent.

---

## Zustand + TanStack Router — Auth Guards

Use `createStore` (vanilla) so Router loaders can read state synchronously:

```typescript
// stores/auth.ts
import { createStore } from "zustand/vanilla";

export const authStore = createStore<AuthState>()(
  persist(
    (set) => ({
      token: null,
      user: null,
      setAuth: (token, user) => set({ token, user }),
      clear: () => set({ token: null, user: null }),
    }),
    { name: "auth" },
  ),
);
```

```typescript
// routes/__root.tsx
export const Route = createRootRouteWithContext<RouterContext>()({
  beforeLoad: () => {
    const { token } = authStore.getState();
    return { isAuthenticated: !!token };
  },
});

// routes/(app)/dashboard.tsx
export const Route = createFileRoute("/(app)/dashboard")({
  beforeLoad: ({ context }) => {
    if (!context.isAuthenticated) throw redirect({ to: "/login" });
  },
});
```

---

## TanStack Table + Virtual + React Query

Wiring order: React Query provides `data` → Table processes it (`useReactTable`) → Virtual
renders visible rows (`useVirtualizer` on `table.getRowModel().rows`). See `virtual.md` for
virtualization patterns and `table.md` (in `react-advanced`) for table patterns.

---

## Suspense & Error Boundary Placement

- **Route-level**: use `pendingComponent`/`errorComponent` on route definitions
- **Within-route**: wrap non-blocking data in individual `<Suspense>` boundaries
- **Group co-dependent queries** under one `<Suspense>` so they resolve together

### Error recovery with React Query

```typescript
errorComponent: ({ error, reset }) => {
  const router = useRouter()
  const queryErrorResetBoundary = useQueryErrorResetBoundary()
  useEffect(() => { queryErrorResetBoundary.reset() }, [queryErrorResetBoundary])
  return <button onClick={() => router.invalidate()}>Retry</button>
}
```

Use `router.invalidate()` (not `reset()`) when the error came from a loader — it
coordinates both router reload and error boundary reset together.

---

## Common Pitfalls

1. **React Query AND XState for the same data** — React Query owns server data. XState
   orchestrates UI flows only.

2. **Duplicating URL state in useState** — use `Route.useSearch()` directly.

3. **SSR hydration mismatch with XState** — machine initial context must be deterministic.
   Pass initial data as `input` from route props, not from `window` or browser-only APIs.
