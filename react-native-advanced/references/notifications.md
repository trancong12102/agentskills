# Push Notifications: expo-notifications

Setup, permission handling, and common pitfalls for push notifications in Expo apps.

---

## SDK 53 Breaking Changes

1. **Push notifications no longer work in Expo Go on Android.** A development build is
   required. iOS Expo Go still works via EAS auto-configuration.
2. **Config plugin must be explicitly listed** in `app.json` `plugins` array. Previously
   implicit — now required or iOS push breaks silently.

```json
{
  "plugins": [
    ["expo-notifications", { "enableBackgroundRemoteNotifications": true }]
  ]
}
```

---

## Permission + Token Registration

The registration function must handle these non-obvious requirements:

- **Check `Device.isDevice`** — simulators/emulators cannot receive push tokens, exit early
- **Create Android channel BEFORE requesting permissions** — Android 13+ requires this order
  or permissions silently fail
- **Check existing status before requesting** — calling `requestPermissionsAsync()` when
  already granted is fine, but re-prompting after denial does nothing on iOS (OS ignores it)
- **Use `Constants.expoConfig.extra.eas.projectId`** for the token request — falls back to
  `Constants.easConfig.projectId`

### Android Channel Rules

- Channel must exist **before** requesting permissions on Android 13+.
- You **cannot modify** a channel after creation (Android OS limitation) — only name and
  description can be updated. To change importance, vibration, or sound, delete and recreate
  with a **new** channel ID. Users lose their customizations on deletion.

### iOS Provisional Notifications

Request quiet delivery without the upfront permission prompt:

```typescript
await Notifications.requestPermissionsAsync({
  ios: {
    allowAlert: true,
    allowBadge: true,
    allowSound: true,
    allowProvisional: true,
  },
});

const { ios } = await Notifications.getPermissionsAsync();
// CRITICAL: check ios.status, not the root status field
// Root status collapses PROVISIONAL -> 'granted', which is misleading
if (ios?.status === Notifications.IosAuthorizationStatus.PROVISIONAL) {
  // quiet delivery only — show in-app prompt to upgrade
}
```

---

## Notification Listeners

### Foreground Display — Must Configure

Without `setNotificationHandler`, notifications are **silently suppressed** when the app is
in the foreground:

```typescript
Notifications.setNotificationHandler({
  handleNotification: async () => ({
    shouldShowBanner: true, // replaces deprecated shouldShowAlert (SDK 53+)
    shouldShowList: true,
    shouldPlaySound: true,
    shouldSetBadge: false,
  }),
});
```

### Three Listener Patterns

```typescript
// 1. Foreground: fires while app is running
Notifications.addNotificationReceivedListener((notification) => {
  // notification.request.content.data has your payload
});

// 2. Tap interaction: foreground, background, AND relaunched-from-killed
Notifications.addNotificationResponseReceivedListener((response) => {
  const data = response.notification.request.content.data;
  // response.actionIdentifier === Notifications.DEFAULT_ACTION_IDENTIFIER for normal tap
});

// 3. Killed state: check on cold start
const lastResponse = Notifications.getLastNotificationResponse(); // sync in SDK 53+
```

### Background Tasks with TaskManager

Define headless tasks in **module scope** (e.g., `index.ts`), not inside a component:

```typescript
import * as TaskManager from "expo-task-manager";
import * as Notifications from "expo-notifications";

const TASK_NAME = "BACKGROUND_NOTIFICATION_TASK";

// Must be top-level
TaskManager.defineTask(TASK_NAME, ({ data, error }) => {
  if (error) return;
  // handle notification in background
});

Notifications.registerTaskAsync(TASK_NAME);
```

**Android killed-state limitations:**

- Data-only FCM messages (no `title`/`body`) do not trigger background tasks when the app
  is killed on Android.
- Android Doze mode can suppress execution entirely.
- Use FCM Notification-type messages to guarantee delivery in killed state.

---

## Push Tickets vs Receipts

`sendPushNotificationsAsync()` returns **tickets** immediately — they confirm Expo received
the request, not that the device received it. **Receipts** become available ~15 minutes
later and expose:

- `DeviceNotRegistered` — stale token, remove from your database
- Credential failures
- Rate limiting

Always poll receipts in production with `getPushNotificationReceiptsAsync()`.

---

## Common Pitfalls

1. **Missing config plugin in SDK 53** — `expo-notifications` config plugin must be
   explicitly in `app.json` `plugins`. Without it, iOS push fails silently.

2. **Testing in Expo Go on Android** — does not work in SDK 53+. Use development builds.

3. **Foreground suppression** — without `setNotificationHandler`, all foreground
   notifications are silently dropped. No error.

4. **`shouldShowAlert` deprecated** — use `shouldShowBanner` and `shouldShowList` in SDK 53+.

5. **Android channel immutability** — cannot modify importance/vibration/sound after
   creation. Must delete and recreate with a new ID.

6. **Root `status` hides provisional** — `getPermissionsAsync().status` returns `'granted'`
   for both full and provisional permissions. Check `ios.status` for the real status.

7. **Not polling receipts** — tickets only confirm Expo received the request. Stale tokens
   and delivery failures only surface in receipts.

8. **Background task not at module scope** — defining `TaskManager.defineTask` inside a
   component means it may not be registered when the OS launches the app headlessly.
