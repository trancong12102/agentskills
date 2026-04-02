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

This pattern works for **client-side SPAs with token-based auth only**. For SSR apps
with cookie-based auth, the session check should happen server-side in `beforeLoad` via
`createServerFn`, not from a client store. See `ssr-auth.md` for the SSR-safe pattern.
For Better Auth specifically, see `better-auth-start.md`.

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

## Query Key Namespacing for Auth Safety

Never put auth and data queries in the same key namespace. Blanket `invalidateQueries()`
after data mutations will cascade into session queries, triggering re-render storms.

```typescript
// Auth namespace — never auto-invalidated by data mutations
["auth", "session"][("auth", "activeOrg", orgId)][("auth", "orgs")][
  // Org-scoped data namespace — invalidated on org switch
  ("org", orgId, "projects")
][("org", orgId, "projects", projectId)][("org", orgId, "members")][
  ("org", orgId, "apiKeys")
][
  // Global data namespace
  "posts"
][("posts", postId)];
```

Global mutation handler that excludes auth:

```typescript
const queryClient = new QueryClient({
  mutationCache: new MutationCache({
    onSuccess: () => {
      queryClient.invalidateQueries({
        predicate: (q) => q.queryKey[0] !== "auth",
      });
    },
  }),
});
```

---

## Org-Scoped Queries and Switching

Embed `orgId` at a predictable position in query keys so org switch can target them:

```typescript
// queries/org-data.ts
export const orgKeys = {
  all: (orgId: string) => ["org", orgId] as const,
  projects: (orgId: string) => [...orgKeys.all(orgId), "projects"] as const,
  project: (orgId: string, projectId: string) =>
    [...orgKeys.projects(orgId), projectId] as const,
  members: (orgId: string) => [...orgKeys.all(orgId), "members"] as const,
  apiKeys: (orgId: string) => [...orgKeys.all(orgId), "apiKeys"] as const,
};
```

On org switch, use `removeQueries` (not `invalidateQueries`) to prevent stale cross-org
data from being shown briefly:

```typescript
function switchOrg(prevOrgId: string, nextOrgId: string) {
  // Hard remove all org-scoped data — no stale cross-org leakage
  queryClient.removeQueries({ queryKey: orgKeys.all(prevOrgId) });
  // Auth queries preserved — user identity doesn't change on org switch
}
```

**What to preserve on org switch:**

- `["auth", "session"]` — user identity unchanged
- `["auth", "orgs"]` — org list unchanged
- Global reference data (countries, currencies, etc.)

**What must be removed:**

- Everything under `["org", prevOrgId, ...]` — members, projects, settings, API keys

---

## Common Pitfalls

1. **React Query AND XState for the same data** — React Query owns server data. XState
   orchestrates UI flows only.

2. **Duplicating URL state in useState** — use `Route.useSearch()` directly.

3. **SSR hydration mismatch with XState** — machine initial context must be deterministic.
   Pass initial data as `input` from route props, not from `window` or browser-only APIs.

4. **Blanket `invalidateQueries()` nukes auth namespace** — always scope invalidation
   to the specific entity key, or use a `predicate` to exclude `["auth", ...]`.

5. **Org switch without `removeQueries`** — `invalidateQueries` keeps stale data in
   cache and shows it briefly. Use `removeQueries` for hard data isolation.
