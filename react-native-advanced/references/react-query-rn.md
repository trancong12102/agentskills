# React Query in React Native — Platform-Specific Patterns

For core React Query patterns (queryOptions, mutations, cache management, infinite queries),
see `react-advanced`'s `references/react-query.md`. This file covers RN-specific setup and
differences from web.

---

## Required Platform Wiring

### focusManager — refetch on app foreground

See SKILL.md 'Required Setup' for the full implementation. Key insight: without this,
`refetchOnWindowFocus: true` (default) does nothing in RN.

### onlineManager — pause queries when offline

See SKILL.md 'Required Setup'. Two options: `@react-native-community/netinfo` or
`expo-network`. Without this, queries don't pause offline.

---

## Prefetching Without Route Loaders

Expo Router has no native route loaders. The standard approach is `prefetchQuery` on user
interaction:

```typescript
function PostListItem({ id }: { id: string }) {
  const queryClient = useQueryClient()
  const router = useRouter()

  return (
    <Pressable onPress={() => {
      queryClient.prefetchQuery(postQueryOptions(id))  // fire-and-forget
      router.push(`/posts/${id}`)
    }}>
      <Text>{title}</Text>
    </Pressable>
  )
}
```

Key rules:

- **Do not** `await prefetchQuery` before `router.push` — navigation feels slow
- `prefetchQuery` never throws — errors are silently swallowed
- The `staleTime` on `prefetchQuery` only applies to the prefetch call; the destination's
  `useQuery` uses its own `staleTime`
- For layout-level data (auth, user profile), use `ensureQueryData` — it returns data from
  cache if fresh, otherwise fetches

### useFocusEffect — refetch when returning to a screen

Screens stay mounted in a native stack. `useEffect` does not re-run on back navigation:

```typescript
import { useFocusEffect } from "expo-router";

export function useRefreshOnFocus(queryKey: unknown[]) {
  const queryClient = useQueryClient();
  const firstRender = useRef(true);

  useFocusEffect(
    useCallback(() => {
      if (firstRender.current) {
        firstRender.current = false;
        return;
      }
      queryClient.invalidateQueries({ queryKey });
    }, [queryClient, queryKey]),
  );
}
```

Use `invalidateQueries` (respects `staleTime`) not `refetch()` (always re-fetches).

Import `useFocusEffect` from `'expo-router'`, not `'@react-navigation/native'` — Expo
Router's version waits for navigation state to load before firing.

---

## Suspense in React Native

`<Suspense>` and `useSuspenseQuery` work in RN — Suspense is a React feature, not a
browser feature. Same limitations as web:

- No `enabled` option — cannot conditionally skip
- Multiple `useSuspenseQuery` in one component run sequentially (waterfall)
- Changing queryKey re-triggers fallback — wrap navigation with `startTransition`
- Every `<Suspense>` tree needs a matching `<ErrorBoundary>` + `QueryErrorResetBoundary`

**Expo Router gotcha:** File-based layouts create implicit Suspense boundaries. If you add
`useSuspenseQuery` without an explicit `<Suspense>`, the nearest parent layout catches
it — which may be the root layout, causing the entire app to show a fallback.

---

## Cache Persistence

### Approach A: Whole-cache (`PersistQueryClientProvider`)

Persists the entire cache as a serialized blob. Simple but has performance cost for large
datasets.

```typescript
import AsyncStorage from '@react-native-async-storage/async-storage'
import { PersistQueryClientProvider } from '@tanstack/react-query-persist-client'
import { createAsyncStoragePersister } from '@tanstack/query-async-storage-persister'

const queryClient = new QueryClient({
  defaultOptions: { queries: { gcTime: 1000 * 60 * 60 * 24 } },
})

const persister = createAsyncStoragePersister({ storage: AsyncStorage })

// In root layout
<PersistQueryClientProvider
  client={queryClient}
  persistOptions={{ persister, maxAge: 1000 * 60 * 60 * 24 }}
>
  <Slot />
</PersistQueryClientProvider>
```

**Critical: `gcTime` must be >= `maxAge`.** If `gcTime` is 5 minutes (default) but
`maxAge` is 24 hours, data is garbage-collected before it can hydrate.

MMKV adapter for whole-cache:

```typescript
const storage = createMMKV();
const mmkvAdapter = {
  setItem: (key: string, value: string) => storage.set(key, value),
  getItem: (key: string) => storage.getString(key) ?? null,
  removeItem: (key: string) => storage.remove(key),
};
const persister = createAsyncStoragePersister({ storage: mmkvAdapter });
```

### Approach B: Per-query (`experimental_createQueryPersister`)

Persists each query individually. Better for apps with large datasets — queries are lazily
restored when first used.

```typescript
import { experimental_createQueryPersister } from "@tanstack/query-persist-client-core";

const persister = experimental_createQueryPersister({
  storage: AsyncStorage,
  maxAge: 1000 * 60 * 60 * 12,
});

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      gcTime: 1000 * 30, // can be short — restored from storage on demand
      persister: persister.persisterFn,
    },
  },
});
```

Key behaviors:

- `networkMode` defaults to `'offlineFirst'` when a persister is used
- `gcTime` can be low — queries restore from storage on demand
- Can be applied per-query instead of globally

**Prefer per-query persistence** for apps with meaningful data volume.

---

## Key Differences from Web

| Area                 | Web                              | React Native                                   |
| -------------------- | -------------------------------- | ---------------------------------------------- |
| `focusManager`       | Auto-wired via window events     | Must wire AppState manually                    |
| `onlineManager`      | Auto-wired via navigator.onLine  | Must wire NetInfo/expo-network manually        |
| Route loaders        | TanStack Router loaders          | No equivalent — prefetch on interaction        |
| Screen focus refetch | `refetchOnWindowFocus` works     | `useFocusEffect` + `invalidateQueries`         |
| DevTools             | `@tanstack/react-query-devtools` | Flipper plugin (v5 compatible)                 |
| Cache persistence    | localStorage sync persister      | AsyncStorage/MMKV async or per-query persister |
| `retry` default      | 3 (fine for broadband)           | Consider `retry: 1` for mobile networks        |

---

## Common Pitfalls

1. **`PersistQueryClientProvider` hydration timing** — the provider renders children
   immediately but queries remain idle until restoration completes. Use `useIsRestoring()`
   from `@tanstack/react-query-persist-client` to gate UI that depends on hydrated data.

2. **Hermes serialization cost** — very large cache payloads (hundreds of KB) cause jank
   on the JS thread during persist/hydrate. Use per-query persistence for large datasets.

3. **`refetchOnWindowFocus` with `staleTime: 0` on mobile** — every app foreground
   triggers refetch for all mounted queries. Set a meaningful global `staleTime`.
