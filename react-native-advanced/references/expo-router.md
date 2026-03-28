# Expo Router — Navigation, Auth & Search Params

For core React patterns (state management, useEffect ban, React Query, XState, Zustand),
see `react-advanced`. This file covers Expo Router-specific navigation patterns.

---

## Typed Routes

Enable in `app.json`:

```json
{
  "expo": {
    "experiments": { "typedRoutes": true }
  }
}
```

Types are auto-generated from the file system into `.expo/types/router.d.ts`. `Link`'s
`href` and `router.push()` become type-safe — invalid routes cause TS errors.

### Comparison with TanStack Router

| Feature                    | Expo Router (typedRoutes)   | TanStack Router              |
| -------------------------- | --------------------------- | ---------------------------- |
| Route type generation      | Auto from file system       | Explicit `createFileRoute()` |
| Dynamic param typing       | `useLocalSearchParams<>`    | `Route.useParams()`          |
| Search param validation    | Manual (Zod in component)   | `validateSearch: zodSchema`  |
| Coercion (string → number) | No — all params are strings | Yes via `z.coerce.number()`  |

---

## Layouts, Groups, Modals

### Route groups — organize without URL impact

Parenthesized directories (`(auth)`, `(app)`) create layout groups without adding to the URL
path. Each group gets its own `_layout.tsx`. Nest `(tabs)` inside an `(app)` group to
separate auth screens from the main tab navigator.

### Modals

Define modal screens in the root `Stack` with `presentation` in `options`.

`presentation` options: `'modal'`, `'formSheet'`, `'transparentModal'`,
`'containedTransparentModal'`. Use `sheetAllowedDetents` (array of 0-1 fractions) and
`sheetGrabberVisible` for iOS sheet-style modals.

---

## Navigation Patterns

### useLocalSearchParams vs useGlobalSearchParams

- `useLocalSearchParams` — tracks only current screen's params. **Always prefer this.**
- `useGlobalSearchParams` — re-renders on every navigation event in the entire navigator.
  Use only when you genuinely need cross-screen param access.

---

## Search Params Validation with Zod

Expo Router has **no built-in `validateSearch`**. All params arrive as `string | string[]`.
Create a wrapper hook:

```typescript
import { useLocalSearchParams } from "expo-router";
import { z } from "zod";

const productSchema = z.object({
  id: z.string().min(1),
  tab: z.enum(["details", "reviews", "related"]).catch("details"),
  page: z
    .string()
    .transform(Number)
    .pipe(z.number().int().positive())
    .optional(),
});

export function useProductParams() {
  const raw = useLocalSearchParams<{
    id: string;
    tab?: string;
    page?: string;
  }>();
  return productSchema.parse(raw);
}
```

- All params are strings — use `z.string().transform()` to coerce
- Use `.catch()` for fallback values on invalid input
- This is the main gap vs TanStack Router's route-level `validateSearch`

---

## Authentication Patterns

### Pattern 1: Stack.Protected (v5+, recommended)

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

When `session` flips, Expo Router automatically redirects and cleans history. Deep links
to protected screens fail gracefully when unauthenticated.

### Pattern 2: Redirect in layout (v4 compatible)

```typescript
// app/(app)/_layout.tsx
export default function AppLayout() {
  const { session, isLoading } = useSession()
  if (isLoading) return <LoadingScreen />
  if (!session) return <Redirect href="/sign-in" />
  return <Stack />
}
```

### With XState for complex auth flows

XState handles async auth (token check, refresh, error recovery). Derive a boolean for
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

export default function RootLayout() {
  return (
    <AuthContext.Provider>
      <RootNavigator />
    </AuthContext.Provider>
  )
}
```

### With Zustand auth store

```typescript
// stores/authStore.ts
export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      session: null,
      setSession: (token) => set({ session: token }),
    }),
    {
      name: "auth-storage",
      storage: createJSONStorage(() => zustandStorage), // MMKV adapter
    },
  ),
);
```

---

## Deep Linking

Expo Router **automatically enables deep linking for all routes**. Configure scheme in
`app.json`:

```json
{ "expo": { "scheme": "myapp" } }
```

`myapp://user/alice` and `myapp://modal?ref=push` work automatically. For nested routes,
set `unstable_settings.initialRouteName` to ensure the back stack is populated correctly
on deep link entry.

---

## Common Pitfalls

1. **`useEffect` for redirect in layout** — use `Stack.Protected` (v5) or `<Redirect>`
   declaratively. Imperative `router.replace()` in effects causes race conditions.

2. **Zustand + Expo Router `useEffect` on web** — Zustand state updates in effects may
   not trigger redirect on web platform. Use declarative `Stack.Protected` instead.

3. **Missing `unstable_settings.initialRouteName`** — deep links into nested screens
   won't have a proper back stack without this.

4. **`renderRouter` pathname assertion lag** — in tests, pathname state can lag after
   redirects. Always wrap post-navigation assertions with `waitFor`.
