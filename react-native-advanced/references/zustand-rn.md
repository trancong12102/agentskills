# Zustand Persist with MMKV in React Native

For core Zustand patterns (selectors, useShallow, slicing, middleware composition), see
`react-advanced`'s `references/zustand.md`. This file covers RN-specific persistence.

---

## MMKV Storage Adapter

MMKV is synchronous and runs on the native thread — ~30x faster than AsyncStorage, with
no async hydration gap.

```typescript
import { MMKV } from "react-native-mmkv";
import { StateStorage, createJSONStorage } from "zustand/middleware";

// Create at module level — NEVER inside a component or store factory
const mmkv = new MMKV();

const zustandStorage: StateStorage = {
  setItem: (name, value) => mmkv.set(name, value),
  getItem: (name) => mmkv.getString(name) ?? null, // must return null, not undefined
  removeItem: (name) => mmkv.delete(name),
};
```

Wire with `createJSONStorage`:

```typescript
import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";

export const useAppStore = create<AppState>()(
  persist(
    (set) => ({
      token: null,
      theme: "light" as const,
      setToken: (token: string | null) => set({ token }),
      setTheme: (theme: "light" | "dark") => set({ theme }),
    }),
    {
      name: "app-storage",
      storage: createJSONStorage(() => zustandStorage),
      partialize: (state) => ({ token: state.token, theme: state.theme }),
    },
  ),
);
```

---

## Encrypted Storage

For sensitive data (auth tokens, user PII):

```typescript
const secureStorage = new MMKV({
  id: "secure-storage",
  encryptionKey: "your-encryption-key", // or from Keychain/secure store
});

const secureZustandStorage: StateStorage = {
  setItem: (name, value) => secureStorage.set(name, value),
  getItem: (name) => secureStorage.getString(name) ?? null,
  removeItem: (name) => secureStorage.delete(name),
};
```

Use a separate MMKV instance with `encryptionKey` for auth stores.

---

## partialize — Always Exclude Functions

Functions cannot be serialized. Persisting them causes silent errors:

```typescript
persist(
  (set) => ({
    token: null,
    theme: "light",
    setToken: (token) => set({ token }),
    setTheme: (theme) => set({ theme }),
  }),
  {
    name: "app-storage",
    storage: createJSONStorage(() => zustandStorage),
    // Only persist data fields — exclude action functions
    partialize: (state) => ({
      token: state.token,
      theme: state.theme,
    }),
  },
);
```

---

## Hydration Timing

### Synchronous hydration (MMKV)

Since MMKV is synchronous, Zustand stores hydrate instantly on import — no async gap.
This is the primary advantage over AsyncStorage.

### When hydration may still lag

In some Expo/SSR environments, hydration may not complete before the first render. Use
`skipHydration` for explicit control:

```typescript
persist(
  (set) => ({
    /* ... */
  }),
  {
    name: "app-storage",
    storage: createJSONStorage(() => zustandStorage),
    skipHydration: true,
  },
);

// In root layout component
useEffect(() => {
  useAppStore.persist.rehydrate();
}, []);
```

### hasHydrated pattern

For screens that must not render stale initial state:

```typescript
interface AppState {
  _hasHydrated: boolean
  token: string | null
  // ...
}

const useAppStore = create<AppState>()(
  persist(
    (set) => ({
      _hasHydrated: false,
      token: null,
    }),
    {
      name: 'app-storage',
      storage: createJSONStorage(() => zustandStorage),
      onRehydrateStorage: () => () => {
        useAppStore.setState({ _hasHydrated: true })
      },
    },
  ),
)

// Usage in root layout
function RootLayout() {
  const hasHydrated = useAppStore((s) => s._hasHydrated)
  if (!hasHydrated) return <SplashScreen />
  return <Slot />
}
```

---

## Shallow Merge Gotcha

Zustand's default persist merge is `{ ...currentState, ...persistedState }`. This is a
shallow merge — nested objects in persisted state completely overwrite current state
nested objects (including runtime-only keys).

If your state has nested objects with runtime-only keys, use a custom deep merge:

```typescript
persist(
  (set) => ({
    /* ... */
  }),
  {
    name: "app-storage",
    storage: createJSONStorage(() => zustandStorage),
    merge: (persistedState, currentState) => ({
      ...currentState,
      ...(persistedState as Partial<AppState>),
      // Explicitly merge nested objects if needed
      settings: {
        ...currentState.settings,
        ...(persistedState as any)?.settings,
      },
    }),
  },
);
```

---

## Vanilla Store for Non-React Access

Use `createStore` (from `zustand/vanilla`) when the store must be read from XState actions
or other non-React contexts:

```typescript
import { createStore } from "zustand/vanilla";
import { persist, createJSONStorage } from "zustand/middleware";

export const authStore = createStore<AuthState>()(
  persist(
    (set) => ({
      token: null,
      setToken: (token) => set({ token }),
    }),
    {
      name: "auth",
      storage: createJSONStorage(() => zustandStorage),
    },
  ),
);

// Non-reactive: authStore.getState(), authStore.setState()
// Reactive in React: useStore(authStore, (s) => s.token)
```

---

## Common Pitfalls

1. **`getString` returns `undefined`** — Zustand's `StateStorage.getItem` must return
   `null` for missing keys. Always coerce: `mmkv.getString(name) ?? null`.

2. **Creating MMKV instance inside a component** — each call creates a new native
   instance. Declare at module level and reuse.

3. **One MMKV instance per store** — do not share encrypted and unencrypted instances
   across stores. Create separate instances for different security levels.

4. **Not using `partialize`** — persisting action functions causes serialization errors.
   Always exclude functions and derived state.

5. **Persisting ephemeral UI state** — loading spinners, modal open/close, transient
   selections should not be persisted. Only persist data that must survive app restart.
