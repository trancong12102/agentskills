# TanStack Virtual v3 — Best Practices & Patterns

## Core Concepts

`useVirtualizer` renders only items visible in the scroll window plus `overscan` extras.
Two critical requirements:

1. A **fixed-height scroll container** (`overflow: auto`, known height)
2. A **spacer element** sized to `getTotalSize()` inside it

```typescript
const rowVirtualizer = useVirtualizer({
  count: 10000,
  getScrollElement: () => parentRef.current,
  estimateSize: () => 35,
  overscan: 5,
})

<div ref={parentRef} style={{ height: '400px', overflow: 'auto' }}>
  <div style={{ height: rowVirtualizer.getTotalSize(), position: 'relative' }}>
    {rowVirtualizer.getVirtualItems().map((virtualRow) => (
      <div
        key={virtualRow.key}
        style={{
          position: 'absolute',
          top: 0, left: 0, width: '100%',
          height: virtualRow.size,
          transform: `translateY(${virtualRow.start}px)`,
        }}
      >
        {data[virtualRow.index]}
      </div>
    ))}
  </div>
</div>
```

---

## Dynamic Row Heights

Pass `measureElement` as ref callback with `data-index`:

```typescript
const virtualizer = useVirtualizer({
  count: items.length,
  getScrollElement: () => parentRef.current,
  estimateSize: () => 45,
})

// Packed-div pattern (more reliable for dynamic heights)
<div style={{
  position: 'absolute',
  top: 0, left: 0, width: '100%',
  transform: `translateY(${virtualizer.getVirtualItems()[0]?.start ?? 0}px)`,
}}>
  {virtualizer.getVirtualItems().map((virtualRow) => (
    <div
      key={virtualRow.key}
      data-index={virtualRow.index}
      ref={virtualizer.measureElement}
    >
      {items[virtualRow.index]}
    </div>
  ))}
</div>
```

---

## Horizontal and Grid Virtualization

### Horizontal

```typescript
const columnVirtualizer = useVirtualizer({
  horizontal: true,
  count: 10000,
  getScrollElement: () => parentRef.current,
  estimateSize: () => 100,
});
// Position: transform: `translateX(${virtualCol.start}px)`
```

### Grid

Combine row and column virtualizers on the same scroll container:

```typescript
transform: `translateX(${virtualColumn.start}px) translateY(${virtualRow.start}px)`;
```

Outer div sized to `rowVirtualizer.getTotalSize()` x `columnVirtualizer.getTotalSize()`.

---

## Infinite Scrolling with React Query

```typescript
const { data, fetchNextPage, hasNextPage, isFetchingNextPage } =
  useInfiniteQuery({
    queryKey: ["items"],
    queryFn: ({ pageParam }) => fetchPage(pageParam),
    getNextPageParam: (lastPage) => lastPage.nextOffset,
    initialPageParam: 0,
  });

const allRows = data?.pages.flatMap((p) => p.rows) ?? [];

const rowVirtualizer = useVirtualizer({
  count: hasNextPage ? allRows.length + 1 : allRows.length,
  getScrollElement: () => parentRef.current,
  estimateSize: () => 100,
  overscan: 5,
});

useEffect(() => {
  const lastItem = rowVirtualizer.getVirtualItems().at(-1);
  if (!lastItem) return;
  if (
    lastItem.index >= allRows.length - 1 &&
    hasNextPage &&
    !isFetchingNextPage
  ) {
    fetchNextPage();
  }
}, [
  rowVirtualizer.getVirtualItems(),
  hasNextPage,
  isFetchingNextPage,
  allRows.length,
]);
```

---

## Performance

- **`overscan`** — items rendered beyond visible area (default: 1). Increase to reduce
  blank frames during fast scrolling. Too high wastes CPU.
- **`gap`** — pixel spacing between items without needing margin per item.
- **`contain: 'strict'`** on scroll container — significant CSS performance win for large
  lists, prevents layout recalculations from propagating outside.
- **`useWindowVirtualizer`** — when browser window is the scroll container:

  ```typescript
  const virtualizer = useWindowVirtualizer({
    count: 10000,
    estimateSize: () => 35,
    scrollMargin: listRef.current?.offsetTop ?? 0,
  });
  ```

### Sticky headers

Override `rangeExtractor` to force header indices into render range:

```typescript
rangeExtractor: (range) => {
  const next = new Set([activeStickyIndex, ...defaultRangeExtractor(range)]);
  return [...next].sort((a, b) => a - b);
};
```

### Scroll restoration

```typescript
const virtualizer = useVirtualizer({
  initialOffset: savedScrollOffset,
  ...
})
// Or: virtualizer.scrollToIndex(savedIndex, { align: 'start' })
```

---

## Common Pitfalls

1. **No explicit height on scroll container** — must have defined height (px, vh, or
   flex: 1 in flex container). Without it, `overflow: auto` cannot scroll.

2. **Using array index as key** — always use `virtualRow.key`, not `virtualRow.index`.

3. **Wrong data-index for dynamic measurement** — `data-index={virtualRow.index}` must
   be on the outermost measured element, not a child.

4. **estimateSize too small** — if estimate is much smaller than actual, initial scroll
   calculations will be wrong. Estimate at largest typical size or slightly over.

5. **Multiple virtualizers without scrollMargin** — stacked virtualizers in same
   container need `scrollMargin` set to their pixel offset.
