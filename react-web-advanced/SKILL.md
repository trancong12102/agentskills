---
name: react-web-advanced
description: "Web-specific React patterns for type-safe file-based routing, route-level data loading, server-side rendering, search param validation, code splitting, and list virtualization. Use when building React web apps with route loaders, SSR streaming, validated search params, lazy route splitting, or virtualizing large DOM lists. Do not use for React Native apps — use react-native-advanced instead."
---

# React Web Advanced: TanStack Router, Start & Virtual

Web-specific patterns for React apps built on the TanStack Router + Start + Virtual stack.
This skill extends `react-advanced` (core cross-platform patterns). Read that skill first for
React Query, XState, Zustand, Zod, TanStack Form, and TanStack Table conventions.

## Table of Contents

1. [Web Architecture](#web-architecture)
2. [Route Loader + React Query Pattern](#route-loader--react-query-pattern)
3. [Performance Patterns](#performance-patterns)
4. [File Organization](#file-organization)
5. [Common Pitfalls](#common-pitfalls)
6. [Reference Files](#reference-files)

---

## Web Architecture

The web stack adds three layers on top of the shared core:

| Layer               | Library          | Responsibility                                      |
| ------------------- | ---------------- | --------------------------------------------------- |
| Routing + URL state | TanStack Router  | Type-safe navigation, search params, route loaders  |
| Full-stack boundary | TanStack Start   | Server functions (`createServerFn`), SSR, streaming |
| Large lists         | TanStack Virtual | Virtualized rendering for 1000+ items               |

### The golden rule: `queryOptions` as single source of truth

Define query options once, import everywhere — loaders, components, invalidation:

```typescript
// queries/posts.ts
export const postsQueryOptions = queryOptions({
  queryKey: ["posts"],
  queryFn: fetchPosts,
  staleTime: 30_000,
});
```

### Router + React Query wiring

The router receives `QueryClient` as context — the single integration point:

```typescript
const router = createRouter({
  routeTree,
  context: { queryClient },
  defaultPreload: "intent",
  defaultPreloadStaleTime: 0, // Let React Query manage staleness
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
```

`defaultPreloadStaleTime: 0` is intentional — without it, the router caches loader results
independently, causing React Query's `staleTime` to be ignored during preloads.

---

## Route Loader + React Query Pattern

### ensureQueryData for blocking, prefetchQuery for non-blocking

```typescript
export const Route = createFileRoute('/posts/$postId')({
  loader: async ({ context: { queryClient }, params }) => {
    // Fire-and-forget secondary data
    queryClient.prefetchQuery(commentsQueryOptions(params.postId))
    // Block route render until critical data is ready
    await queryClient.ensureQueryData(postQueryOptions(params.postId))
  },
  component: PostDetail,
})

function PostDetail() {
  const { postId } = Route.useParams()
  // Data guaranteed in cache — instant, no loading state
  const { data: post } = useSuspenseQuery(postQueryOptions(postId))
  return <h1>{post.title}</h1>
}
```

### Avoid waterfall requests

Prefetch all independent data in route loaders using `Promise.all`:

```typescript
loader: async ({ context: { queryClient }, params }) => {
  await Promise.all([
    queryClient.ensureQueryData(userQueryOptions(params.id)),
    queryClient.ensureQueryData(permissionsQueryOptions(params.id)),
  ]);
  // Fire-and-forget for non-critical
  queryClient.prefetchQuery(activityQueryOptions(params.id));
};
```

- Never fetch data in `useEffect` that could go in a route loader
- Parent and child route loaders run concurrently by default

---

## Performance Patterns

### React Compiler (React 19+)

With the compiler enabled:

- **Do not** manually wrap components in `React.memo`
- **Do not** manually use `useMemo` / `useCallback` for performance
- **Do** write idiomatic React — the compiler handles memoization
- **Do** ensure code follows Rules of React (no mutation during render)

Manual `useMemo`/`useCallback` remain useful only for controlling effect dependencies.

### Suspense boundaries placement

- Route-level boundaries: use `pendingComponent` / `errorComponent` on route definitions
- Within routes: wrap non-blocking data in `<Suspense>` individually
- Group co-dependent queries under one `<Suspense>` so they resolve together
- Independent queries get separate `<Suspense>` boundaries

### Code splitting

- Split routes using `.lazy()` or `.lazy.tsx` files — critical config (loader, params)
  stays in the main file, component/UI splits into the lazy file
- Use `React.lazy` for heavy on-demand components (rich editors, charts)
- Machine definitions auto-split since they are separate `.ts` files

---

## File Organization

```text
src/
  routes/                  # TanStack Router file-based routes
    __root.tsx             # Root layout, router context type
    (auth)/                # Route group — no URL impact
    (app)/
      users/
        $userId.tsx
        $userId.lazy.tsx   # Component-only code split
        -components/       # "-" prefix excludes from route tree
  queries/                 # queryOptions definitions — one file per entity
  mutations/               # useMutation wrappers
  machines/                # XState machine definitions (pure TS, no React)
  stores/                  # Zustand stores
  serverFns/               # TanStack Start server functions
  components/
    ui/                    # Design system primitives
    shared/                # Cross-feature shared components
  lib/
    query-client.ts        # QueryClient singleton
    router.ts              # Router singleton
  test/
    setup.ts               # Vitest setup
    test-utils.tsx         # renderWithProviders
    mocks/handlers.ts      # MSW handlers
```

Key conventions:

- Route-specific components use `-` prefix directories to avoid route tree inclusion
- Pathless route groups `(name)/` for organization without URL impact
- `.lazy.tsx` files export component/pendingComponent/errorComponent only
- Co-locate test files next to source (`.test.ts` / `.test.tsx`)

---

## Common Pitfalls

1. **Wrong property order in `createFileRoute`** — must be
   `validateSearch -> loaderDeps -> beforeLoad -> loader` for TypeScript inference.
   Install `@tanstack/eslint-plugin-router` with `create-route-property-order` rule.

2. **Returning entire search in `loaderDeps`** — invalidates cache on any param change.
   Extract only the deps the loader uses.

3. **`preload="intent"` without React Query cache** — preloaded data is discarded if user
   doesn't navigate. Combine with React Query's `ensureQueryData` for cache persistence.

4. **Not registering the router** — without `declare module Register`, everything is `any`.

5. **Forgetting `<Suspense>` around `<Await>`** — runtime error without a Suspense ancestor.

6. **`defaultPreloadStaleTime` not set to 0** — Router's default staleTime overrides React
   Query's staleTime during preloads, causing stale data to be served.

7. **`useLoaderData` in `notFoundComponent`** — not valid. Use `Route.useParams()` or
   pass data via `throw notFound({ data: ... })`.

8. **Building all pages before verifying auth flow** — implement and verify the auth flow
   first (login → session cookie → protected route guard → redirect) end-to-end before
   building feature pages. Auth integration bugs hide behind each other (7+ layers deep).
   See `references/ssr-auth.md`.

9. **`defaultPreload: "intent"` triggering auth checks on hover** — with intent preloading,
   `beforeLoad` fires on every link hover. If `beforeLoad` does auth validation, every hover
   triggers a session check. Disable intent preloading during auth debugging, or make auth
   checks cache-aware (`queryClient.getQueryData(sessionKey)` before fetching).

10. **Blanket `invalidateQueries()` cascading through auth hooks** —
    `queryClient.invalidateQueries()` with no key filter invalidates everything, including
    session queries. Auth reactive hooks refetch, components re-render, `beforeLoad` re-runs.
    Always scope invalidation to the specific entity query key.
    See `references/ssr-auth.md` for patterns.

---

## Reference Files

Read the relevant reference file when working with a specific library:

| File                        | When to read                                                |
| --------------------------- | ----------------------------------------------------------- |
| `references/router.md`      | Routing, search params, loaders, code splitting, navigation |
| `references/start.md`       | Server functions, SSR, middleware, deployment               |
| `references/virtual.md`     | Virtualization, dynamic heights, infinite scroll, grids     |
| `references/integration.md` | Router+Query wiring, auth guards, Suspense placement        |
| `references/ssr-auth.md`    | SSR cookie auth, Vite proxy, CORS, env vars, HMR stability  |
| `references/testing.md`     | Testing Router routes, renderWithProviders                  |
