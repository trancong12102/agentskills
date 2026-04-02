# SSR + Cookie Auth — Dev & Production Patterns

Patterns for any cookie-based auth library (Better Auth, Lucia, Auth.js, custom) running
in an SSR framework on Vite. Covers the full cookie lifecycle: browser → Vite dev server →
SSR handler → API → back.

---

## Cookie Configuration for Local Development

`Secure` cookies are **silently dropped** by browsers on `http://localhost`. The cookie is
set in the response header but the browser never stores it — no error, no warning.

```typescript
// auth.config.ts
export const authConfig = {
  advanced: {
    useSecureCookies: process.env.NODE_ENV === "production",
  },
  session: {
    cookieCache: { enabled: true, maxAge: 300 },
  },
};
```

Verify with browser devtools: `Application → Cookies`. If empty after login, cookies are
being dropped. Also check `document.cookie` in the console — empty string means no
accessible cookies were set.

---

## Vite Proxy for Same-Origin Cookies

Dashboard (port A) and API (port B) are different origins in dev. Cookies set by the API
**cannot be read by the dashboard** because different ports = different origins for cookie
`SameSite` enforcement.

Use Vite's built-in proxy so both dashboard and API share a single origin:

```typescript
// vite.config.ts
export default defineConfig({
  server: {
    proxy: {
      "/api": {
        target: "http://localhost:6781", // API server
        changeOrigin: true,
        cookieDomainRewrite: "localhost",
      },
    },
  },
});
```

The auth client should always call `/api/...` (relative path, no host). Never use an
absolute URL like `http://localhost:6781` from the client — that bypasses the proxy and
creates cross-origin cookie issues.

### When to use server functions vs direct client calls

Most calls should go **direct from client → API**. The browser sends cookies
automatically — no server function needed for mutations or client-side data fetching.

Server functions are only needed for **SSR session checks** in `beforeLoad`, because
during SSR the code runs on the server where there is no browser to send cookies.

### SSR circular fetch prevention

During SSR, `createServerFn` runs on the Vite dev server. If the server function fetches
`getRequest().url` (which points back through the proxy), it creates an infinite loop:

```text
Browser → Vite (SSR) → server function → fetch("/api/...") → Vite (proxy) → API
                                                ↑                              |
                                                └── but if URL is the SSR URL ─┘  DEADLOCK
```

**Fix:** Server-side code must bypass the proxy. Two approaches:

**Preferred: `getRequestHeaders()`** — when auth runs in the same process (monolith):

```typescript
// serverFns/auth.ts
import { createServerFn } from "@tanstack/react-start";
import { getRequestHeaders } from "@tanstack/react-start/server";

export const getSession = createServerFn({ method: "GET" }).handler(
  async () => {
    return auth.api.getSession({ headers: getRequestHeaders() });
  },
);
```

`getRequestHeaders()` uses `AsyncLocalStorage` — cookies are available automatically.
No manual forwarding needed.

**Alternative: internal URL** — when auth is a separate service:

```typescript
const API_INTERNAL = process.env.API_INTERNAL_URL ?? "http://localhost:6781";

export const getSession = createServerFn({ method: "GET" }).handler(
  async () => {
    const headers = getRequestHeaders();
    const res = await fetch(`${API_INTERNAL}/api/auth/get-session`, {
      headers: { cookie: headers.get("cookie") ?? "" },
    });
    return res.json();
  },
);
```

Key rules:

- Client-side: relative paths (`/api/...`) → goes through Vite proxy
- Server-side (SSR): `getRequestHeaders()` for same-process, internal URL for separate service
- Always forward the `cookie` header when making cross-service auth calls

---

## CORS for Credentialed Requests

When a proxy is not feasible (separate deployments, mobile app hitting API directly):

```typescript
// api/cors.ts
const corsConfig = {
  origin: (origin: string) => trustedOrigins.includes(origin),
  credentials: true, // Required for cookies
  methods: ["GET", "POST", "PUT", "DELETE"],
  allowedHeaders: ["Content-Type", "Authorization"],
};

// Auth library config
const trustedOrigins = [
  "http://localhost:6780", // Dashboard dev
  "https://dashboard.example.com",
];
```

Checklist:

- `Access-Control-Allow-Credentials: true` — without this, browser blocks cookie headers
- `Access-Control-Allow-Origin` must be an explicit origin — `*` is invalid with credentials
- `trustedOrigins` must list **every** client origin — empty array means reject everything
- `SameSite: "None"` + `Secure: true` required for cross-origin cookies (HTTPS only)

---

## Environment Variables in Vite

Three ways to wire env vars, each with different visibility:

| Method                              | Available in                              | Safe for secrets?        |
| ----------------------------------- | ----------------------------------------- | ------------------------ |
| `VITE_*` prefix in `.env`           | Client + SSR via `import.meta.env.VITE_*` | No — inlined into bundle |
| `define` in `vite.config.ts`        | Client + SSR via string replacement       | No — inlined into bundle |
| `process.env.*` in server functions | Server only (`createServerFn`)            | Yes                      |

Common traps:

- `import.meta.env.API_URL` is `undefined` — Vite only exposes vars prefixed with `VITE_`
- `define: { "import.meta.env.API_URL": ... }` uses **string replacement** — bracket
  notation `import.meta.env["API_URL"]` won't match the dot-notation define key
- Never put auth secrets (session encryption keys, DB credentials) in `VITE_*` vars

---

## Auth Session in Route Guards (SSR-Safe)

Auth reactive hooks (`useSession()`, `useActiveOrganization()`) return `null` during SSR
and during refetch windows. Using them in route guards causes flash redirects:

```typescript
// BAD — hook returns null during SSR → redirects to login even if user is authenticated
function AuthLayout() {
  const session = useSession();
  if (!session.data) return <Navigate to="/login" />;
  return <Outlet />;
}
```

Use `beforeLoad` with a server-validated session check instead:

```typescript
// queries/auth.ts
export const sessionQueryOptions = queryOptions({
  queryKey: ["auth", "session"],
  queryFn: () => getSession(), // createServerFn that reads cookie
  staleTime: 5 * 60 * 1000,   // 5 min — prevents refetch on every navigation
  refetchOnMount: false,
  refetchOnWindowFocus: false,
});

// routes/__root.tsx — fetch session once, inject into context
export const Route = createRootRouteWithContext<{ queryClient: QueryClient }>()({
  beforeLoad: async ({ context: { queryClient } }) => {
    const session = await queryClient.ensureQueryData(sessionQueryOptions);
    return { session };
  },
});

// routes/_authed.tsx — guard layout, single place for all protected routes
export const Route = createFileRoute("/_authed")({
  beforeLoad: ({ context }) => {
    if (!context.session?.user) throw redirect({ to: "/sign-in" });
    return { user: context.session.user };
  },
  component: () => <Outlet />,
});
```

**Critical:** `beforeLoad` re-runs on **every** client-side navigation. Without
`staleTime` and `refetchOnMount: false`, every link click triggers a server round-trip
for session validation. `ensureQueryData` returns cached data instantly when fresh.

Reactive auth hooks remain useful for **UI rendering** (showing user avatar, org name) —
just never use them as the source of truth for route access control.

For Better Auth specifically, see `better-auth-start.md` — it has additional patterns
for the nanostores signal system that causes re-render loops with TanStack Router.

---

## Preventing FOUC with `_authed` Layout Route

Use a pathless layout route (`_authed`) as a single auth guard for all protected pages.
Because `beforeLoad` runs before rendering, the redirect happens server-side on first
load — unauthenticated HTML is never sent to the browser.

```text
routes/
  __root.tsx       ← fetchSession in beforeLoad, injects into context
  _authed.tsx      ← checks context.session, throws redirect if absent
  _authed/
    dashboard.tsx
    settings.tsx
  sign-in.tsx      ← never nested under _authed
```

The login route should only redirect **away** if the user is already authenticated:

```typescript
// routes/sign-in.tsx
export const Route = createFileRoute("/sign-in")({
  beforeLoad: ({ context }) => {
    if (context.session?.user) throw redirect({ to: "/" });
  },
});
```

This prevents the `login ↔ dashboard` infinite redirect loop — login never guards
against unauthenticated users (they're supposed to be there).

---

## Hydration Mismatch Prevention

The dehydrated query state is embedded as JSON in the HTML. Components that read auth
state from route context (`Route.useRouteContext()`) render identically server and client.

**Patterns that cause mismatches:**

- Reading `localStorage` directly in render (server has no `localStorage`)
- Using auth reactive hooks (`useSession()`) that return `null` on server, data on client
- Branching on `typeof window` in render

**Fixes:**

Use `ssr: "data-only"` for routes where component rendering unavoidably differs:

```typescript
export const Route = createFileRoute("/_authed/dashboard")({
  ssr: "data-only", // loader runs server-side, component renders client-only
});
```

For isolated client-only UI elements, use `useHydrated()`:

```typescript
import { useHydrated } from "@tanstack/react-router";

function TimezoneDisplay() {
  const hydrated = useHydrated();
  if (!hydrated) return <Skeleton />;
  return <span>{Intl.DateTimeFormat().resolvedOptions().timeZone}</span>;
}
```

---

## Cloudflare Workers Environment

Workers don't have `process.env` at runtime. Access bindings via `cloudflare:workers`
— only inside `createServerFn` handlers (not directly in `beforeLoad` or `loader`):

```typescript
import { env } from "cloudflare:workers";

const getConfig = createServerFn({ method: "GET" }).handler(async () => {
  return { secret: env.AUTH_SECRET }; // typed via wrangler types
});
```

### Vite plugin order

```typescript
// vite.config.ts
plugins: [
  cloudflare({ viteEnvironment: { name: "ssr" } }), // FIRST
  tanstackStart(),
  react(),
];
```

### Required compatibility flags

```jsonc
// wrangler.jsonc
{ "compatibility_flags": ["nodejs_compat"] } // TanStack Start uses Node APIs
```

---

## Auth Hooks and HMR Stability

Auth client libraries that create global state (nanostores, event listeners, subscriptions)
accumulate instances under Vite HMR. Each hot update creates a new client without cleaning
up the old one, causing cascading refetches and UI freezes.

**Symptom:** Playwright tests pass (no HMR) but browser freezes on navigation.

### Pattern 1: Singleton with HMR cleanup

```typescript
// lib/auth-client.ts
import { createAuthClient } from "auth-lib/client"; // not /react

export const authClient = createAuthClient({ baseURL: "/api/auth" });

if (import.meta.hot) {
  import.meta.hot.dispose(() => {
    authClient.destroy?.(); // cleanup listeners if the lib supports it
  });
}
```

### Pattern 2: Isolate outside the route tree

Place auth client creation in a module that is not part of the route tree (e.g.,
`lib/auth-client.ts`). Route files are the most frequently HMR'd — keeping stateful
singletons out of them reduces accumulation.

### Diagnosing HMR vs logic bugs

If behavior differs between Playwright (fast, no HMR) and the browser (slow, with HMR):

1. Hard-refresh the browser (`Cmd+Shift+R`) — clears HMR module cache
2. Test in incognito without extensions — React DevTools can cause extra re-renders
3. Disable `defaultPreload: "intent"` — hover-triggered `beforeLoad` fires auth checks
   on every link hover, masking the real problem
4. Binary search: strip components from the layout until the freeze stops

---

## Common Pitfalls

1. **`useSecureCookies: true` on HTTP localhost** — browser silently drops the cookie.
   Condition on `NODE_ENV` or protocol.

2. **Blanket `invalidateQueries()` after mutations** — invalidates session queries too,
   triggering auth hook refetches that cascade through every component using `useSession`.
   Always scope: `invalidateQueries({ queryKey: ["specific-entity"] })`.
   See `integration.md` for query key namespacing patterns.

3. **Auth reactive hooks in `beforeLoad`** — hooks cannot run in loader context. Use
   server-validated session checks via `createServerFn`.

4. **`trustedOrigins: []`** — empty array rejects all cross-origin requests. Must
   explicitly list every client origin.

5. **`getRequest().url` as fetch target in SSR** — during SSR on Vite dev server, the
   request URL routes back through the proxy, creating an infinite loop. Use
   `getRequestHeaders()` for same-process or internal URL for separate service.

6. **Auth client module not HMR-safe** — global listeners and stores accumulate on each
   hot update. Use `import.meta.hot.dispose` or isolate outside HMR boundary.

7. **`beforeLoad` re-runs on every client navigation** — without `staleTime` on the
   session query, every link click triggers a server round-trip. Use `ensureQueryData`
   with `staleTime: 5 * 60 * 1000` and `refetchOnMount: false`.

8. **Login ↔ dashboard redirect loop** — login route must never guard against
   unauthenticated users. Only redirect away if the user IS authenticated.

9. **`cloudflare:workers` import in isomorphic code** — fails during client build.
   Only import inside `createServerFn` handlers or `.server()` middleware blocks.
