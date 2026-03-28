# Ecosystem Integration — Cross-Platform Patterns

For platform-specific integration:

- **Web:** see `react-web-advanced`'s `references/integration.md` (Router+Query, Suspense)
- **React Native:** see `react-native-advanced`'s SKILL.md (Expo Router+Query, Stack.Protected)

This file covers integration patterns that work identically on web and React Native.

---

## Zustand + React Query — Never Duplicate

Zustand and React Query serve different purposes. Never store API data in Zustand:

```typescript
// BAD — duplicating server data in Zustand
const useStore = create((set) => ({
  users: [],
  fetchUsers: async () => {
    const users = await api.getUsers();
    set({ users }); // now you own caching, refetch, invalidation
  },
}));

// GOOD — Zustand for client UI state only
const useUiStore = create<UiState>()((set) => ({
  selectedUserId: null,
  setSelectedUser: (id) => set({ selectedUserId: id }),
}));

// React Query for server data
const { data: users } = useSuspenseQuery(usersQueryOptions);
const selectedId = useUiStore((s) => s.selectedUserId);
const selectedUser = users.find((u) => u.id === selectedId);
```

---

## Zustand + XState — Shared UI State from Machines

Use `createStore` (vanilla) so XState actions can read/write without React:

```typescript
// stores/ui.ts
import { createStore } from "zustand/vanilla";

export const uiStore = createStore<UiState>()((set) => ({
  sidebarOpen: false,
  toggleSidebar: () => set((s) => ({ sidebarOpen: !s.sidebarOpen })),
}));
```

```typescript
// machines/layout.ts — read/write Zustand from XState actions
import { uiStore } from "../stores/ui";

const layoutMachine = setup({
  actions: {
    closeSidebar: () => {
      uiStore.setState({ sidebarOpen: false });
    },
    readSidebar: () => {
      const { sidebarOpen } = uiStore.getState();
      // use value...
    },
  },
}).createMachine({
  /* ... */
});
```

---

## XState + React Query — Bridge Pattern

Never call React hooks inside a machine. The correct bridge:

```typescript
function CheckoutFlow() {
  const [snapshot, send] = useMachine(checkoutMachine);
  const { data: cart } = useSuspenseQuery(cartQueryOptions);

  // Bridge: push server state into machine via events
  useEffect(() => {
    if (cart) send({ type: "CART_LOADED", cart });
  }, [cart, send]);

  // Machine triggers mutations via actions
  const submitOrder = useMutation({
    mutationFn: createOrder,
    onSuccess: () => send({ type: "ORDER_CONFIRMED" }),
    onError: (err) => send({ type: "ORDER_FAILED", error: err.message }),
  });
}
```

React Query owns fetching/caching. XState receives data via events and handles
orchestration only.

---

See `xstate.md` for XState React integration patterns: component-scoped (`useMachine`) vs
shared (`createActorContext`), and `invoke` vs `spawnChild` distinction.

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

## Common Pitfalls

1. **React Query AND XState for the same data** — React Query owns server data. XState
   orchestrates UI flows only. Machine context holds UI state, not raw server data.

2. **React hooks inside XState machines** — hooks cannot run outside React tree.

3. **Over-abstracting query keys** — centralize in `queryOptions` objects.

4. **XState for simple toggles** — `useState` for single booleans.
