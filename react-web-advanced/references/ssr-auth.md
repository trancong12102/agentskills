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

### SSR circular fetch prevention

During SSR, `createServerFn` runs on the Vite dev server. If the server function fetches
`getRequest().url` (which points back through the proxy), it creates an infinite loop:

```text
Browser → Vite (SSR) → server function → fetch("/api/...") → Vite (proxy) → API
                                                ↑                              |
                                                └── but if URL is the SSR URL ─┘  DEADLOCK
```

**Fix:** Server-side code must call the API directly, bypassing the proxy:

```typescript
// serverFns/auth.ts
const API_INTERNAL = process.env.API_INTERNAL_URL ?? "http://localhost:6781";

export const getSession = createServerFn({ method: "GET" }).handler(
  async () => {
    const request = getRequest();
    // Forward cookies from the browser request to the API
    const res = await fetch(`${API_INTERNAL}/api/auth/get-session`, {
      headers: { cookie: request.headers.get("cookie") ?? "" },
    });
    return res.json();
  },
);
```

Key rules:

- Client-side: relative paths (`/api/...`) → goes through Vite proxy
- Server-side (SSR): absolute internal URL → bypasses proxy, hits API directly
- Always forward the `cookie` header from the incoming request when making server-to-server
  auth calls

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
- Never put auth secrets (`BETTER_AUTH_SECRET`, DB credentials) in `VITE_*` vars

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
  staleTime: 300_000,
});

// routes/(authed).tsx
export const Route = createFileRoute("/(authed)")({
  beforeLoad: async ({ context: { queryClient } }) => {
    const session = await queryClient.ensureQueryData(sessionQueryOptions);
    if (!session?.user) throw redirect({ to: "/login" });
    return { session };
  },
});
```

Reactive auth hooks remain useful for **UI rendering** (showing user avatar, org name) —
just never use them as the source of truth for route access control.

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

3. **Auth reactive hooks in `beforeLoad`** — hooks cannot run in loader context. Use
   server-validated session checks via `createServerFn`.

4. **`trustedOrigins: []`** — empty array rejects all cross-origin requests. Must
   explicitly list every client origin.

5. **`getRequest().url` as fetch target in SSR** — during SSR on Vite dev server, the
   request URL routes back through the proxy, creating an infinite loop. Use an internal
   base URL for server-to-server calls.

6. **Auth client module not HMR-safe** — global listeners and stores accumulate on each
   hot update. Use `import.meta.hot.dispose` or isolate outside HMR boundary.
