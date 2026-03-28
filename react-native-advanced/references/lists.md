# FlashList + React Query — List Patterns

## Why FlashList

FlashList recycles native view instances instead of creating new cells on every mount.
This means lower memory usage, fewer blank cells on fast scroll, and better frame rates.
v2 handles sizing automatically — no `estimatedItemSize` needed.

For heterogeneous lists, provide `getItemType` so the recycler matches cell types correctly.

---

## FlashList + useInfiniteQuery Pattern

```typescript
import { FlashList } from '@shopify/flash-list'
import { useInfiniteQuery } from '@tanstack/react-query'

function PostList() {
  const { data, fetchNextPage, hasNextPage, isFetchingNextPage, refetch, isRefetching } =
    useInfiniteQuery({
      queryKey: ['posts'],
      queryFn: ({ pageParam }) => fetchPosts(pageParam),
      initialPageParam: 0,
      getNextPageParam: (lastPage) => lastPage.nextCursor ?? undefined,
    })

  // Memoize — data.pages reference changes on each fetch
  const items = useMemo(
    () => data?.pages.flatMap((p) => p.items) ?? [],
    [data],
  )

  // Stable callback — avoids unnecessary FlashList rerenders
  const handleEndReached = useCallback(() => {
    if (hasNextPage && !isFetchingNextPage) fetchNextPage()
  }, [hasNextPage, isFetchingNextPage, fetchNextPage])

  return (
    <FlashList
      data={items}
      renderItem={({ item }) => <PostCard post={item} />}
      keyExtractor={(item) => item.id}
      onEndReached={handleEndReached}
      onEndReachedThreshold={0.3}
      ListFooterComponent={isFetchingNextPage ? <ActivityIndicator /> : null}
      refreshControl={
        <RefreshControl refreshing={isRefetching} onRefresh={refetch} />
      }
    />
  )
}
```

### Critical: guard against double-firing

FlashList can call `onEndReached` multiple times in quick succession.
The `!isFetchingNextPage` guard is essential — without it, duplicate page fetches produce
duplicated items.

### `getNextPageParam` must return `undefined` for "no more pages"

```typescript
// API returns null for no next page — coerce to undefined
getNextPageParam: (lastPage) => lastPage.nextCursor ?? undefined,
```

Returning `null` does not signal "no next page" in React Query v5 — only `undefined` does.

---

## FlashList v2 — Key Changes

FlashList v2 is a ground-up rewrite. **New Architecture (Fabric) is required.**

### Automatic Sizing (v2)

`estimatedItemSize` is **deprecated and ignored** in v2 — FlashList handles sizing
automatically. Remove it from your props. The dev warning about estimates being off no
longer fires.

### overrideItemLayout — only `span` works

In v2, the `layout` object only exposes `span` — the `size` field was removed from the
type entirely. Setting `layout.size` is a TypeScript error:

```typescript
<FlashList
  numColumns={3}
  overrideItemLayout={(layout, item) => {
    layout.span = item.isHeader ? 3 : 1 // header spans full width
  }}
/>
```

### MasonryFlashList → `masonry` prop

`MasonryFlashList` is deprecated in v2. Use the `masonry` boolean prop on `FlashList`:

```typescript
<FlashList masonry numColumns={2} data={items} renderItem={renderItem} />
```

### New v2 hooks

- **`useRecyclingState(defaultValue, deps)`** — state that resets when the cell is recycled
  for a different item. Solves stale-state bugs from v1 cell reuse.
- **`useLayoutState()`** — subscribe to layout changes without prop drilling.

### `onStartReached` (v2)

Symmetrical to `onEndReached` — fires when scrolling near the top. Use with
`onStartReachedThreshold` for chat-style reverse-scroll pagination.

---

## keyExtractor

```typescript
// Always use stable server-side IDs
keyExtractor={(item) => item.id}

// Namespace if items appear in multiple lists
keyExtractor={(item) => `post-${item.id}`}

// NEVER use index — breaks selection, animation, and state tracking
// BAD: keyExtractor={(item, index) => String(index)}
```

---

## getItemType — required for heterogeneous lists

When items have different layouts (headers, footers, different card types), FlashList must
know the type for optimal recycling:

```typescript
<FlashList
  data={items}
  getItemType={(item) => item.type}  // 'header' | 'card' | 'ad'
  renderItem={({ item }) => {
    if (item.type === 'header') return <Header item={item} />
    if (item.type === 'ad') return <Ad item={item} />
    return <Card item={item} />
  }}
/>
```

Without `getItemType`, FlashList recycles any view for any item — causing layout jumps
when a header cell is recycled as a card cell.

---

## onEndReachedThreshold tuning

- `0.3` — good default for typical list items
- `0.1` — for very tall items (full-screen images)
- `0.5` — for feeds where aggressive prefetching is desirable

---

## select for data transformation

Reverse page order or reshape data without touching the cache:

```typescript
useInfiniteQuery({
  ...opts,
  select: (data) => ({
    pages: [...data.pages].reverse(),
    pageParams: [...data.pageParams].reverse(),
  }),
});
```

---

## Common Pitfalls

1. **Not memoizing `items` array** — `data.pages.flatMap()` creates a new array reference
   on every render. Wrap with `useMemo`.

2. **Unstable `handleEndReached` callback** — recreated every render causes FlashList to
   re-register the listener. Wrap with `useCallback`.

3. **Missing `refreshControl` on pull-to-refresh** — use React Query's `refetch` and
   `isRefetching` to drive `RefreshControl`.

4. **FlashList inside ScrollView** — FlashList manages its own scroll. Wrapping in
   ScrollView causes double-scroll and breaks virtualization.
