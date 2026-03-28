# FlashList + React Query — List Patterns

## FlashList vs FlatList

| Concern        | FlatList               | FlashList                                 |
| -------------- | ---------------------- | ----------------------------------------- |
| Cell reuse     | No — creates new cells | Yes — recycles view instances             |
| Memory usage   | Higher                 | Lower                                     |
| Blank cells    | Common on fast scroll  | Rare; `onBlankArea` for monitoring        |
| Setup friction | None                   | Requires `estimatedItemSize`              |
| Heterogeneous  | Works, no hints        | Requires `getItemType` for best recycling |

**Use FlatList** when: list is short (<50 items) or items need full unmount/remount
lifecycle. **Use FlashList** for long, performance-critical lists.

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
      estimatedItemSize={120}
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

FlashList (and FlatList) can call `onEndReached` multiple times in quick succession.
The `!isFetchingNextPage` guard is essential — without it, duplicate page fetches produce
duplicated items.

### `getNextPageParam` must return `undefined` for "no more pages"

```typescript
// API returns null for no next page — coerce to undefined
getNextPageParam: (lastPage) => lastPage.nextCursor ?? undefined,
```

Returning `null` does not signal "no next page" in React Query v5 — only `undefined` does.

---

## estimatedItemSize

The most impactful prop. Gives FlashList a hint for initial layout before actual sizes
are measured.

- **Too high** → fewer items render initially, blank areas on fast scroll
- **Too low** → over-allocates cells at startup (minor perf cost)
- Use the **median** item height, not the mean (outliers skew the mean)
- FlashList warns in dev if estimate is significantly off — take the warning seriously

### overrideItemLayout — exact sizes when known

```typescript
<FlashList
  overrideItemLayout={(layout, item) => {
    layout.size = item.isHeader ? 80 : 60
  }}
  estimatedItemSize={60}
/>
```

Eliminates estimation errors for items with known sizes.

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
  estimatedItemSize={100}
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
