# Data-fetching reinventions (TanStack Query / SWR / RTK Query)

Load when the diff includes a React file AND any of `@tanstack/react-query`, `swr`, `@reduxjs/toolkit/query` is installed — OR when the diff shows the classic `useEffect + fetch + setState` trio regardless of installed libs.

The hand-rolled trio has four recurring bugs: no cancellation on unmount, race between overlapping requests, missing error boundary, no cache. Every mature query lib handles all four.

## The reinvented trio

```tsx
const [data, setData] = useState(null);
const [loading, setLoading] = useState(true);
const [error, setError] = useState(null);
useEffect(() => {
  setLoading(true);
  fetch(`/api/${id}`)
    .then((r) => r.json())
    .then((d) => setData(d))
    .catch(setError)
    .finally(() => setLoading(false));
}, [id]);
```

Matched by `react-fetch-useeffect` rule. Every query lib compresses this to 1 line.

## TanStack Query (preferred if installed)

```tsx
const { data, isLoading, error } = useQuery({
  queryKey: ["item", id],
  queryFn: () => fetch(`/api/${id}`).then((r) => r.json()),
});
```

Commonly reinvented features:

| feature                        | manual pattern                           | useQuery option                         |
| ------------------------------ | ---------------------------------------- | --------------------------------------- |
| Cache invalidation             | `setData(null)` + refetch                | `queryClient.invalidateQueries(...)`    |
| Retry on error                 | Custom loop + exponential backoff        | `retry: 3` (+ optional `retryDelay`)    |
| Stale-while-revalidate         | Cache first + re-fetch in background     | Default SWR behavior                    |
| Dependent queries              | Chained `useEffect`s                     | `enabled: !!prereq`                     |
| Pagination                     | Manual `page` state + URL param          | `useInfiniteQuery` + `getNextPageParam` |
| Optimistic update              | Immediately set + revert on failure      | `onMutate` + `onError` rollback         |
| Polling                        | `setInterval` + clearInterval on unmount | `refetchInterval: ms`                   |
| Window focus refetch           | `visibilitychange` + focus listeners     | `refetchOnWindowFocus` (on by default)  |
| Shared cache across components | Custom React context                     | Automatic via `queryKey` equality       |

### Mutations

Hand-rolled:

```tsx
const onSubmit = async (values) => {
  setSubmitting(true);
  try {
    await fetch("/api/x", { method: "POST", body: JSON.stringify(values) });
    refetch();
  } catch (e) {
    setErr(e);
  }
  setSubmitting(false);
};
```

With query:

```tsx
const mutation = useMutation({
  mutationFn: (values) => ky.post("/api/x", { json: values }),
  onSuccess: () => queryClient.invalidateQueries({ queryKey: ["x"] }),
});
```

## SWR (preferred if installed, smaller footprint)

```tsx
const { data, error, isLoading } = useSWR(`/api/${id}`, (url) =>
  fetch(url).then((r) => r.json()),
);
```

Feature parity with TanStack Query for most cases. Differences:

- SWR has simpler API surface (~3kb vs ~13kb).
- TanStack Query has richer devtools, infinite-query support, and better TS types.
- Don't suggest one over the other if installed — honor project choice.

## RTK Query (preferred if Redux Toolkit is installed)

```ts
const api = createApi({
  reducerPath: "api",
  baseQuery: fetchBaseQuery({ baseUrl: "/api/" }),
  endpoints: (b) => ({
    getItem: b.query<Item, string>({ query: (id) => `item/${id}` }),
  }),
});

const { data } = api.useGetItemQuery(id);
```

Suggest only if `@reduxjs/toolkit` ≥ 1.8 is installed. Otherwise over-engineered for the use case.

## When NOT to use a query lib

- Single page with one request that loads once and never needs refetch — `useEffect` + `useState` is fine.
- Build-time / server-side data where React doesn't manage the lifecycle.
- Non-React codebase — go with `ky` / `ofetch` / `axios` and a plain in-memory Map cache.

Threshold: if the file has ≥2 fetch calls OR any of {retry, cache, refetch-on-focus, poll, invalidation}, recommend a lib.

## Suggesting which lib to install

Order:

1. Already-installed query lib — use it.
2. `@reduxjs/toolkit` installed → suggest RTK Query.
3. Otherwise → suggest TanStack Query (industry default, best docs, active maintenance).
4. If project is bundle-constrained (<50kb budget, edge target) → SWR.

Never suggest `react-query` (v3 name) — always `@tanstack/react-query` (v4+).

## Server-side data fetching

For Next.js / Remix / Astro, prefer the framework's loader pattern over `useEffect`:

| framework            | idiom                                                     |
| -------------------- | --------------------------------------------------------- |
| Next.js App Router   | `async function Page() { const data = await fetch(...) }` |
| Next.js Pages Router | `getServerSideProps` / `getStaticProps`                   |
| Remix                | `export async function loader({ params })`                |
| Astro                | Top-level `await` in `.astro` frontmatter                 |
| TanStack Start       | `loader` on `createRoute`                                 |

Flag `useEffect + fetch` in a server-first framework as a higher-priority (P1) reinvention — it misses the point of the framework.
