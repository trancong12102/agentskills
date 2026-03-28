# Expo Essentials: Image, Secure Store, Haptics, Updates

Patterns and pitfalls for commonly-used Expo SDK APIs that most apps need.

---

## expo-image

expo-image uses SDWebImage/Glide with built-in caching — always use it over RN's built-in `Image`.

### FlashList Integration — recyclingKey

Without `recyclingKey`, recycled cells flash the previous item's image:

```typescript
<FlashList
  data={items}
  renderItem={({ item }) => (
    <Image
      source={{ uri: item.imageUrl }}
      recyclingKey={item.id}
      style={{ width: 100, height: 100 }}
      contentFit="cover"
    />
  )}
  estimatedItemSize={120}
/>
```

Known issues with `recyclingKey`:

- On iOS, blurhash placeholders render at wrong size in recycled cells.
- With `transition` prop, the crossfade happens between old and new images instead of from
  blank. Set `transition={0}` when using `recyclingKey` if this causes visual glitches.

### Placeholder Alignment

`placeholderContentFit` defaults to `'scale-down'`, not the same as `contentFit` (default
`'cover'`). This mismatch causes a visible scaling jump as the placeholder transitions to
the loaded image. Set `placeholderContentFit` to match `contentFit`:

```typescript
<Image
  source={{ uri: url }}
  placeholder={{ blurhash }}
  contentFit="cover"
  placeholderContentFit="cover"
/>
```

### Caching Patterns

Use `Image.prefetch()` for critical assets, `Image.clearDiskCache()` on logout. Memory cache
evicts aggressively — use `'disk'` or `'memory-disk'` for persistence.

### Blurhash vs Thumbhash

| Feature       | Blurhash                             | Thumbhash            |
| ------------- | ------------------------------------ | -------------------- |
| Aspect ratio  | Always 1:1 square                    | Encodes aspect ratio |
| Configuration | `componentX/Y` (1-9)                 | None needed          |
| Transparency  | No                                   | Yes                  |
| Gotcha        | Misrenders when contentFit != 'fill' | No known issues      |

---

## expo-secure-store

### Size Limit

~2048 bytes per value (iOS Keychain practical limit). Exceeding this throws, not truncates.

### Hybrid Pattern with MMKV

Use expo-secure-store for **encryption keys**, use encrypted MMKV for **data**:

```typescript
import * as SecureStore from "expo-secure-store";
import * as Crypto from "expo-crypto";
import { createMMKV } from "react-native-mmkv";

// Generate and persist encryption key in hardware-backed storage
let encryptionKey = SecureStore.getItem("mmkv-enc-key");
if (!encryptionKey) {
  encryptionKey = Crypto.randomUUID();
  SecureStore.setItem("mmkv-enc-key", encryptionKey);
}

// Use MMKV with that key for sessions, large objects
const secureStorage = createMMKV({ id: "secure-session", encryptionKey });
```

This avoids the 2048-byte limit while keeping secrets in hardware-backed storage.

### iOS Keychain Persistence

iOS keychain data **persists across app uninstalls** if reinstalled with the same bundle ID.
Android does not. Clear secure store on first launch for consistent behavior:

```typescript
const isFirstLaunch = !SecureStore.getItem("app-installed");
if (isFirstLaunch) {
  await SecureStore.deleteItemAsync("auth-token");
  SecureStore.setItem("app-installed", "true");
}
```

### Biometric Authentication

Keys with `requireAuthentication: true` become **permanently unreadable** when biometrics
change (fingerprint added/removed). On Android, throws `KeyPermanentlyInvalidatedException`
with no recovery except clearing app data.

```typescript
try {
  const token = await SecureStore.getItemAsync("auth-token", {
    requireAuthentication: true,
  });
} catch {
  // Biometrics changed — key permanently invalidated
  await SecureStore.deleteItemAsync("auth-token");
  // Must re-authenticate via password
}
```

`requireAuthentication` is not supported in Expo Go. The synchronous `getItem()` with
biometrics blocks the JS thread — prefer `getItemAsync()`.

---

## expo-haptics

Three APIs: `impactAsync(style)` for physical collisions, `notificationAsync(type)` for
semantic outcomes, `selectionAsync()` for picker/slider ticks.

### Semantic Intensity Mapping

| Intensity | Use for                                   |
| --------- | ----------------------------------------- |
| Light     | Hover, selection scrub, scroll tick       |
| Medium    | Button tap, toggle, checkbox              |
| Heavy     | Destructive confirm, payment, form submit |
| Success   | Completion, save, send                    |
| Error     | Validation failure, network error         |

### When Haptics Are Silent (iOS)

- Low Power Mode enabled
- User disabled in Settings > Sounds & Haptics
- Camera recording (physical stabilization)
- Dictation active

**Never rely on haptics as the sole feedback** — always pair with visual or audio feedback.

### Throttling

Rapid consecutive triggers degrade the haptic engine. Add 100-150ms cooldown for
interactions that fire repeatedly (sliders, scroll pickers).

---

## expo-updates (OTA)

### Standard Pattern

Standard flow: `checkForUpdateAsync()` -> `fetchUpdateAsync()` -> `reloadAsync()`. Always
guard with `if (__DEV__) return` and handle `isRollBackToEmbedded`.

### Reactive Hook Pattern

Use `Updates.useUpdates()` hook for reactive `isUpdatePending`/`isUpdateAvailable` state.

Do not put logic after `reloadAsync()` — execution after it resolves is not guaranteed.

### checkForUpdateAsync Freezes UI

On Android with slow/absent network, `checkForUpdateAsync()` can block UI for 10s to 3+
minutes. Mitigations:

- Run in `useEffect` after first render, not during startup
- Wrap with `Promise.race()` for manual timeout
- Use `checkAutomatically: 'WIFI_ONLY'` to skip on cellular

### Emergency Launch Detection

```typescript
const { isEmergencyLaunch, emergencyLaunchReason } = Updates;
if (isEmergencyLaunch) {
  // OTA update caused a crash — rolled back to embedded bundle
  // Log to error tracking immediately
  Sentry.captureMessage(`Emergency launch: ${emergencyLaunchReason}`);
}
```

Always instrument this — it's your signal that a production OTA caused a crash.

### Runtime Version Policy

Use `appVersion` in production (derives from `version` in app.json). The `fingerprint`
policy is experimental. Use `@expo/fingerprint` as a CI detection tool to decide whether
a change needs a new build or can ship as OTA — but keep `appVersion` as the actual
runtime version strategy.

### Staged Rollout

```bash
eas update --channel production --rollout-percentage 10   # 10% first
eas update --channel production --rollout-percentage 100  # promote
eas update:rollback --channel production                  # rollback
```

Keep a log of update group IDs. You cannot run `eas update:rollback` without a group ID.
