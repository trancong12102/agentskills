---
name: react-native-advanced
description: "React Native and Expo patterns for navigation, data fetching lifecycle, infinite scroll lists, form handling, state persistence, authentication routing, gesture-driven animations, bottom sheets, push notifications, and OTA updates. Use when building Expo/React Native apps that need data prefetching without route loaders, auth guard routing, infinite scroll with FlashList, gesture-driven animations, or native platform integration (push notifications, OTA updates, MMKV persistence). Do not use for React web apps."
---

# React Native Advanced: Expo + TanStack/XState/Zustand Ecosystem

React Native and Expo patterns for apps built with the TanStack ecosystem, XState, and
Zustand. This skill extends `react-advanced` (core cross-platform patterns). Read that skill
first for React Query, XState, Zustand, Zod, TanStack Form, and TanStack Table conventions.

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
| CSS animations   | Reanimated         | UI-thread worklets, CSS transitions (v4)          |
| DOM events       | Gesture Handler    | Gesture composition API, UI-thread callbacks      |
| `<img>`          | expo-image         | SDWebImage/Glide, blurhash, disk caching          |
| Web Push API     | expo-notifications | FCM/APNs, channels, background tasks              |
| Service Workers  | expo-updates       | OTA updates, staged rollout, emergency rollback   |

Cross-platform libraries (identical API on web and RN):
React Query, XState, Zustand, Zod, TanStack Form, TanStack Table

---

## Required Setup

Two integrations are **mandatory** — without them, React Query's auto-refetch and offline
handling do not work in React Native:

- **focusManager** — wire `AppState` to `focusManager.setFocused()` so `refetchOnWindowFocus`
  works when the app returns to foreground. Without this, the default is silently ignored.
- **onlineManager** — wire `NetInfo` to `onlineManager.setOnline()` so queries pause/resume
  on connectivity changes.

Call both hooks once in the root `_layout.tsx` inside `QueryClientProvider`.

See `references/react-query-rn.md` for full implementation.

---

## Data Fetching Without Route Loaders

Expo Router has **no native route loaders** (data loaders are web-only/alpha). The pattern
is: prefetch on user interaction, consume in the destination screen.

Rules:

- Fire `prefetchQuery` without `await` before `router.push` — awaiting makes navigation slow.
- Use `useFocusEffect` (from `expo-router`) to invalidate stale data when a screen regains
  focus. `useEffect` does not re-run when navigating back because screens stay mounted.
- Use `invalidateQueries` (respects `staleTime`) instead of `refetch()` (always re-fetches).

See `references/react-query-rn.md` for prefetch and `useRefreshOnFocus` patterns.

---

## Navigation + Auth

### Stack.Protected (Expo Router v5+, recommended)

```typescript
// app/_layout.tsx
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

When `session` flips, Expo Router automatically redirects and cleans history. For complex
auth flows (token check, refresh, error recovery), use XState to manage auth state and
derive a boolean for the `guard` prop.

See `references/expo-router.md` for the XState auth variant and Zustand auth store patterns.

---

## Lists: FlashList + React Query

FlashList replaces TanStack Virtual for RN — it uses native view recycling instead of
DOM-based absolute positioning.

Rules:

- The `!isFetchingNextPage` guard in `onEndReached` is essential — FlashList can fire
  `onEndReached` multiple times in quick succession, causing duplicate fetches.
- Memoize the flattened `items` array with `useMemo` — `data.pages.flatMap()` creates a new
  reference each render.
- Stabilize `handleEndReached` with `useCallback`.
- `getNextPageParam` must return `undefined` (not `null`) to signal no next page.

See `references/lists.md` for the full infinite scroll pattern, FlashList v2 changes,
and performance tuning.

---

## TanStack Form in RN

TanStack Form is headless — no DOM dependency, no adapter needed. Key differences from web:

- `TextInput` uses `onChangeText` (string directly) instead of `onChange` (event object).
  Wire `field.handleChange` directly to `onChangeText`.
- For numeric fields, convert at the call site:
  `onChangeText={(val) => field.handleChange(val === '' ? null : Number(val))}`.
- Wrap forms in `ScrollView` with `keyboardShouldPersistTaps="handled"` — otherwise the
  first tap on Submit dismisses the keyboard instead of firing the press.

---

## Zustand Persist with MMKV

MMKV is synchronous and runs on the native thread — no async hydration gap. Create the
MMKV instance at module level (never inside a component). The `StateStorage` adapter must
return `null` for missing keys: `mmkv.getString(name) ?? null`.

For sensitive data, use an encrypted MMKV instance with `encryptionKey`. For the hybrid
pattern (hardware-backed key + encrypted MMKV), see `references/expo-essentials.md`.

See `references/zustand-rn.md` for the full adapter, store creation, encrypted storage,
hydration timing, and vanilla store patterns.

---

## Animations & Gestures

Reanimated runs animations on the **UI thread** via worklets. Gesture Handler routes touch
events to the same thread. Reanimated 4 adds CSS-style declarative transitions.

### Critical Rules

- **Replace entire values**: `sv.value = { x: 50 }` not `sv.value.x = 50` (breaks reactivity)
- **No destructuring**: `const { x } = sv.value` creates a plain number, not reactive
- **No reads during render**: shared value reads are side effects, violate Rules of React
- **Shared values don't re-render**: if you need React to respond, maintain separate state
- **GestureHandlerRootView** must wrap the app root with `style={{ flex: 1 }}`
- **Android modals** need their own `GestureHandlerRootView` (outside native root view)

### Threading Summary

Shared values live on the UI thread. Reading `.value` on the JS thread is a blocking bridge
call — never do it in hot paths. Gesture callbacks and `useAnimatedStyle` run on the UI
thread. Use `runOnJS` (Reanimated 3) or `scheduleOnRN` (Reanimated 4) to call JS functions.

### Declarative vs Imperative

Use **CSS transitions** (Reanimated 4) for state-driven style changes (toggle colors,
opacity, dimensions). Use **layout animations** (`entering`/`exiting` props) for mount/unmount.
Use **worklets + shared values** for gesture-driven animations and imperative chains.

See `references/animations.md` for detailed patterns, Reanimated 4 API changes, gesture
composition, and CSS transitions.

---

## Bottom Sheets

`@lodev09/react-native-true-sheet` — a native bottom sheet backed by
`UISheetPresentationController` (iOS) and `BottomSheetDialog` (Android). No Reanimated
dependency. New Architecture only.

### Key Rules

- **Imperative control only** — use `ref.present()`/`dismiss()`/`resize()`, not state props.
- **Max 3 detents**, sorted smallest to largest. Use `'auto'` for content-fitting.
- **Never combine `scrollable` with `'auto'` detent** — they conflict. Use fixed detents.
- **Never use `flex: 1` on sheet content** — collapses to zero height. Use `flexGrow` or
  fixed height.
- **Always `dismiss()` before unmounting** — the native sheet outlives the React component.
- **Standard `TextInput` works** — no special keyboard component needed (native handling).
- **Standard `ScrollView`/`FlashList` works** with `scrollable` + `nestedScrollEnabled` — auto-detected in v3 (up to 2 levels deep).

See `references/bottom-sheet.md` for full patterns, platform differences, and Expo Router
integration.

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

Structure: `app/` contains route files with `_layout.tsx` per segment. Route groups `(auth)/`,
`(app)/`, `(tabs)/` organize without URL impact. Feature code lives outside `app/`:
`queries/` (queryOptions objects, not hooks), `mutations/`, `machines/` (pure TS, no React
imports), `stores/` (Zustand + MMKV persist), `hooks/`, `components/`.

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

6. **FlashList v2 requires New Architecture** — v2 is a ground-up rewrite for Fabric.
   `estimatedItemSize` is deprecated (ignored). `overrideItemLayout` only supports `span`.

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

13. **Unmounting `TrueSheet` while open** — the native sheet does NOT dismiss automatically.
    Always call `dismiss()` before removing from tree.

14. **`auto` detent with `scrollable` in TrueSheet** — auto-sizing and scroll pinning
    conflict. Use fixed detents when `scrollable={true}`.

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
| `references/animations.md`      | Reanimated, Gesture Handler patterns and gotchas          |
| `references/bottom-sheet.md`    | react-native-true-sheet setup, detents, Expo Router       |
| `references/notifications.md`   | expo-notifications permissions, listeners, background     |
| `references/expo-essentials.md` | expo-image, expo-secure-store, expo-haptics, expo-updates |
