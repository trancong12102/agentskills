---
name: react-advanced
description: "Advanced React patterns and conventions for data fetching, tables, forms, state machines, client state management, schema validation, and testing. Use when tackling complex React problems — not simple component questions, but multi-concern tasks like server-driven tables with filtering, multi-step wizards, eliminating useEffect, Suspense architecture, choosing between state management approaches, or designing data flow across server/client/URL/form state. Do not use for web-specific routing/SSR or React Native-specific navigation/performance."
---

# React Advanced: Core Patterns & Conventions (Cross-Platform)

This skill defines the rules, conventions, and architectural decisions for building modern
React applications with the TanStack ecosystem and XState. It is intentionally opinionated
to prevent common pitfalls and enforce patterns that scale.

These patterns work identically on **web and React Native**. For platform-specific patterns:

- **Web:** see `react-web-advanced` (TanStack Router, Start, Virtual)
- **React Native:** see `react-native-advanced` (Expo Router, FlashList, MMKV)

For detailed API documentation of any library mentioned here, use other appropriate tools
(documentation lookup, web search, etc.) — this skill focuses on **how** and **why** to
use these tools, not their full API surface.

## Table of Contents

1. [The useEffect Ban](#the-useeffect-ban)
2. [State Management Philosophy](#state-management-philosophy)
3. [Architecture: Which Library Owns What](#architecture-which-library-owns-what)
4. [Component Composition](#component-composition)
5. [Common Pitfalls](#common-pitfalls)
6. [Reference Files](#reference-files)

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
2. **Browser/native API subscriptions with cleanup** — when `useSyncExternalStore` is too
   low-level for a one-off case (WebSocket connections, resize observers)
3. **Analytics/logging on mount** — fire-and-forget side effects with no state updates
4. **Bridging React Query data into XState** — the `useEffect` bridge pattern for
   pushing server state into a machine via events (see `references/xstate.md`)

### useMountEffect for mount-only effects

When you have a legitimate mount-only effect (cases 1–3 above), use a `useMountEffect`
helper instead of raw `useEffect(fn, [])`:

```typescript
// utils/useMountEffect.ts
import { useEffect, type EffectCallback } from "react";

// eslint-disable-next-line react-hooks/exhaustive-deps
const useMountEffect = (effect: EffectCallback) => useEffect(effect, []);

export default useMountEffect;
```

Usage:

```typescript
useMountEffect(() => {
  const plugin = $.myPlugin(ref.current);
  return () => {
    plugin.destroy();
  };
});
```

Why: raw `useEffect(fn, [])` triggers the `react-hooks/exhaustive-deps` lint rule and
makes the developer prove the empty array is intentional. `useMountEffect` makes the
"run once on mount" intent explicit in code and silences the warning correctly.

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
in the client) are fundamentally different concerns. Mixing them causes stale data bugs,
duplication, and synchronization nightmares.

| Concern           | Owner         | Examples                                                    |
| ----------------- | ------------- | ----------------------------------------------------------- |
| Server data       | React Query   | Users, posts, products, orders                              |
| URL/route state   | Router        | Path params, search params (TanStack Router or Expo Router) |
| Complex UI flows  | XState        | Multi-step wizards, auth flows, drag-and-drop               |
| Shared client UI  | Zustand       | Theme, sidebar, selected items, global filters, preferences |
| Form fields       | TanStack Form | Input values, validation errors, submission                 |
| Schema validation | Zod           | Search params, form validators, API contracts               |
| Simple local UI   | `useState`    | Toggle, accordion expanded, input focus                     |

### Decision flowchart

```text
Is the data from a server / API?
  YES -> React Query (queryOptions + useSuspenseQuery)
  NO -> Is it in the URL / route params?
    YES -> Router (platform-specific: TanStack Router or Expo Router)
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

| Layer              | Library        | Responsibility                                      |
| ------------------ | -------------- | --------------------------------------------------- |
| Server state       | React Query    | Fetching, caching, invalidation, background refetch |
| Complex UI state   | XState         | State machines, actor model, flow orchestration     |
| Shared client UI   | Zustand        | Cross-component UI state, preferences, selections   |
| Form lifecycle     | TanStack Form  | Field values, validation, submission                |
| Schema validation  | Zod            | Search params, form validators, API contracts       |
| Data display       | TanStack Table | Headless sorting, filtering, pagination, grouping   |
| Testing            | Vitest + TL    | Unit, component, integration, machine testing       |
| Simple local state | useState       | Toggles, local inputs, component-scoped values      |

### The golden rule: `queryOptions` as single source of truth

Define query options once, import everywhere — loaders, components, invalidation:

```typescript
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

3. **Duplicating URL state in useState** — use your router's search/param hooks directly.

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

---

## Reference Files

Read the relevant reference file when working with a specific library:

| File                        | When to read                                                          |
| --------------------------- | --------------------------------------------------------------------- |
| `references/react-query.md` | Query patterns, mutations, cache, Suspense integration                |
| `references/table.md`       | Column defs, sorting, filtering, pagination, server-side ops          |
| `references/form.md`        | Field validation, arrays, schema validation, performance              |
| `references/xstate.md`      | State machines, actors, auth flows, wizards, React integration        |
| `references/zustand.md`     | Shared client UI state, selectors, slices, middleware, vanilla stores |
| `references/zod.md`         | Schema validation, v4 API, Form integration, error handling           |
| `references/testing.md`     | Vitest setup, Testing Library, MSW, testing Query/Form/XState         |
| `references/integration.md` | Combining libraries: Zustand+XState, Query+XState bridge              |
