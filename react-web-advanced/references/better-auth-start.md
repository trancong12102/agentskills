# Better Auth + TanStack Start + React Query

Integration patterns for Better Auth in TanStack Start apps. Covers the signal
architecture that causes re-render loops, and the React Query wrapper that fixes them.

---

## Why NOT `useSession()` from `better-auth/react`

Better Auth's React hooks use **nanostores** atoms bound via `useSyncExternalStore`.
Every successful API mutation toggles internal signals, causing ALL hook subscribers
to re-render synchronously. Combined with TanStack Router's `beforeLoad`:

1. User navigates → `beforeLoad` runs
2. Some mutation succeeds → `$sessionSignal` toggles
3. `useSession()` subscribers re-render → layout re-renders → `RouterProvider` re-renders
4. Router re-runs `beforeLoad` → loop

### Signal cascade on `organization.setActive()`

`setActive()` triggers **3 cascading signal flips** with `setTimeout(..., 10)` spacing:

| Signal                | Hook affected             | Delay |
| --------------------- | ------------------------- | ----- |
| `$sessionSignal`      | `useSession()`            | 10ms  |
| `$activeOrgSignal`    | `useActiveOrganization()` | 10ms  |
| `$activeMemberSignal` | `useActiveMember()`       | 10ms  |

Each flip triggers a separate re-render wave through the entire component tree. Any
mutation under `/organization/...` also toggles `$activeOrgSignal` via `atomListeners`.

### Additional refetch triggers on `useSession()`

The session atom auto-refetches on:

- Window focus (`visibilitychange`) — rate-limited to once per 5 seconds
- Tab sync (`BroadcastChannel`) — always on
- Online event (`navigator.onLine`)

These are fine for standalone apps but compound the problem in Router layouts where
re-renders propagate through the route tree.

---

## Data flow: when server function vs direct client call

Most operations should be **direct client → API calls**. The browser sends cookies
automatically — no server function or cookie forwarding needed.

Server functions are only needed for **SSR session checks** in `beforeLoad`, because
during SSR the code runs on the server where there is no browser to send cookies.

```text
SSR (beforeLoad):  server fn → getRequestHeaders() → API    ← needs server fn
Client navigation: ensureQueryData → cached, no network      ← React Query cache
Client mutation:   authClient.signIn.email() → API           ← browser sends cookies
Client fetch:      fetch("/api/projects") → API              ← browser sends cookies
```

**Rule: only use `createServerFn` for session reads in `beforeLoad`. Everything else
goes direct from the client.**

---

## Setup

### 1. Auth client: use `better-auth/client` (vanilla)

Import from `better-auth/client`, NOT `better-auth/react`. The React import brings
nanostores hooks that cause the re-render loops described above:

```typescript
// lib/auth-client.ts
import { createAuthClient } from "better-auth/client";
import { organizationClient, apiKeyClient } from "better-auth/client/plugins";

export const authClient = createAuthClient({
  baseURL: "/api/auth", // relative — goes through Vite proxy in dev
  plugins: [organizationClient(), apiKeyClient()],
});
```

### 2. Server function: only for SSR session check

This is the **only** server function needed for auth. Everything else (mutations,
data fetching) calls the API directly from the client.

```typescript
// serverFns/auth.ts
import { createServerFn } from "@tanstack/react-start";
import { getRequestHeaders } from "@tanstack/react-start/server";
import { auth } from "~/lib/auth"; // your betterAuth() instance

export const getSession = createServerFn({ method: "GET" }).handler(
  async () => {
    return auth.api.getSession({ headers: getRequestHeaders() });
  },
);
```

`getRequestHeaders()` uses `AsyncLocalStorage` — cookies from the incoming browser
request are available automatically during SSR.

If auth is a separate service (not in the same process), use the internal URL
pattern from `ssr-auth.md` instead of `auth.api.getSession`.

### 3. Server: `tanstackStartCookies` plugin (conditional)

Only needed when Better Auth is mounted **inside** the TanStack Start app via
a route handler (e.g., `routes/api/auth/$.ts`). It ensures `Set-Cookie` headers
pass through TanStack Start's server function layer correctly.

**Not needed** when the API is a separate service (browser calls it directly via
Vite proxy) — the browser and API exchange cookies without TanStack Start in the
middle. The session read server function (`getSession`) only reads headers, never
sets cookies, so it doesn't need this plugin either.

```typescript
// Only if auth is mounted inside TanStack Start:
import { tanstackStartCookies } from "better-auth/tanstack-start";

export const auth = betterAuth({
  plugins: [organization(), apiKey(), tanstackStartCookies()], // must be last
});
```

---

## Auth State via React Query

### Session query options (the one server function case)

```typescript
// queries/auth.ts
export const sessionQueryOptions = queryOptions({
  queryKey: ["auth", "session"],
  queryFn: () => getSession(), // server fn — needed for SSR in beforeLoad
  staleTime: 5 * 60 * 1000, // 5 min — prevents refetch on every navigation
  refetchOnMount: false,
  refetchOnWindowFocus: false,
});
```

### Root route: fetch once, pass via context

```typescript
// routes/__root.tsx
export const Route = createRootRouteWithContext<{
  queryClient: QueryClient;
}>()({
  beforeLoad: async ({ context: { queryClient } }) => {
    const session = await queryClient.ensureQueryData(sessionQueryOptions);
    return { session };
  },
});
```

`ensureQueryData` returns cached data instantly if within `staleTime`. No network
call on subsequent navigations. On SSR, data is dehydrated into HTML.

### `_authed` layout guard

```typescript
// routes/_authed.tsx
export const Route = createFileRoute("/_authed")({
  beforeLoad: ({ context }) => {
    if (!context.session?.user) throw redirect({ to: "/sign-in" });
    return { user: context.session.user };
  },
  component: () => <Outlet />,
});
```

Children access `user` via `Route.useRouteContext()` — stable, no re-renders.

### Session invalidation after login/logout

```typescript
// After login
await authClient.signIn.email({ email, password });
queryClient.resetQueries({ queryKey: ["auth", "session"] });
router.invalidate();

// After logout
await authClient.signOut();
queryClient.resetQueries({ queryKey: ["auth"] });
router.invalidate();
```

Use `resetQueries` (not `invalidateQueries`) — clears cache AND triggers refetch,
ensuring the route guard sees updated state immediately.

---

## Organization State via React Query

### Query options (direct client calls — no server function needed)

```typescript
// queries/org.ts — queryFn calls API directly, browser sends cookies
export const activeOrgQueryOptions = (orgId: string | null) =>
  queryOptions({
    queryKey: ["auth", "activeOrg", orgId],
    queryFn: () =>
      authClient.organization.getFullOrganization({
        query: { organizationId: orgId! },
      }),
    staleTime: 5 * 60 * 1000,
    enabled: !!orgId,
  });

export const orgListQueryOptions = queryOptions({
  queryKey: ["auth", "orgs"],
  queryFn: () => authClient.organization.list(),
  staleTime: 5 * 60 * 1000,
});
```

### Org switch handler

```typescript
async function switchOrg(prevOrgId: string, nextOrgId: string) {
  await authClient.organization.setActive({ organizationId: nextOrgId });

  // Reset session (activeOrganizationId changed)
  queryClient.resetQueries({ queryKey: ["auth", "session"] });

  // Remove old org-scoped data (prevents cross-org leakage)
  queryClient.removeQueries({ queryKey: ["org", prevOrgId] });

  // Re-run route loaders
  router.invalidate();
}
```

See `integration.md` for the query key namespacing pattern that makes `removeQueries`
safe for auth keys.

---

## Type Inference

```typescript
// Infer session type from server auth config
import type { auth } from "@api/auth";
export type Session = typeof auth.$Infer.Session;
// { user: User & { activeOrganizationId: string | null; ... }; session: ... }

// Org plugin types
export type ActiveOrg = typeof authClient.$Infer.ActiveOrganization;
export type OrgMember = typeof authClient.$Infer.Member;
```

For monorepo with shared types, use `inferAdditionalFields`:

```typescript
import { inferAdditionalFields } from "better-auth/client/plugins";

export const authClient = createAuthClient({
  plugins: [organizationClient(), inferAdditionalFields<typeof auth>()],
});
```

---

## Common Pitfalls

1. **Routing mutations through server functions** — unnecessary complexity. The
   browser sends cookies automatically on direct client → API calls. Only the SSR
   session check in `beforeLoad` needs a server function.

2. **Importing from `better-auth/react`** — brings nanostores hooks that cause
   re-render cascades with TanStack Router. Use `better-auth/client` instead.

3. **Using `useSession()` for route guards** — returns `null` during SSR, causing
   flash redirects. Use `ensureQueryData(sessionQueryOptions)` in `beforeLoad`.

4. **`organization.setActive()` without explicit query reset** — triggers 3
   cascading nanostores signal flips. Always follow with `resetQueries` +
   `removeQueries` + `router.invalidate()`.

5. **Missing `tanstackStartCookies()` when auth is mounted inside Start** — only
   needed when Better Auth route handler lives inside the TanStack Start app.
   Not needed when auth is a separate API service.
