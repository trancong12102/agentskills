# Bottom Sheet: react-native-true-sheet

Patterns and pitfalls for `@lodev09/react-native-true-sheet` v3.x — a native bottom sheet
library backed by `UISheetPresentationController` (iOS) and `BottomSheetDialog` (Android).
No Reanimated dependency required. New Architecture only.

---

## Mental Model: Native + Imperative

TrueSheet differs fundamentally from JS-animated bottom sheets:

| Concept          | JS-animated sheets            | react-native-true-sheet                 |
| ---------------- | ----------------------------- | --------------------------------------- |
| Control          | Declarative state/index props | Imperative `ref.present()`/`dismiss()`  |
| Sizes            | `snapPoints` string array     | `detents` — max 3, fraction or `'auto'` |
| Implementation   | JS gestures + Reanimated      | Native platform sheet APIs              |
| Keyboard         | Manual (special TextInput)    | Automatic (native keyboard avoidance)   |
| Unmount behavior | Component controls visibility | Native sheet outlives component         |

The native backing gives true platform behavior (system corner radius, keyboard avoidance,
hardware-accelerated drag) but means you **cannot control the sheet via state alone** — the
ref is mandatory.

---

## Detents

`detents` is `SheetDetent[]` — **maximum 3 entries**, sorted smallest to largest.

```typescript
type SheetDetent = "auto" | number; // number is 0–1 fraction of screen height

<TrueSheet detents={[0.5, 1]}>        {/* half and full screen */}
<TrueSheet detents={["auto"]}>         {/* fit to content height */}
<TrueSheet detents={["auto", 0.5, 1]}> {/* auto, half, full */}
```

### `auto` Detent Rules

- Content **must have a defined height** — `flex: 1` inside an `auto` sheet produces zero
  height. Use fixed `height`, `flexGrow`, or intrinsic sizing.
- **`auto` + `scrollable` is incompatible** — the auto-sizing mechanism and scroll pinning
  conflict. Use fixed detents with `scrollable`.
- Dynamic content changes after the sheet opens may not recalculate `auto` height correctly.
  Load data before calling `present()`.

---

## Ref Methods

All methods return `Promise<void>`:

```typescript
const sheet = useRef<TrueSheet>(null);

await sheet.current?.present(); // present at detent index 0
await sheet.current?.present(1); // present at second detent
await sheet.current?.resize(1); // change detent while open
await sheet.current?.dismiss(); // dismiss this sheet + all sheets stacked above
await sheet.current?.dismissStack(); // dismiss only sheets above, keep this one open
```

- **`present()` on an already-presented sheet** logs a warning (v3.8+). Use `resize()` for
  programmatic size changes.
- **`dismiss()` is a cascade** (v3.8+) — removes the entire stack including the current sheet.
  Use `dismissStack()` to dismiss only children.

### Global Control via `name` Prop

```typescript
<TrueSheet name="settings-sheet">...</TrueSheet>

// From anywhere in the app:
TrueSheet.present("settings-sheet");
TrueSheet.dismiss("settings-sheet");
TrueSheet.dismissAll();
```

---

## ScrollView / FlashList Integration

ScrollViews are **auto-detected** in v3 — no ref wiring needed. Set `scrollable` and use
standard RN scroll components directly:

```typescript
<TrueSheet scrollable detents={[0.5, 1]}>
  <FlashList nestedScrollEnabled data={items} renderItem={renderItem} />
</TrueSheet>
```

`scrollable={true}` coordinates scroll vs. drag: scrolling down from top drags the sheet to
a smaller detent; scrolling up expands it. `nestedScrollEnabled` is required for the scroll
detection to work correctly, especially on Android.

### Gotchas

- **Never combine `scrollable` with `'auto'` detent** — use fixed detents only.
- **Auto-detection depth limit** — TrueSheet finds the scroll view up to 2 levels deep. If
  FlashList is wrapped in more than one container View, `scrollable` will have no effect.
- **Don't conditionally mount/unmount ScrollViews inside sheets on Android** — breaks scroll
  handling. Toggle visibility with `display: 'none'` or opacity instead.
- **Pull-to-refresh on Android** fails when ScrollView content doesn't fill the sheet height.

---

## Keyboard Handling

TrueSheet handles the keyboard **natively on both platforms** — no `KeyboardAvoidingView` or
special TextInput components needed. Standard `TextInput` works directly.

On iOS, the sheet auto-expands to the largest detent when a TextInput is focused. On Android,
the native dialog handles `adjustResize` automatically.

### Known Limitations

- **Switching between TextInputs in a ScrollView (iOS)** — tapping from one input to another
  dismisses the keyboard instead of switching focus. Workaround: use a flat View container
  (no ScrollView) for forms.
- **Multi-detent keyboard expansion (iOS)** — when at a smaller detent, focusing an input
  expands to the largest detent, potentially mispositions the input. For keyboard-heavy forms,
  start at the largest detent.

---

## Header and Footer

```typescript
<TrueSheet
  header={<MyHeader />}
  footer={<MyStickyFooter />}
>
  <MyScrollableContent />
</TrueSheet>
```

- `header` is pinned above scrollable content — does not scroll.
- `footer` floats at the bottom, lifts with the keyboard.
- Avoid conditional footer rendering — empty space can remain after removal on iOS. Render
  with hidden/zero-height content instead.

---

## Expo Router Integration

### Sheet Navigator

```typescript
import { createTrueSheetNavigator } from "@lodev09/react-native-true-sheet/navigation";
import { withLayoutContext } from "expo-router";

const { Navigator } = createTrueSheetNavigator();
const Sheet = withLayoutContext(Navigator);
```

The first `<Sheet.Screen>` is the base content; subsequent screens are sheets. Screen options
accept all TrueSheet props.

### Deep Link Caveat

On cold-start deep links, `initialDetentIndex` may silently fail because the native view
hasn't attached yet. Use `useFocusEffect` with a state flag:

```typescript
const [didPresent, setDidPresent] = useState(false);

useFocusEffect(
  useCallback(() => {
    if (!didPresent) sheet.current?.present();
  }, [didPresent]),
);

<TrueSheet onDidPresent={() => setDidPresent(true)} />;
```

---

## Reanimated Integration (Optional)

For gesture-driven backdrop opacity or position-linked animations:

```typescript
import { ReanimatedTrueSheet } from "@lodev09/react-native-true-sheet/reanimated";

const { animatedPosition, animatedIndex } = useReanimatedTrueSheet("my-sheet");

// Drive a custom backdrop
const backdropStyle = useAnimatedStyle(() => ({
  opacity: animatedPosition.value * 0.5,
}));
```

Requires `react-native-reanimated >= 4.0.0` and wrapping the app root with
`ReanimatedTrueSheetProvider`.

---

## Platform Differences

| Behavior              | iOS                                             | Android                                                     |
| --------------------- | ----------------------------------------------- | ----------------------------------------------------------- |
| `backgroundBlur`      | Full support (20+ styles)                       | Not supported                                               |
| `detached` mode       | Supported (floating card)                       | Not supported                                               |
| Liquid Glass          | Auto on iOS 26+ (opt-out via `backgroundColor`) | N/A                                                         |
| `dismissible={false}` | Disables swipe-to-dismiss                       | Must handle `onBackPress` for hw back button                |
| Keyboard              | Auto-expands to largest detent                  | Adjusts window                                              |
| GestureHandler        | Works normally                                  | Use `flexGrow: 1` not `flex: 1` on `GestureHandlerRootView` |

---

## Key Props — Non-Obvious Defaults

| Prop               | Default     | Gotcha                                                               |
| ------------------ | ----------- | -------------------------------------------------------------------- |
| `dimmed`           | `true`      | Background is NOT interactive while dimmed                           |
| `dismissible`      | `true`      | Drag-to-dismiss enabled by default (opposite of gorhom)              |
| `draggable`        | `true`      | `false` hides grabber and disables drag entirely                     |
| `insetAdjustment`  | `automatic` | Subtracts bottom safe area from detent heights                       |
| `cornerRadius`     | system      | `undefined` = platform default, `0` = sharp                          |
| `maxContentHeight` | —           | Caps `auto` and `1` detent height (renamed from `maxHeight` in v3.9) |

---

## Common Pitfalls

1. **Unmounting `TrueSheet` while open** — the native sheet does NOT dismiss. Always call
   `dismiss()` before removing from tree. This is a `wontfix` design choice.

2. **`auto` detent with `scrollable`** — auto-sizing and scroll pinning conflict. Use fixed
   detents when scrollable.

3. **`flex: 1` on sheet content** — the sheet has no intrinsic height. Content collapses to
   zero. Use fixed height, `flexGrow`, or intrinsic sizing.

4. **Closing RN `<Modal>` with TrueSheet on top (iOS)** — causes blank screen. RN only
   dismisses the topmost native controller. Dismiss the sheet first.

5. **Calling `present()` on an already-open sheet** — no-op with warning (v3.8+). Use
   `resize()` to change detent.

6. **`GestureHandlerRootView` inside sheet on Android** — use `flexGrow: 1`, not `flex: 1`,
   to avoid layout issues.

7. **SDK/RN version requirement** — v3.x requires New Architecture (Fabric). Paper is not
   supported. Expo SDK 52+ with New Architecture enabled.
