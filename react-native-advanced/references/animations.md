# Animations & Gestures: Reanimated + Gesture Handler + Moti

Patterns and pitfalls for building fluid, gesture-driven animations in Expo apps. Covers
Reanimated 4 (UI-thread animations), Gesture Handler 2 (gesture composition), and Moti
(declarative animation layer).

---

## Reanimated Threading Model

Reanimated runs animations on the **UI thread** via worklets. The JS thread is asynchronous
— reading `.value` on the JS thread blocks until the value is fetched from the UI thread.

```text
JS Thread          UI Thread
  |                   |
  |-- sv.value ------>|  (write: instant)
  |<-- sv.value ------|  (read: async bridge, blocks JS)
  |                   |
  |  useAnimatedStyle runs here (UI thread)
  |  gesture callbacks run here (UI thread)
```

### Shared Value Rules

```typescript
// WRONG — mutating properties breaks reactivity
sv.value.x = 50;
sv.value.push(1000);

// CORRECT — replace the entire value
sv.value = { x: 50, y: 0 };
sv.value = [...sv.value, 1000];
// or use .modify() for partial updates
sv.modify((v) => {
  v.x = 50;
  return v;
});
```

Common mistakes:

- **Forgetting `.value`** — reading `sv` instead of `sv.value`. No error, animation silently
  does nothing.
- **Destructuring** — `const { x } = sv.value` creates a plain number, not reactive.
- **Reading during render** — shared value reads are side effects, violate Rules of React.
  Read only in `useAnimatedStyle`, gesture callbacks, or `useEffect`.
- **Expecting re-renders** — shared value changes do NOT trigger React re-renders. If UI
  needs to react (show/hide non-animated elements), maintain separate React state.

### useAnimatedStyle Best Practices

```typescript
const animatedStyle = useAnimatedStyle(() => ({
  transform: [{ translateX: x.value }],
  opacity: interpolate(progress.value, [0, 1], [0, 1], Extrapolation.CLAMP),
}))

// Keep static styles separate — only animated properties in useAnimatedStyle
const styles = StyleSheet.create({ container: { flex: 1, borderRadius: 8 } })

<Animated.View style={[styles.container, animatedStyle]} />
```

- Never mutate shared values inside the callback — undefined behavior, can cause infinite loops.
- Define animation builders (`FadeIn`, custom configs) outside the component or with
  `useMemo` — new instances every render defeat the optimization.

### useDerivedValue for Computed Values

Memoize computed values derived from other shared values instead of recalculating the same
interpolation in multiple `useAnimatedStyle` hooks:

```typescript
const opacity = useDerivedValue(() =>
  interpolate(progress.value, [0, 1], [0, 1], Extrapolation.CLAMP),
);
```

### Layout Animations

```typescript
<Animated.View entering={FadeIn.duration(300)} exiting={FadeOut.duration(200)}>
  <Text>Animates in and out</Text>
</Animated.View>
```

Pitfalls:

- `entering`/`exiting` props are registered once at mount — changing them after mount does nothing.
- Removing a non-animated parent triggers children's exiting animations, but the parent
  does not wait — content disappears abruptly. Wrap animated children in a stable parent.
- **FlashList + layout animations**: pass `skipEnteringExitingAnimations` to prevent all
  items from animating in on initial render.
- Spring animations (`.springify()`) do not work on web.

### Reanimated 4 API Changes

| Reanimated 3                      | Reanimated 4                              |
| --------------------------------- | ----------------------------------------- |
| `runOnJS(fn)(args)`               | `scheduleOnRN(fn, ...args)` (variadic)    |
| `runOnUI(fn)(args)`               | `scheduleOnUI(fn, ...args)` (variadic)    |
| `useAnimatedGestureHandler`       | Gesture Handler 2 `Gesture` API (removed) |
| `useScrollViewOffset`             | `useScrollOffset`                         |
| Babel plugin: `reanimated/plugin` | `react-native-worklets/plugin`            |

Reanimated 4 is **New Architecture only**. Expo SDK 53+ enables New Architecture by default.
The Babel plugin is auto-configured by `babel-preset-expo` — do not add it manually.

Import `scheduleOnRN`/`scheduleOnUI` from `react-native-worklets`, not `react-native-reanimated`:

```typescript
import { scheduleOnRN } from "react-native-worklets";
```

`runOnJS`/`runOnUI` remain as deprecated aliases in `react-native-reanimated`.

### `.get()` / `.set()` — React Compiler Compatible

Reanimated 4 adds `.get()` and `.set()` as alternatives to `.value` that work with the
React Compiler (which may break the `.value` proxy):

```typescript
const x = useSharedValue(0);
x.get(); // read (React Compiler-safe)
x.set((v) => v + 1); // write with updater (React Compiler-safe)
```

Use `.get()`/`.set()` in new code when adopting the React Compiler.

### `withSpring` — Threshold Change in v4

`restDisplacementThreshold` and `restSpeedThreshold` are replaced by a single
`energyThreshold` (default `6e-9`). Code passing the old threshold options silently uses
wrong defaults.

### Performance

Reanimated handles ~100 simultaneously animated components on low-end Android, ~500 on iOS.
Never use `useState` for gesture-driven position values — it causes re-renders and jank.

---

## Gesture Handler 2

### New API (Always Use This)

The old API (`<PanGestureHandler>`, `useAnimatedGestureHandler`) is deprecated. Use the
`Gesture` builder API with `GestureDetector`:

Use `Gesture.Pan()` with `onChange`/`onEnd` callbacks (worklets) to update shared values,
`useAnimatedStyle` for the animated view, and wrap the target with `GestureDetector`.

### GestureHandlerRootView — Critical Setup

Every app must wrap root with `GestureHandlerRootView`:

```typescript
<GestureHandlerRootView style={{ flex: 1 }}>
  {/* entire app */}
</GestureHandlerRootView>
```

- `flex: 1` is **required** — without it, the wrapper has zero height.
- Must be at the **true root** — gesture relations only work within the same root.
- **Android modals**: gestures break because modals render outside the root view.
  Wrap modal content with its own `GestureHandlerRootView`.

### Calling JS Functions from Gestures

Gesture callbacks are worklets (UI thread). To call JS-thread functions:

```typescript
const pan = Gesture.Pan().onEnd(() => {
  "worklet";
  runOnJS(setIsOpen)(false); // Reanimated 3
  // scheduleOnRN(setIsOpen, false) // Reanimated 4 (variadic, not array)
});
```

### Gesture Composition

```typescript
// Race: first to activate wins, cancels the rest
const composed = Gesture.Race(pan, longPress);

// Simultaneous: all activate together (gallery: pan + pinch + rotation)
const gallery = Gesture.Simultaneous(drag, pinch, rotate);

// Exclusive: priority order — double-tap must fail before single-tap fires
const taps = Gesture.Exclusive(doubleTap, singleTap);
```

- Never reuse a gesture instance across two `GestureDetector`s — this throws.
- For **pan inside ScrollView**: use `simultaneousWithExternalGesture` for both to work,
  or `blocksExternalGesture` for the pan to take priority.

---

## Reanimated 4 — CSS Transitions (Declarative)

Reanimated 4 adds CSS-style declarative transitions that replace the need for libraries
like Moti. State-driven style changes animate automatically:

```typescript
<Animated.View
  style={{
    backgroundColor: isActive ? "blue" : "gray",
    opacity: isVisible ? 1 : 0,
    transitionProperty: "backgroundColor, opacity",
    transitionDuration: 300,
  }}
/>
```

### When to Use Which API

| API                                      | Use for                                        |
| ---------------------------------------- | ---------------------------------------------- |
| CSS transitions (`transition*`)          | State-driven style changes (toggle, show/hide) |
| Layout animations (`entering`/`exiting`) | Mount/unmount animations                       |
| `useAnimatedStyle` + shared values       | Gesture-driven, scroll-driven, imperative      |
| `withSpring`/`withTiming`                | Programmatic animations in worklets            |

### CSS Transitions — Key Props

- `transitionProperty` — comma-separated list of animated properties
- `transitionDuration` — milliseconds (number, not string)
- `transitionDelay` — milliseconds
- `transitionTimingFunction` — `'ease-in'`, `'ease-out'`, `'linear'`, or cubic-bezier

CSS transitions are **Reanimated 4+ only** (New Architecture required). For Reanimated 3
projects, use `useAnimatedStyle` with `withTiming`/`withSpring` for the same effect.
