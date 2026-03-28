---
name: react-native-advanced
description: "React Native and Expo patterns for navigation, data fetching lifecycle, infinite scroll lists, form handling, state persistence, authentication routing, gesture-driven animations, bottom sheets, push notifications, and OTA updates. Use when building Expo/React Native apps that need screen-level data prefetching, auth guards with protected routes, infinite scroll feeds, native form input handling, offline-capable state persistence, platform-specific setup (focus/online managers), fluid animations and gesture interactions, modal bottom sheets, push notification flows, or over-the-air update strategies. Do not use for React web apps."
---

# React Native Advanced: Expo + TanStack/XState/Zustand Ecosystem

React Native and Expo patterns for apps built with the TanStack ecosystem, XState, and
Zustand. This skill extends `react-advanced` (core cross-platform patterns). Read that skill
first for React Query, XState, Zustand, Zod, TanStack Form, and TanStack Table conventions.

## Table of Contents

1. [RN Architecture](#rn-architecture)
2. [Required Setup](#required-setup)
3. [Data Fetching Without Route Loaders](#data-fetching-without-route-loaders)
4. [Navigation + Auth](#navigation--auth)
5. [Lists: FlashList + React Query](#lists-flashlist--react-query)
6. [TanStack Form in RN](#tanstack-form-in-rn)
7. [Zustand Persist with MMKV](#zustand-persist-with-mmkv)
8. [Animations & Gestures](#animations--gestures)
9. [Bottom Sheets](#bottom-sheets)
10. [Push Notifications](#push-notifications)
11. [OTA Updates](#ota-updates)
12. [File Organization](#file-organization)
13. [Common Pitfalls](#common-pitfalls)
14. [Reference Files](#reference-files)

---

## RN Architecture

Libraries map differently in React Native compared to web:

| Web Library      | RN Equivalent      | Key Difference                                    |
| ---------------- | ------------------ | ------------------------------------------------- |
| TanStack Router  | Expo Router        | No route loaders on native, file-based navigation |
| TanStack Start   | —                  | No SSR/server functions on native                 |
| TanStack Virtual | FlashList          | Native view recycling, not DOM virtualization     |
| localStorage     | MMKV               | Synchronous, native-thread, 30x faster            |
| window events    | AppState/NetInfo   | Manual wiring required for focus/online managers  |
| CSS animations   | Reanimated         | UI-thread worklets, shared values                 |
| DOM events       | Gesture Handler    | Gesture composition API, UI-thread callbacks      |
| Framer Motion    | Moti               | Declarative layer over Reanimated                 |
| `<img>`          | expo-image         | SDWebImage/Glide, blurhash, disk caching          |
| Web Push API     | expo-notifications | FCM/APNs, channels, background tasks              |
| Service Workers  | expo-updates       | OTA updates, staged rollout, emergency rollback   |

Cross-platform libraries (identical API on web and RN):
React Query, XState, Zustand, Zod, TanStack Form, TanStack Table

---

## Required Setup

These two integrations are **mandatory** — without them, React Query's auto-refetch and
offline handling do not work in React Native.

### focusManager — refetch when app returns to foreground

```typescript
// hooks/useAppState.ts
import { useEffect } from "react";
import { AppState, Platform } from "react-native";
import type { AppStateStatus } from "react-native";
import { focusManager } from "@tanstack/react-query";

function onAppStateChange(status: AppStateStatus) {
  if (Platform.OS !== "web") {
    focusManager.setFocused(status === "active");
  }
}

export function useAppState() {
  useEffect(() => {
    const sub = AppState.addEventListener("change", onAppStateChange);
    return () => sub.remove();
  }, []);
}
```

### onlineManager — pause/resume queries based on network

```typescript
// hooks/useOnlineManager.ts
import { useEffect } from "react";
import NetInfo from "@react-native-community/netinfo";
import { onlineManager } from "@tanstack/react-query";

export function useOnlineManager() {
  useEffect(() => {
    return NetInfo.addEventListener((state) => {
      onlineManager.setOnline(!!state.isConnected);
    });
  }, []);
}
```

Call both hooks once in the root layout:

```typescript
// app/_layout.tsx
export default function RootLayout() {
  useAppState()
  useOnlineManager()
  return (
    <QueryClientProvider client={queryClient}>
      <Slot />
    </QueryClientProvider>
  )
}
```

---

## Data Fetching Without Route Loaders

Expo Router has **no native route loaders** (data loaders are web-only/alpha). The pattern
is: prefetch on user interaction, consume in the destination screen.

### Prefetch on press (don't await — keep navigation instant)

```typescript
function PostListItem({ id }: { id: string }) {
  const queryClient = useQueryClient()
  const router = useRouter()

  return (
    <Pressable
      onPress={() => {
        queryClient.prefetchQuery(postQueryOptions(id)) // fire-and-forget
        router.push(`/posts/${id}`)
      }}
    >
      <Text>{title}</Text>
    </Pressable>
  )
}
```

### Refetch when screen regains focus

Screens stay mounted in a native stack. `useEffect` does not re-run when navigating back.
Use `useFocusEffect` to invalidate stale data:

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

Use `invalidateQueries` (respects `staleTime`) instead of `refetch()` (always re-fetches).

---

## Navigation + Auth

### Stack.Protected (Expo Router v5+, recommended)

```typescript
// app/_layout.tsx
import { Stack } from 'expo-router'
import { useAuthStore } from '@/stores/authStore'

export default function RootLayout() {
  const session = useAuthStore((s) => s.session)

  return (
    <Stack>
      <Stack.Protected guard={!!session}>
        <Stack.Screen name="(tabs)" />
        <Stack.Screen name="modal" options={{ presentation: 'modal' }} />
      </Stack.Protected>
      <Stack.Protected guard={!session}>
        <Stack.Screen name="sign-in" />
      </Stack.Protected>
    </Stack>
  )
}
```

When `session` flips, Expo Router automatically redirects and cleans history.

### With XState for complex auth flows

XState manages async auth (token check, refresh, error recovery). Derive a boolean for
`Stack.Protected`:

```typescript
const AuthContext = createActorContext(authMachine)

function RootNavigator() {
  const session = AuthContext.useSelector((s) => s.context.session)
  const isChecking = AuthContext.useSelector((s) => s.matches('checking'))

  if (isChecking) return <SplashScreen />

  return (
    <Stack>
      <Stack.Protected guard={!!session}>
        <Stack.Screen name="(app)" />
      </Stack.Protected>
      <Stack.Protected guard={!session}>
        <Stack.Screen name="sign-in" />
      </Stack.Protected>
    </Stack>
  )
}
```

---

## Lists: FlashList + React Query

FlashList replaces TanStack Virtual for RN — it uses native view recycling instead of
DOM-based absolute positioning.

### Infinite scroll pattern

```typescript
function PostList() {
  const { data, fetchNextPage, hasNextPage, isFetchingNextPage, refetch, isRefetching } =
    useInfiniteQuery({
      queryKey: ['posts'],
      queryFn: ({ pageParam }) => fetchPosts(pageParam),
      initialPageParam: 0,
      getNextPageParam: (lastPage) => lastPage.nextCursor ?? undefined,
    })

  const items = useMemo(() => data?.pages.flatMap((p) => p.items) ?? [], [data])

  const handleEndReached = useCallback(() => {
    if (hasNextPage && !isFetchingNextPage) fetchNextPage()
  }, [hasNextPage, isFetchingNextPage, fetchNextPage])

  return (
    <FlashList
      data={items}
      renderItem={({ item }) => <PostCard post={item} />}
      keyExtractor={(item) => item.id}
      estimatedItemSize={120}
      onEndReached={handleEndReached}
      onEndReachedThreshold={0.3}
      ListFooterComponent={isFetchingNextPage ? <ActivityIndicator /> : null}
      refreshControl={<RefreshControl refreshing={isRefetching} onRefresh={refetch} />}
    />
  )
}
```

The `!isFetchingNextPage` guard in `handleEndReached` is essential — FlashList can fire
`onEndReached` multiple times in quick succession, causing duplicate fetches.

---

## TanStack Form in RN

TanStack Form is headless — no DOM dependency, no adapter needed. The key difference is
`TextInput` uses `onChangeText` (string directly) instead of `onChange` (event object).

```typescript
<form.Field name="username">
  {(field) => (
    <TextInput
      value={field.state.value}
      onChangeText={field.handleChange}   // string directly — no event extraction
      onBlur={field.handleBlur}           // triggers isTouched + onBlur validators
      autoCapitalize="none"
    />
  )}
</form.Field>
```

For numeric fields, convert at the call site:

```typescript
<TextInput
  keyboardType="numeric"
  onChangeText={(val) => field.handleChange(val === '' ? null : Number(val))}
  value={String(field.state.value ?? '')}
/>
```

Wrap forms in `ScrollView` with `keyboardShouldPersistTaps="handled"` — otherwise the
first tap on Submit dismisses the keyboard instead of firing the press.

---

## Zustand Persist with MMKV

MMKV is synchronous and runs on the native thread — no async hydration gap.

```typescript
import { MMKV } from "react-native-mmkv";
import { StateStorage, createJSONStorage } from "zustand/middleware";

const mmkv = new MMKV(); // create at module level — never inside a component

const zustandStorage: StateStorage = {
  setItem: (name, value) => mmkv.set(name, value),
  getItem: (name) => mmkv.getString(name) ?? null, // must return null, not undefined
  removeItem: (name) => mmkv.delete(name),
};

export const useAppStore = create<AppState>()(
  persist(
    (set) => ({
      /* state + actions */
    }),
    {
      name: "app-storage",
      storage: createJSONStorage(() => zustandStorage),
      partialize: (state) => ({ token: state.token, theme: state.theme }),
    },
  ),
);
```

For sensitive data, use an encrypted MMKV instance with `encryptionKey`. For the hybrid
pattern (hardware-backed key + encrypted MMKV), see `references/expo-essentials.md`.

---

## Animations & Gestures

Reanimated runs animations on the **UI thread** via worklets. Gesture Handler routes touch
events to the same thread. Moti provides a declarative API on top of Reanimated.

### Threading Model

Shared values live on the UI thread. Reading `.value` on the JS thread is a blocking bridge
call — never do it in hot paths. Writing is instant.

```typescript
const x = useSharedValue(0);

// Gesture callback — runs on UI thread (worklet)
const pan = Gesture.Pan()
  .onChange((e) => {
    "worklet";
    x.value += e.changeX; // instant, no bridge
  })
  .onEnd(() => {
    "worklet";
    x.value = withSpring(0);
    runOnJS(onDragEnd)(); // call JS functions via runOnJS
  });

// Animated style — also runs on UI thread
const style = useAnimatedStyle(() => ({
  transform: [{ translateX: x.value }],
}));
```

### Critical Rules

- **Replace entire values**: `sv.value = { x: 50 }` not `sv.value.x = 50` (breaks reactivity)
- **No destructuring**: `const { x } = sv.value` creates a plain number, not reactive
- **No reads during render**: shared value reads are side effects, violate Rules of React
- **Shared values don't re-render**: if you need React to respond, maintain separate state
- **GestureHandlerRootView** must wrap the app root with `style={{ flex: 1 }}`
- **Android modals** need their own `GestureHandlerRootView` (outside native root view)

### When to Use Moti vs Raw Reanimated

Use **Moti** for entrance/exit animations, state-driven transitions, loading skeletons.
Use **raw Reanimated** for gesture-driven animations, imperative chains, shared element
transitions. Mixing is valid — `MotiView` wraps `Animated.View`.

For detailed patterns, see `references/animations.md`.

---

## Bottom Sheets

`@gorhom/bottom-sheet` is the standard. Built on Reanimated + Gesture Handler.

### Minimal Example

```typescript
import BottomSheet from '@gorhom/bottom-sheet'

function MySheet() {
  return (
    <BottomSheet snapPoints={['25%', '50%']} enableDynamicSizing={false}>
      <BottomSheetView>
        <Text>Content</Text>
      </BottomSheetView>
    </BottomSheet>
  )
}
```

### Key Rules

- **Always use `BottomSheetTextInput`** instead of `TextInput` inside sheets — keyboard
  handling breaks silently otherwise.
- **Always use library scrollables** (`BottomSheetScrollView`, `BottomSheetFlatList`) — plain
  `ScrollView`/`FlatList` steal gestures.
- **Set `enableDynamicSizing={false}`** when using fixed `snapPoints` — default `true`
  injects an extra snap point, shifting all indices.
- **Set `keyboardBlurBehavior="restore"`** — default `"none"` leaves the sheet elevated
  after keyboard closes.
- **Add backdrop manually** — defaults to `null` (no backdrop, background interactive).
- **Set `enablePanDownToClose`** — defaults to `false` (users can't swipe to close).

For full patterns and Expo Router integration, see `references/bottom-sheet.md`.

---

## Push Notifications

expo-notifications handles FCM (Android) and APNs (iOS) through Expo's push service.

### SDK 53 Breaking Changes

- **Android push does not work in Expo Go** — requires development build.
- **Config plugin must be explicit** in `app.json` `plugins` array.

### Key Pattern

```typescript
// Must configure or foreground notifications are silently suppressed
Notifications.setNotificationHandler({
  handleNotification: async () => ({
    shouldShowBanner: true,
    shouldShowList: true,
    shouldPlaySound: true,
    shouldSetBadge: false,
  }),
});
```

### Notification Rules

- **Android channels must exist before requesting permissions** on Android 13+.
- **Channels are immutable** — cannot change importance/vibration after creation.
- **Poll receipts in production** — tickets only confirm Expo received the request.
- **Background tasks must be defined at module scope** (not inside components).

For permission handling, listeners, and background tasks, see `references/notifications.md`.

---

## OTA Updates

expo-updates enables over-the-air JavaScript bundle updates without app store releases.

### Update Check Pattern

```typescript
import * as Updates from "expo-updates";

// Run after first render, not during startup (can freeze UI on slow network)
const check = await Updates.checkForUpdateAsync();
if (check.isAvailable) {
  await Updates.fetchUpdateAsync();
  await Updates.reloadAsync();
}
```

### Emergency Launch Detection

```typescript
if (Updates.isEmergencyLaunch) {
  // OTA caused a crash — app rolled back to embedded bundle
  // Log immediately to error tracking
}
```

Always instrument this — it's your production crash signal for OTA updates.

For staged rollout, runtime versions, and reactive patterns, see `references/expo-essentials.md`.

---

## File Organization

```text
app/
  _layout.tsx              # Root layout — providers, auth guard
  (auth)/
    _layout.tsx            # Auth group layout
    sign-in.tsx
  (app)/
    _layout.tsx            # App group layout (tabs)
    (tabs)/
      _layout.tsx          # Tab navigator
      index.tsx
      profile.tsx
    [id].tsx               # Dynamic route
    modal.tsx              # Modal screen
queries/                   # queryOptions definitions
mutations/                 # useMutation wrappers
machines/                  # XState machine definitions
stores/                    # Zustand stores (MMKV persist)
hooks/                     # useAppState, useOnlineManager, useRefreshOnFocus
components/                # Shared components
```

Key conventions:

- Route groups `(name)/` organize without URL impact
- `_layout.tsx` defines the navigator for each segment
- Machine definitions are pure TypeScript — no React imports
- `queries/` files export `queryOptions` objects, not hooks

---

## Common Pitfalls

1. **Missing focusManager/onlineManager setup** — `refetchOnWindowFocus` and offline
   pausing do nothing without manual AppState and NetInfo wiring.

2. **Awaiting `prefetchQuery` before `router.push`** — makes navigation feel slow. Fire
   prefetch without await, let React Query cache serve the destination screen.

3. **Using `useGlobalSearchParams` instead of `useLocalSearchParams`** — global re-renders
   on every navigation event. Always prefer local.

4. **`useEffect` for screen focus refetch** — `useEffect` doesn't re-run when navigating
   back (screens stay mounted). Use `useFocusEffect` from `expo-router`.

5. **`getNextPageParam` returning `null`** — must return `undefined` to signal no next page.
   Coerce API nulls: `lastPage.nextCursor ?? undefined`.

6. **FlashList without `estimatedItemSize`** — required prop. Use the median item height,
   not the mean (outliers skew the mean).

7. **MMKV `getString` returns `undefined`** — Zustand's `StateStorage.getItem` must return
   `null` for missing keys. Always coerce: `?? null`.

8. **Creating MMKV instance inside a component** — creates new instances each render.
   Declare at module level.

9. **`keyboardShouldPersistTaps` not set on form ScrollView** — first tap dismisses
   keyboard instead of pressing Submit button.

10. **`retry: 3` (default) on mobile** — with flaky connections, 3 retries with exponential
    backoff can take 30+ seconds. Consider `retry: 1` for time-sensitive UI.

11. **Mutating shared value properties** — `sv.value.x = 50` breaks reactivity. Must
    replace entire value: `sv.value = { x: 50, y: 0 }`.

12. **GestureHandlerRootView missing `flex: 1`** — wrapper has zero height. Gestures appear
    to not work at all.

13. **`TextInput` inside bottom sheet** — keyboard handling breaks silently. Must use
    `BottomSheetTextInput`.

14. **`enableDynamicSizing` with fixed `snapPoints`** — extra snap point injected, all
    indices shift. Set `enableDynamicSizing={false}`.

15. **Missing `setNotificationHandler`** — all foreground notifications silently suppressed.
    No error.

16. **`checkForUpdateAsync` on slow network** — can freeze Android UI for minutes. Run
    after first render with a manual timeout.

17. **expo-image `recyclingKey` in FlashList** — without it, recycled cells flash the
    previous item's image.

18. **expo-secure-store biometric invalidation** — keys with `requireAuthentication` become
    permanently unreadable when biometrics change. No recovery except clearing app data.

---

## Reference Files

| File                            | When to read                                              |
| ------------------------------- | --------------------------------------------------------- |
| `references/react-query-rn.md`  | Focus/online managers, cache persistence, prefetching     |
| `references/expo-router.md`     | Typed routes, layouts, modals, auth, search params        |
| `references/lists.md`           | FlashList + React Query, infinite scroll, performance     |
| `references/zustand-rn.md`      | MMKV persist adapter, encryption, hydration patterns      |
| `references/testing-rn.md`      | RNTL, testing Query/Router/Form/XState, MSW in RN         |
| `references/animations.md`      | Reanimated, Gesture Handler, Moti patterns and gotchas    |
| `references/bottom-sheet.md`    | @gorhom/bottom-sheet setup, keyboard, Expo Router         |
| `references/notifications.md`   | expo-notifications permissions, listeners, background     |
| `references/expo-essentials.md` | expo-image, expo-secure-store, expo-haptics, expo-updates |
