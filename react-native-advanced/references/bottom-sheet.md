# Bottom Sheet: @gorhom/bottom-sheet

Patterns and pitfalls for `@gorhom/bottom-sheet` v5 — the standard bottom sheet library for
Expo apps. Built on Reanimated 3 and Gesture Handler 2.

---

## Setup

Install `@gorhom/bottom-sheet` + `react-native-reanimated` + `react-native-gesture-handler`.
Wrap root with `GestureHandlerRootView` (with `flex: 1`) and `BottomSheetModalProvider`
(only needed for `BottomSheetModal`).

---

## BottomSheet vs BottomSheetModal

| Feature            | BottomSheet                          | BottomSheetModal                         |
| ------------------ | ------------------------------------ | ---------------------------------------- |
| Rendering          | Inline (in parent's tree)            | Portal (renders on top of everything)    |
| Use when           | Always-visible or route-based sheets | Imperative presentation (user-triggered) |
| Dismiss            | `index={-1}` or snap to -1           | `dismiss()` method                       |
| `onClose` callback | Not available — use `onChange`       | Available                                |
| Multiple sheets    | Manual z-ordering                    | Stack managed by provider                |

### Common Mistake: BottomSheet in a Non-Full-Screen Parent

`BottomSheet` renders relative to its parent. If the parent does not fill the screen, the
sheet looks wrong (clipped or mispositioned). Use `BottomSheetModal` when the sheet must
appear on top of all content regardless of position in the tree.

---

## Snap Points and Dynamic Sizing

```typescript
<BottomSheet snapPoints={['25%', '50%', '90%']}>
  <BottomSheetView>
    <Text>Content</Text>
  </BottomSheetView>
</BottomSheet>
```

- Snap points must be sorted **bottom to top**: `['25%', '50%', '90%']`.
- `enableDynamicSizing` defaults to `true`. When enabled without explicit `snapPoints`,
  the sheet auto-sizes to content.
- **Do not provide both `snapPoints` and `enableDynamicSizing={true}`** — the library injects
  an extra snap point for content size and sorts the array, shifting all indices reported by
  `onChange`. Set `enableDynamicSizing={false}` when using fixed snap points.

---

## Keyboard Handling — The Biggest Source of Issues

### Use BottomSheetTextInput, Not TextInput

```typescript
import { BottomSheetTextInput } from '@gorhom/bottom-sheet'

<BottomSheet>
  <BottomSheetTextInput
    placeholder="Type here..."
    style={{ padding: 12, borderWidth: 1 }}
  />
</BottomSheet>
```

The library cannot intercept keyboard events from a plain `TextInput`. This also applies
to third-party components that use `TextInput` internally (autocomplete, dropdowns) — their
inner inputs must be replaced.

### Keyboard Behavior Props

```typescript
<BottomSheet
  snapPoints={['50%', '90%']}
  keyboardBehavior="fillParent"   // 'interactive' (default) | 'extend' | 'fillParent'
  keyboardBlurBehavior="restore"  // 'none' (default) | 'restore'
  enableDynamicSizing={false}     // required with fixed snapPoints + keyboard
>
  <BottomSheetTextInput />
</BottomSheet>
```

- `keyboardBehavior="interactive"` (default): offsets sheet up by keyboard height.
- `keyboardBlurBehavior` defaults to `'none'` — **the sheet stays elevated after keyboard
  closes**. Most developers expect automatic restoration. Set `"restore"` explicitly.
- `android_keyboardInputMode` modifies the entire Activity window — test against full screen
  layout, not just the bottom sheet.

---

## Scrollable Content

Always use the library's scrollable wrappers — never bare React Native equivalents:

| Use                      | Instead of                                  |
| ------------------------ | ------------------------------------------- |
| `BottomSheetScrollView`  | `ScrollView`                                |
| `BottomSheetFlatList`    | `FlatList`                                  |
| `BottomSheetSectionList` | `SectionList`                               |
| `BottomSheetView`        | `View` (for non-scrollable dynamic content) |

These wrappers ensure gesture interactions between sheet dragging and content scrolling
work correctly. A plain `ScrollView` steals pan gestures from the sheet.

---

## Non-Obvious Prop Defaults

| Prop                   | Default | Surprise                                                  |
| ---------------------- | ------- | --------------------------------------------------------- |
| `backdropComponent`    | `null`  | No backdrop — background is fully interactive             |
| `enablePanDownToClose` | `false` | Users cannot swipe to close unless enabled                |
| `animateOnMount`       | `true`  | Sheet animates open even with `index={-1}`                |
| `keyboardBlurBehavior` | `none`  | Sheet stays elevated after keyboard closes                |
| `enableDynamicSizing`  | `true`  | Extra snap point injected when combined with `snapPoints` |

### Adding a Backdrop

```typescript
import { BottomSheetBackdrop } from '@gorhom/bottom-sheet'

const renderBackdrop = useCallback(
  (props: BottomSheetBackdropProps) => (
    <BottomSheetBackdrop
      {...props}
      disappearsOnIndex={0}
      appearsOnIndex={1}
      pressBehavior="close"
    />
  ),
  []
)

<BottomSheet backdropComponent={renderBackdrop}>
```

### Detecting Close

`onClose` does **not** exist on the base `BottomSheet`. Use `onChange`:

```typescript
<BottomSheet
  onChange={(index) => {
    if (index === -1) {
      // Sheet closed
    }
  }}
>
```

`onClose` is only available on `BottomSheetModal`.

---

## Expo Router Integration

### Route-Based Bottom Sheets

For sheets tied to navigation (detail panels, settings), use `BottomSheet` + Expo Router:

```typescript
// app/detail/[id].tsx
export default function DetailSheet() {
  const { id } = useLocalSearchParams()
  const router = useRouter()

  return (
    <BottomSheet
      enablePanDownToClose
      onChange={(index) => {
        if (index === -1) router.back()
      }}
    >
      <DetailContent id={id} />
    </BottomSheet>
  )
}
```

This gives you deep-linking and clean state without managing `present()`/`dismiss()`.

### BottomSheetModal with Tabs

`BottomSheetModalProvider` must sit **above** the `Tabs` component. Placing it inside a tab
screen or using it with `modal`/`transparentModal` presentation causes the portal to inject
beneath the screen — invisible and non-interactive.

---

## Common Pitfalls

1. **Using `TextInput` instead of `BottomSheetTextInput`** — keyboard avoidance breaks
   silently. No error, the keyboard just covers the input.

2. **Combining `snapPoints` with `enableDynamicSizing`** — extra snap point injected,
   all indices shift. Set `enableDynamicSizing={false}` with fixed snap points.

3. **Missing backdrop** — defaults to `null`. Users expect a dimmed background and tap
   outside to close. Must be added manually.

4. **`enablePanDownToClose` not set** — defaults to `false`. Users can't swipe to close.

5. **`keyboardBlurBehavior="none"` (default)** — sheet stays elevated after keyboard
   closes. Set to `"restore"`.

6. **SDK upgrade breakage** — bottom-sheet has had regressions with multiple Expo SDK
   versions. Always verify behavior after SDK upgrades in a development build.

7. **v5 + Reanimated 4** — v5 targets Reanimated 3. Check GitHub issues for Reanimated 4
   compatibility before upgrading.
