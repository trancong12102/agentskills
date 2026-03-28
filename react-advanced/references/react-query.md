# TanStack React Query v5 — Best Practices & Patterns

## queryOptions Factory Pattern

The single most important structural pattern. Define query options once, reuse everywhere:

```typescript
import { queryOptions } from "@tanstack/react-query";

export const postsQueryOptions = queryOptions({
  queryKey: ["posts"],
  queryFn: fetchPosts,
  staleTime: 30_000,
});

export const postQueryOptions = (postId: string) =>
  queryOptions({
    queryKey: ["posts", postId],
    queryFn: () => fetchPost(postId),
    staleTime: 30_000,
  });

// Usage in components
const { data } = useQuery(postsQueryOptions);

// Usage for prefetching — same object
queryClient.prefetchQuery(postsQueryOptions);

// Type-safe cache read — data type inferred automatically
const data = queryClient.getQueryData(postsQueryOptions.queryKey);
```

### Query key hierarchy pattern

```typescript
const todoKeys = {
  all: ["todos"] as const,
  lists: () => [...todoKeys.all, "list"] as const,
  list: (filters: string) => [...todoKeys.lists(), { filters }] as const,
  details: () => [...todoKeys.all, "detail"] as const,
  detail: (id: number) => [...todoKeys.details(), id] as const,
};

// Invalidate all todo queries
queryClient.invalidateQueries({ queryKey: todoKeys.all });
// Invalidate only detail queries
queryClient.invalidateQueries({ queryKey: todoKeys.details() });
```

Query keys are hashed deterministically — object property order inside a key segment does
not matter, but array item order does.

---

## useQuery vs useSuspenseQuery

| Concern            | `useQuery`                             | `useSuspenseQuery`           |
| ------------------ | -------------------------------------- | ---------------------------- |
| Loading state      | `isPending` / `isLoading` on component | `<Suspense fallback>` parent |
| Error state        | `isError` / `error` on component       | `<ErrorBoundary>` parent     |
| `data` type        | `TData \| undefined`                   | `TData` (always defined)     |
| `enabled` option   | Supported                              | Not supported                |
| `placeholderData`  | Supported                              | Not supported                |
| TypeScript benefit | Requires null checks                   | `data` is non-nullable       |

Prefer `useSuspenseQuery` when co-locating loading/error UI at boundary level. Use
`startTransition` when changing queryKey to avoid replacing UI with fallback on updates.

### Dependent queries

With `useQuery` — use `enabled`:

```typescript
const { data: user } = useQuery({
  queryKey: ["user", email],
  queryFn: getUserByEmail,
});
const { data: projects } = useQuery({
  queryKey: ["projects", user?.id],
  queryFn: getProjectsByUser,
  enabled: !!user?.id,
});
```

With `useSuspenseQuery` — dependencies are automatically serial (React suspends until
first query resolves), no `enabled` needed.

### Parallel queries

Static number: call `useQuery` multiple times. Dynamic number: use `useQueries`:

```typescript
const usersMessages = useQueries({
  queries: userIds.map((id) => ({
    queryKey: ["messages", id],
    queryFn: () => getMessagesByUsers(id),
  })),
});
```

---

## Mutation Patterns

### Optimistic updates

```typescript
const mutation = useMutation({
  mutationFn: updateTodo,
  onMutate: async (newTodo) => {
    // 1. Cancel outgoing refetches (MUST await)
    await queryClient.cancelQueries({ queryKey: ["todos", newTodo.id] });
    // 2. Snapshot for rollback
    const previousTodo = queryClient.getQueryData(["todos", newTodo.id]);
    // 3. Optimistically update cache
    queryClient.setQueryData(["todos", newTodo.id], newTodo);
    // 4. Return snapshot as context
    return { previousTodo };
  },
  onError: (err, newTodo, context) => {
    // Rollback on failure
    queryClient.setQueryData(["todos", newTodo.id], context.previousTodo);
  },
  onSettled: (data, error, newTodo) => {
    // Always sync with server (return the promise to keep mutation pending)
    return queryClient.invalidateQueries({ queryKey: ["todos", newTodo.id] });
  },
});
```

Key rules:

- Always `await queryClient.cancelQueries()` in `onMutate`
- Return the `invalidateQueries` promise from `onSettled` to keep mutation pending until
  refetch completes
- The return value of `onMutate` becomes `context` in `onError`/`onSettled`

---

## Cache Management

### staleTime vs gcTime

| Option      | Default | Meaning                                                |
| ----------- | ------- | ------------------------------------------------------ |
| `staleTime` | `0`     | How long data is fresh. No refetch during this window. |
| `gcTime`    | `5 min` | How long inactive cached data is kept before GC.       |

Recommended global default:

```typescript
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 60_000, // 1 minute
      gcTime: 5 * 60_000, // 5 minutes (default)
    },
  },
});
```

### Prefetching strategies

```typescript
queryClient.prefetchQuery(opts); // Respects staleTime, skips if fresh
queryClient.fetchQuery(opts); // Force fetch regardless of staleness
queryClient.ensureQueryData(opts); // Return cache if present, else fetch
```

Prefetch on hover for instant navigation:

```typescript
<Link
  onMouseEnter={() => queryClient.prefetchQuery(postQueryOptions(id))}
  to={`/posts/${id}`}
>
```

### placeholderData vs initialData

|                          | `placeholderData`                 | `initialData` |
| ------------------------ | --------------------------------- | ------------- |
| Persisted to cache?      | No                                | Yes           |
| Triggers refetch?        | Always                            | Only if stale |
| `isPlaceholderData` flag | Yes                               | No            |
| Use case                 | Skeleton data, `keepPreviousData` | SSR props     |

Use `keepPreviousData` helper for pagination:

```typescript
import { keepPreviousData } from "@tanstack/react-query";

const { data, isPlaceholderData } = useQuery({
  queryKey: ["projects", page],
  queryFn: () => fetchProjects(page),
  placeholderData: keepPreviousData,
});
```

### select for data transformation

Transforms data before returning to component. Re-runs only when data or `select`
reference changes:

```typescript
const { data: names } = useQuery({
  ...userOptions(),
  select: useCallback((users) => users.map((u) => u.name), []),
});
```

Do not put `select` results into `useState` — the most common derived-state anti-pattern.

---

## Common Pitfalls

1. **staleTime: 0 (default)** — every mount triggers refetch. Set a meaningful global default.
2. **Hardcoded query keys** — use `queryOptions` factory. Typos break cache sharing silently.
3. **Derived state in useState** — use `select` instead of `useEffect` + `setState`.
4. **Not awaiting cancelQueries** — in-flight refetch overwrites optimistic update.
5. **Not returning invalidateQueries promise** — mutation transitions to success before
   refetch completes.
6. **refetchOnWindowFocus with staleTime: 0** — feels jarring in forms. Increase `staleTime`
   or disable per-query.
7. **v5 breaking changes from v4** — `cacheTime` renamed to `gcTime`, `keepPreviousData`
   option replaced by `placeholderData: keepPreviousData` helper, `initialPageParam`
   required for infinite queries, `onSuccess`/`onError`/`onSettled` removed from `useQuery`.
