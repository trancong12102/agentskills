---
name: react-advanced
description: "Advanced React patterns and conventions for data fetching, tables, forms, routing, state machines, client state management, schema validation, and testing. Use when tackling complex React problems — not simple component questions, but multi-concern tasks like server-driven tables with filtering, multi-step wizards, eliminating useEffect, Suspense architecture, choosing between state management approaches (Zustand vs XState vs useState), schema validation with Zod, or testing TanStack/XState code."
---

# React Advanced: Modern Patterns & Conventions

This skill defines the rules, conventions, and architectural decisions for building modern
React applications with the TanStack ecosystem and XState. It is intentionally opinionated
to prevent common pitfalls and enforce patterns that scale.

For detailed API documentation of any library mentioned here, use other appropriate tools
(documentation lookup, web search, etc.) — this skill focuses on **how** and **why** to
use these tools, not their full API surface.

## Table of Contents

1. [The useEffect Ban](#the-useeffect-ban)
2. [State Management Philosophy](#state-management-philosophy)
3. [Architecture: Which Library Owns What](#architecture-which-library-owns-what)
4. [Performance Patterns](#performance-patterns)
5. [Component Composition](#component-composition)
6. [Common Pitfalls](#common-pitfalls)
7. [File Organization](#file-organization)
8. [Reference Files](#reference-files)

---

## The useEffect Ban

Do not use `useEffect` for:

- **Data fetching** — use React Query (`useSuspenseQuery` / `useQuery`) or route loaders
- **Derived state** — compute during render or use `useMemo`
- **Syncing state with props** — use the prop directly, or reset with React `key`
- **Responding to user events** — put logic in event handlers
- **Subscribing to external stores** — use `useSyncExternalStore`
- **Complex async flows** — use XState machines with `invoke` / `fromPromise`

### Acceptable uses of useEffect

These are the **only** legitimate cases:

1. **Synchronizing with non-React external systems** — DOM APIs, third-party widgets
   (maps, charts), imperative libraries that need mount/unmount lifecycle
2. **Browser API subscriptions with cleanup** — when `useSyncExternalStore` is too
   low-level for a one-off case (WebSocket connections, resize observers)
3. **Analytics/logging on mount** — fire-and-forget side effects with no state updates
4. **Bridging React Query data into XState** — the `useEffect` bridge pattern for
   pushing server state into a machine via events (see `references/xstate.md`)

### What to use instead

| Instead of useEffect for...    | Use                                        |
| ------------------------------ | ------------------------------------------ |
| Fetching data                  | `useSuspenseQuery` + route loader prefetch |
| Consuming a Promise            | `use()` hook (React 19+) + `<Suspense>`    |
| Derived/computed values        | Direct computation or `useMemo`            |
| External store subscription    | `useSyncExternalStore`                     |
| Deferring expensive renders    | `useDeferredValue` / `useTransition`       |
| Complex async orchestration    | XState `invoke` with `fromPromise`         |
| Resetting state on prop change | React `key` prop on the component          |
| User-triggered side effects    | Event handlers directly                    |

---

## State Management Philosophy

### Server state vs client state — never mix them

**Server state** (data from APIs/databases) and **client state** (UI state existing only
in the browser) are fundamentally different concerns. Mixing them causes stale data bugs,
duplication, and synchronization nightmares.

| Concern           | Owner           | Examples                                                    |
| ----------------- | --------------- | ----------------------------------------------------------- |
| Server data       | React Query     | Users, posts, products, orders                              |
| URL state         | TanStack Router | Path params, search params, hash                            |
| Complex UI flows  | XState          | Multi-step wizards, auth flows, drag-and-drop               |
| Shared client UI  | Zustand         | Theme, sidebar, selected items, global filters, preferences |
| Form fields       | TanStack Form   | Input values, validation errors, submission                 |
| Schema validation | Zod             | Search params, form validators, API contracts               |
| Simple local UI   | `useState`      | Toggle, accordion expanded, input focus                     |

### Decision flowchart

```text
Is the data from a server / API?
  YES -> React Query (queryOptions + useSuspenseQuery)
  NO -> Is it in the URL?
    YES -> TanStack Router (useSearch / useParams)
    NO -> Is it a complex multi-state flow (3+ states, async, guards)?
      YES -> XState (useMachine / createActorContext)
      NO -> Is it a form field?
        YES -> TanStack Form (with Zod validators)
        NO -> Is it shared across components / trees?
          YES -> Zustand (create store + selectors)
          NO -> useState / useReducer
```

### When to reach for XState over useState/useReducer

Use XState when:

- There are **3+ mutually exclusive states** with defined transitions
- **Async side effects** must be cancelled on state change (race conditions)
- The logic has **guards** (conditions that gate transitions)
- You need **parallel states** for independent concerns
- The flow needs to be **tested in isolation** from React
- Multiple steps with **back/forward navigation** (wizards)

Do not use XState for simple toggles, single boolean flags, or counter state. That is
`useState` territory.

---

## Architecture: Which Library Owns What

| Layer               | Library             | Responsibility                                      |
| ------------------- | ------------------- | --------------------------------------------------- |
| Routing + URL state | TanStack Router     | Type-safe navigation, search params, route loaders  |
| Full-stack boundary | TanStack Start      | Server functions (`createServerFn`), SSR, streaming |
| Server state        | React Query         | Fetching, caching, invalidation, background refetch |
| Complex UI state    | XState              | State machines, actor model, flow orchestration     |
| Shared client UI    | Zustand             | Cross-component UI state, preferences, selections   |
| Form lifecycle      | TanStack Form       | Field values, validation, submission                |
| Schema validation   | Zod                 | Search params, form validators, API contracts       |
| Data display        | TanStack Table      | Headless sorting, filtering, pagination, grouping   |
| Large lists         | TanStack Virtual    | Virtualized rendering for 1000+ items               |
| Testing             | Vitest + TL + MSW   | Unit, component, integration, machine testing       |
| Simple local state  | useState/useReducer | Toggles, local inputs, component-scoped values      |

### The golden rule: `queryOptions` as single source of truth

Define query options once, import everywhere — loaders, components, invalidation:

```typescript
// queries/posts.ts
import { queryOptions } from "@tanstack/react-query";

export const postsQueryOptions = queryOptions({
  queryKey: ["posts"],
  queryFn: fetchPosts,
  staleTime: 30_000,
});

export const postQueryOptions = (postId: string) =>
  queryOptions({
    queryKey: ["posts", postId],
    queryFn: () => fetchPost(postId),
    staleTime: 30_000,
  });
```

### Route loader + React Query integration pattern

```typescript
// Route: prefetch in loader, consume in component
export const Route = createFileRoute('/posts/$postId')({
  loader: async ({ context: { queryClient }, params }) => {
    // Non-blocking secondary data
    queryClient.prefetchQuery(commentsQueryOptions(params.postId))
    // Blocking critical data
    await queryClient.ensureQueryData(postQueryOptions(params.postId))
  },
  component: PostDetail,
})

function PostDetail() {
  const { postId } = Route.useParams()
  // Data guaranteed in cache — instant, no loading state
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

---

## Performance Patterns

### React Compiler (React 19+)

React Compiler performs automatic memoization at build time. With the compiler enabled:

- **Do not** manually wrap components in `React.memo`
- **Do not** manually use `useMemo` / `useCallback` for performance
- **Do** write idiomatic React — the compiler handles memoization
- **Do** ensure code follows Rules of React (no mutation during render, no side effects
  in render, no reading mutable refs during render)

Manual `useMemo`/`useCallback` remain useful only for controlling effect dependencies,
not as performance tools.

### Suspense boundaries placement

- Route-level boundaries: use `pendingComponent` / `errorComponent` on route definitions
- Within routes: wrap non-blocking data in `<Suspense>` individually
- Group co-dependent queries under one `<Suspense>` so they resolve together
- Independent queries get separate `<Suspense>` boundaries

### Avoid waterfall requests

- Prefetch all independent data in route loaders using `Promise.all`:

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

### Code splitting

- Split routes using `.lazy()` or `.lazy.tsx` files — critical config (loader, params)
  stays in the main file, component/UI splits into the lazy file
- Use `React.lazy` for heavy on-demand components (rich editors, charts)
- Machine definitions auto-split since they are separate `.ts` files

---

## Component Composition

### Compound components

Use Context-based compound components when a group of components shares implicit state.
The parent manages state; children consume it through context. Memoize the context value.

### Slots pattern

Use named props for slot-like composition (`header`, `footer`, `actions`). Avoid deeply
nested render-prop trees.

### Inversion of Control

When adding boolean props or branching logic to handle caller-specific behavior, push
that logic back to the caller via callbacks, reducers, or render functions. Three similar
if-statements is a signal to invert control.

---

## Common Pitfalls

1. **Derived state in useEffect** — computing values in an effect and storing in useState
   causes double renders. Compute during render or use `useMemo`.

2. **Storing server data in client state** — putting API responses in Zustand/Redux means
   you own caching and invalidation. Use React Query instead.

3. **Duplicating URL state in useState** — use `Route.useSearch()` / `Route.useParams()`
   directly. TanStack Router's search params are a URL-synchronized store.

4. **Using React Query AND XState for the same data** — React Query owns fetching/caching.
   XState receives data via events and handles orchestration only.

5. **Calling React hooks inside XState machines** — hooks only work in React components.
   Use `fromPromise` in machines, bridge data via `useEffect` events.

6. **Array indices as keys** — use stable IDs (`item.id`). Index keys cause incorrect
   state association on reorder/insert/delete.

7. **Defining components inside components** — creates new component types each render,
   forcing React to unmount/remount. Define at module level.

8. **Context for high-frequency state** — Context re-renders all consumers on every
   change. Use Zustand with selectors for shared rapidly-changing values (see
   `references/zustand.md`), or local state if component-scoped.

9. **Not using `.catch()` on Zod search param schemas** — `.default()` only handles
   missing keys; `.catch()` also handles invalid values from malformed URLs.

10. **Wrong property order in `createFileRoute`** — must be
    `validateSearch -> loaderDeps -> beforeLoad -> loader` for TypeScript inference.

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
  stores/                  # Zustand stores (use createStore for vanilla access)
  serverFns/               # TanStack Start server functions
  components/
    ui/                    # Design system primitives
    shared/                # Cross-feature shared components
  lib/
    query-client.ts        # QueryClient singleton
    router.ts              # Router singleton
  test/
    setup.ts               # Vitest setup (jest-dom, MSW server lifecycle)
    test-utils.tsx          # Shared wrappers (renderWithProviders, re-exports)
    mocks/
      handlers.ts          # Default MSW handlers
      node.ts              # MSW setupServer
```

Key conventions:

- Machine definitions are pure TypeScript — no React imports, testable in isolation
- `queries/` files export `queryOptions` objects, not hooks
- Route-specific components use `-` prefix directories to avoid route tree inclusion
- Pathless route groups `(name)/` for organization without URL impact
- Zustand stores use `createStore` (vanilla) when accessed from XState or Router loaders
- Co-locate test files next to source (`.test.ts` / `.test.tsx`)
- Machine tests are pure TypeScript — no DOM environment needed

---

## Reference Files

Read the relevant reference file when working with a specific library:

| File                        | When to read                                                          |
| --------------------------- | --------------------------------------------------------------------- |
| `references/react-query.md` | Query patterns, mutations, cache, Suspense integration                |
| `references/router.md`      | Routing, search params, loaders, code splitting, navigation           |
| `references/start.md`       | Server functions, SSR, middleware, deployment                         |
| `references/table.md`       | Column defs, sorting, filtering, pagination, server-side ops          |
| `references/form.md`        | Field validation, arrays, schema validation, performance              |
| `references/virtual.md`     | Virtualization, dynamic heights, infinite scroll, grids               |
| `references/xstate.md`      | State machines, actors, auth flows, wizards, React integration        |
| `references/zustand.md`     | Shared client UI state, selectors, slices, middleware, vanilla stores |
| `references/zod.md`         | Schema validation, v4 API, Router/Form integration, error handling    |
| `references/testing.md`     | Vitest setup, Testing Library, MSW, testing Query/Router/Form/XState  |
| `references/integration.md` | Combining all libraries, data flow, Zustand patterns                  |
