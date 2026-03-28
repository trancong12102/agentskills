# TanStack Start — Best Practices & Patterns

TanStack Start is a full-stack React framework built on TanStack Router + Vinxi
(Vite + Nitro). It adds server functions, SSR streaming, and middleware on top of
Router's type-safe routing.

## Server Functions — `createServerFn`

Type-safe RPC that executes on the server, callable from client or server:

```typescript
import { createServerFn } from "@tanstack/react-start";

export const getUser = createServerFn({ method: "GET" }).handler(async () => {
  return db.users.findFirst();
});

// With validation + middleware
export const getTodos = createServerFn({ method: "GET" })
  .inputValidator(zodValidator(z.object({ userId: z.string() })))
  .middleware([authMiddleware])
  .handler(async ({ data, context }) => {
    return db.todos.findMany({ where: { userId: data.userId } });
  });
```

### createServerFn vs createServerOnlyFn

- `createServerFn` — callable from both client (via HTTP) and server (direct call)
- `createServerOnlyFn` — throws error if called from client. For sensitive operations
  (reading `process.env.SECRET`, direct DB writes)

### Streaming server functions

```typescript
// Async generator (cleaner)
const streamingFn = createServerFn().handler(async function* () {
  for (const msg of messages) {
    await sleep(500);
    yield msg;
  }
});
```

### Integration with React Query

```typescript
// serverFns/posts.ts
export const getPost = createServerFn({ method: "GET" })
  .validator((data: { postId: string }) => data)
  .handler(async ({ data }) =>
    db.posts.findUnique({ where: { id: data.postId } }),
  );

// queries/posts.ts
export const postQueryOptions = (postId: string) =>
  queryOptions({
    queryKey: ["posts", postId],
    queryFn: () => getPost({ data: { postId } }),
  });
```

---

## Middleware

### Request middleware (wraps server function calls)

```typescript
import { createMiddleware } from "@tanstack/react-start";

const authMiddleware = createMiddleware().server(async ({ next, request }) => {
  const session = await auth.getSession({ headers: request.headers });
  if (!session) throw new Error("Unauthorized");
  return await next({ context: { session } });
});
```

### Function middleware with client + server sides

```typescript
const authMiddleware = createMiddleware({ type: "function" })
  .client(async ({ next }) => {
    return next({ headers: { Authorization: `Bearer ${getToken()}` } });
  })
  .server(async ({ next }) => {
    // server-side logic
    return await next();
  });
```

Middleware chains compose via `.middleware([mw1, mw2])`.

---

## SSR Patterns

TanStack Start defaults to streaming SSR. The HTML shell renders immediately and
deferred data streams in.

Root route setup:

```typescript
export const Route = createRootRoute({
  head: () => ({
    meta: [
      { charSet: 'utf-8' },
      { name: 'viewport', content: 'width=device-width, initial-scale=1' },
    ],
  }),
  component: () => (
    <html>
      <head><HeadContent /></head>
      <body>
        <Outlet />
        <Scripts />
      </body>
    </html>
  ),
})
```

Use `defer`/`Await` from TanStack Router for streaming deferred data in loaders.

---

## How Start Differs from Next.js & Remix

| Feature          | Next.js App Router                  | TanStack Start                         |
| ---------------- | ----------------------------------- | -------------------------------------- |
| Component model  | Server Components (opt-in client)   | Client components (SSR by default)     |
| RSC support      | Production-ready                    | Actively developing, not default yet   |
| Data fetching    | `async` Server Components + `fetch` | Route `loader` + `createServerFn`      |
| Type safety      | Partial (no search param inference) | Full end-to-end TS                     |
| Server functions | Server Actions (form-first)         | `createServerFn` — callable anywhere   |
| Caching          | Complex implicit `fetch` cache      | React Query or route `staleTime`       |
| Build system     | Webpack/Turbopack                   | Vite + Vinxi (Nitro)                   |
| Deployment       | Vercel-optimized                    | Node, Bun, CF Workers, Vercel, Netlify |

Key differences:

- No `"use client"` / `"use server"` boundary confusion — normal React components that
  SSR and hydrate. Server-exclusive logic is explicit via `createServerFn`
- No implicit caching layers — use React Query or explicit `staleTime`
- Type safety is first-class — search params, path params, loader data, route context
- Data fetching in `loader` functions runs before rendering (no waterfalls)

---

## Configuration

```typescript
// vite.config.ts
import { tanstackStart } from "@tanstack/react-start/plugin/vite";

export default defineConfig({
  plugins: [
    tanstackStart({
      srcDirectory: "src",
      router: { routesDirectory: "routes" },
    }),
    viteReact(),
  ],
});
```
