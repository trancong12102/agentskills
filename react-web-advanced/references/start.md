# TanStack Start — Best Practices & Patterns

TanStack Start is a full-stack React framework built on TanStack Router + Vinxi
(Vite + Nitro). It adds server functions, SSR streaming, and middleware on top of
Router's type-safe routing.

## Server Functions — `createServerFn`

Type-safe RPC: `createServerFn({ method }).inputValidator(schema).middleware([...]).handler(fn)`.
Chain is fully type-safe — `data` and `context` types flow through.

### createServerFn vs createServerOnlyFn

- `createServerFn` — callable from both client (via HTTP) and server (direct call)
- `createServerOnlyFn` — throws error if called from client. For sensitive operations
  (reading `process.env.SECRET`, direct DB writes)

Streaming: use async generators in handlers (`async function*`).

### Integration with React Query

```typescript
// serverFns/posts.ts
export const getPost = createServerFn({ method: "GET" })
  .inputValidator((data: { postId: string }) => data)
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

Chain via `.middleware([mw1, mw2])` on server functions. Two types:

- **Request middleware** (`createMiddleware().server(fn)`) — wraps handler, reads headers,
  injects context (auth sessions, tracing)
- **Function middleware** (`createMiddleware({ type: "function" })`) — has `.client()` side
  (runs in browser before request, injects headers) and `.server()` side

For auth session middleware that reads cookies and injects user context, see
`ssr-auth.md` — it covers the SSR-specific cookie lifecycle that middleware depends on.

---

## SSR Patterns

Start defaults to streaming SSR — HTML shell renders immediately, deferred data streams in.
Use `defer`/`Await` from TanStack Router for non-blocking loader data. For SSR with
cookie-based auth (session cookies, Vite proxy in dev, CORS), see `ssr-auth.md`.

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
