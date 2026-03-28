# Zustand v5 — Client UI State Management

## When to Use Zustand

| Tool        | Use for                                                                                         |
| ----------- | ----------------------------------------------------------------------------------------------- |
| `useState`  | Local, single-component state — toggles, local inputs                                           |
| Zustand     | Shared client UI state across components — theme, sidebar, selected items, filters, preferences |
| XState      | Complex flows with 3+ states, guards, async orchestration — wizards, auth, drag-and-drop        |
| React Query | Server data — never store API responses in Zustand                                              |

Do not use Zustand for server data. Do not use Zustand for state that belongs in the URL
(use TanStack Router search params instead).

---

## Key Patterns

### TypeScript: curried form is required

```typescript
// create<Type>()(creator) — the double parentheses are mandatory for TS inference
const useBearStore = create<BearState>()((set, get) => ({
  bears: 0,
  increase: (by) => set((state) => ({ bears: state.bears + by })),
}));
```

### Vanilla store for non-React access

Use `createStore` (from `zustand/vanilla`) when the store must be read/written from
XState actions, TanStack Router loaders, or plain modules:

```typescript
import { createStore } from "zustand/vanilla";

export const authStore = createStore<AuthState>()((set) => ({
  token: null,
  setToken: (token) => set({ token }),
  clear: () => set({ token: null }),
}));

// Non-reactive: getState(), setState(), subscribe()
```

Bind to React with `useStore(authStore, selector)` when needed in components.

---

## Selectors and Performance

### Always use selectors — never subscribe to the whole store

```typescript
// BAD — re-renders on ANY state change
const state = useStore();

// GOOD — only re-renders when bears changes
const bears = useBearStore((state) => state.bears);
```

### `useShallow` for multi-field selectors

Selecting an object or array creates a new reference every render. Wrap with `useShallow`:

```typescript
import { useShallow } from "zustand/react/shallow";

// BAD — new object every render → excess re-renders
const { count, text } = useStore((s) => ({ count: s.count, text: s.text }));

// GOOD
const { count, text } = useStore(
  useShallow((s) => ({ count: s.count, text: s.text })),
);
```

### Derived values — compute, don't store

```typescript
// BAD — stored derived value can go stale
const useStore = create(() => ({
  bears: 3,
  foodPerBear: 2,
  totalFood: 6,
}));

// GOOD — compute in selector
const totalFood = useBearStore((s) => s.bears * s.foodPerBear);
```

---

## Slicing Pattern

Split large stores into slices. Each slice is a `StateCreator` that knows the full
store type:

```typescript
const createBearSlice: StateCreator<
  BearSlice & FishSlice,
  [],
  [],
  BearSlice
> = (set) => ({
  bears: 0,
  addBear: () => set((state) => ({ bears: state.bears + 1 })),
});

// Combine with spread
const useBoundStore = create<BearSlice & FishSlice>()((...a) => ({
  ...createBearSlice(...a),
  ...createFishSlice(...a),
}));
```

---

## Middleware Best Practices

### Composition order: `devtools` outermost

```typescript
const useStore = create<MyState>()(
  devtools(
    persist(
      (set) => ({
        /* state */
      }),
      { name: "my-store" },
    ),
  ),
);
```

### `persist` — always use `partialize` to avoid persisting actions/derived state

```typescript
persist(
  (set) => ({
    /* state + actions */
  }),
  {
    name: "my-store",
    partialize: (state) => ({ bears: state.bears }), // only persist data fields
  },
);
```

### `devtools` — label actions for debuggability

```typescript
increment: () => set((s) => ({ count: s.count + 1 }), false, 'count/increment'),
```

### `subscribeWithSelector` — subscribe to slices outside React

```typescript
// Only fires when position.x changes, not on every state change
store.subscribe(
  (state) => state.position.x,
  (x) => console.log("x changed to", x),
);
```

---

## Integration with XState and TanStack Router

### XState actions read/write Zustand directly

```typescript
import { authStore } from "../stores/auth";

const machine = setup({
  actions: {
    clearAuth: () => {
      authStore.setState({ token: null, user: null });
    },
  },
}).createMachine({
  /* ... */
});
```

### Router loaders read Zustand for auth guards

```typescript
import { authStore } from "../stores/auth";

export const Route = createFileRoute("/dashboard")({
  loader: async ({ context: { queryClient } }) => {
    const { token } = authStore.getState();
    if (!token) throw redirect({ to: "/login" });
    await queryClient.ensureQueryData(dashboardQueryOptions);
  },
});
```

---

## Common Pitfalls

1. **Subscribing to the whole store** — always use selectors.

2. **Storing server data in Zustand** — use React Query. Zustand has no cache
   invalidation, background refetch, or deduplication.

3. **Skipping `useShallow` for multi-field selectors** — new object/array references
   cause excess re-renders.

4. **One monolithic store** — unrelated slices cause cross-component re-renders. Use
   multiple stores or the slicing pattern.

5. **Using Context for frequently-changing shared state** — Context re-renders all
   consumers. Use Zustand with selectors instead.

---

## v4 → v5 Key Gotchas

| v4 pattern                                     | v5 fix                                     |
| ---------------------------------------------- | ------------------------------------------ |
| `create(fn, shallow)` (equality fn as 2nd arg) | Use `useShallow` wrapper — 2nd arg removed |
| `shallow` import from `zustand/shallow`        | `useShallow` from `zustand/react/shallow`  |
| Default exports                                | Removed — use named exports only           |
| React 16.8+                                    | Min React 18                               |
