# TanStack Router v1 — Best Practices & Patterns

## Type-Safe Routing

The type system is built on the route tree. Every route links to its parent via
`getParentRoute`, enabling TypeScript to infer params, search, and loader data end-to-end.

Register the router for global inference:

```typescript
declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
```

Without this registration, all navigation and hooks are typed as `any`.

### Root route with typed context (dependency injection)

```typescript
interface MyRouterContext {
  queryClient: QueryClient;
}

const rootRoute = createRootRouteWithContext<MyRouterContext>()({
  component: App,
});

const router = createRouter({
  routeTree,
  context: { queryClient },
  defaultPreload: "intent",
  defaultPreloadStaleTime: 0, // Let React Query manage staleness
});
```

---

## File-Based Routing — Non-Obvious Conventions

- `_pathlessLayout.tsx` — wraps children with layout/auth without adding a URL segment
- `(app)/` — route group, organizational only, no URL impact
- `-components/` — prefix `-` excludes from route tree (co-locate route-specific components)

---

## Search Params — Type-Safe Validation

Search params are **always validated** through `validateSearch`. Without it, they are `unknown`.

### With Zod (recommended)

```typescript
import { z } from "zod";

const productSearchSchema = z.object({
  page: z.number().catch(1),
  filter: z.string().catch(""),
  sort: z.enum(["newest", "oldest", "price"]).catch("newest"),
});

export const Route = createFileRoute("/shop/products")({
  validateSearch: productSearchSchema,
});
```

Always use `.catch(fallback)` instead of `.default(value)`. `.default()` only handles
missing keys; `.catch()` also handles type coercion failures from malformed URLs.

### With Valibot (Standard Schema, no adapter needed)

```typescript
import * as v from "valibot";

const schema = v.object({
  page: v.optional(v.fallback(v.number(), 1), 1),
  sort: v.optional(
    v.fallback(v.picklist(["newest", "oldest", "price"]), "newest"),
    "newest",
  ),
});

export const Route = createFileRoute("/shop/products/")({
  validateSearch: schema,
});
```

Access in components via `Route.useSearch()` — fully typed to the validated shape.

---

## Loaders & React Query Integration

### Pattern: ensureQueryData for blocking, prefetchQuery for non-blocking

```typescript
export const Route = createFileRoute('/posts/$postId')({
  loader: async ({ context: { queryClient }, params }) => {
    // Fire-and-forget secondary data
    queryClient.prefetchQuery(commentsQueryOptions(params.postId))
    // Block route render until critical data is ready
    await queryClient.ensureQueryData(postQueryOptions(params.postId))
  },
  component: PostPage,
})

function PostPage() {
  const { postId } = Route.useParams()
  const { data: post } = useSuspenseQuery(postQueryOptions(postId))
  return <h1>{post.title}</h1>
}
```

### loaderDeps — reactive loader inputs

Only the extracted deps trigger re-load, not any other search param change:

```typescript
export const Route = createFileRoute("/posts")({
  validateSearch: productSearchSchema,
  loaderDeps: ({ search }) => ({ page: search.page }),
  loader: ({ deps }) => fetchPosts(deps),
});
```

### Deferred/streaming data

```typescript
import { defer, Await } from '@tanstack/react-router'

export const Route = createFileRoute('/posts/$postId')({
  loader: async ({ params }) => {
    const post = await fetchPost(params.postId)          // awaited — blocking
    const comments = fetchComments(params.postId)         // not awaited
    return { post, comments: defer(comments) }
  },
  component: PostPage,
})

function PostPage() {
  const { post, comments } = Route.useLoaderData()
  return (
    <>
      <h1>{post.title}</h1>
      <Suspense fallback={<Spinner />}>
        <Await promise={comments}>
          {(data) => data.map(c => <div key={c.id}>{c.text}</div>)}
        </Await>
      </Suspense>
    </>
  )
}
```

`Await` requires a `<Suspense>` ancestor — runtime error without it.

---

## Code Splitting

### File-based: `.lazy.tsx` suffix

Split route into critical config (loader, params) and non-critical rendering (component):

```typescript
// routes/posts.tsx — critical config only
export const Route = createFileRoute("/posts")({
  loader: ({ context: { queryClient } }) =>
    queryClient.ensureQueryData(postsQueryOptions),
});
```

```typescript
// routes/posts.lazy.tsx — rendering only
export const Route = createLazyFileRoute("/posts")({
  component: Posts,
  pendingComponent: PostsSkeleton,
  errorComponent: PostsError,
});
```

### Code-based: `.lazy()`

```typescript
const postsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/posts",
  loader: ({ context }) => context.queryClient.ensureQueryData(postsQuery),
}).lazy(() => import("./posts.lazy").then((d) => d.Route));
```

### lazyRouteComponent for component-only splits

```typescript
import { lazyRouteComponent } from "@tanstack/react-router";

const route = createRoute({
  ...opts,
  component: lazyRouteComponent(() => import("../pages/PostsIndex")),
});
```

---

## Navigation — linkOptions

Use `linkOptions` for reusable, type-safe navigation configs shared across components:

```typescript
const dashboardLink = linkOptions({ to: '/dashboard', search: { tab: 'overview' } })
<Link {...dashboardLink}>Dashboard</Link>
```

---

## Route Context — Dependency Injection

Context layers merge down the tree: router-level -> per-route `beforeLoad` -> children.

```typescript
export const Route = createFileRoute("/admin")({
  beforeLoad: async ({ context }) => {
    const user = await context.auth.getUser();
    if (!user.isAdmin) throw redirect({ to: "/login" });
    return { user }; // merged into context for this route and children
  },
  loader: ({ context }) => fetchAdminData(context.user.id),
});
```

---

## Pending & Error States

Use `pendingMs`/`pendingMinMs` to control pending UI timing. Use `router.invalidate()`
(not `reset()`) when error came from a loader — it coordinates both router reload and
error boundary reset together.

---

## Common Pitfalls

1. **Wrong property order in createFileRoute** — must be
   `validateSearch -> loaderDeps -> beforeLoad -> loader` for TS inference.
   Install `@tanstack/eslint-plugin-router` with `create-route-property-order` rule.

2. **Returning entire search in loaderDeps** — invalidates cache on any param change:

   ```typescript
   // Bad
   loaderDeps: ({ search }) => search;
   // Good
   loaderDeps: ({ search }) => ({ page: search.page });
   ```

3. **useLoaderData in notFoundComponent** — not valid. Use `Route.useParams()` or
   pass data via `throw notFound({ data: ... })`.

4. **Not registering the router** — without `declare module Register`, everything is `any`.

5. **Forgetting Suspense around Await** — runtime error without a Suspense ancestor.

6. **preload="intent" without cache** — preloaded data is discarded if user doesn't
   navigate. Combine with React Query for cache persistence.
