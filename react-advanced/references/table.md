# TanStack Table v8 — Best Practices & Patterns

TanStack Table is headless — zero markup, pure state machine. Use `createColumnHelper` for
type-safe column definitions (3 types: `accessor` for data fields, `display` for UI-only,
`group` for nested headers). Wire features via `useReactTable` with tree-shaken row models.

---

## Server-Side Operations

Set `manual*: true` to tell the table data is pre-processed by the server. Omit the
corresponding row model since no client processing occurs:

```typescript
const table = useReactTable({
  columns,
  data,
  getCoreRowModel: getCoreRowModel(),
  // No getSortedRowModel — server handles sorting
  manualSorting: true,
  manualPagination: true,
  manualFiltering: true,
  rowCount: serverData.totalRows, // table derives pageCount
  state: { sorting, pagination, columnFilters },
  onSortingChange: setSorting,
  onPaginationChange: setPagination,
  onColumnFiltersChange: setColumnFilters,
});
```

Combine with React Query:

```typescript
const { data } = useQuery({
  queryKey: ["users", columnFilters, sorting, pagination],
  queryFn: () => fetchUsers({ columnFilters, sorting, pagination }),
});
```

---

## State Management — Controlled vs Uncontrolled

- **Uncontrolled**: no `state` prop — table manages internally (simple client-side tables)
- **Fully controlled**: pass all state + `on*Change` callbacks (server-side pattern)
- **Partially controlled**: control only needed state (e.g., only `pagination` for server
  fetching), leave the rest uncontrolled

---

## Performance with Virtualization

Integrate TanStack Virtual's `useVirtualizer` with `table.getRowModel().rows`:

```typescript
const { rows } = table.getRowModel();
const rowVirtualizer = useVirtualizer({
  count: rows.length,
  getScrollElement: () => parentRef.current,
  estimateSize: () => 35,
  overscan: 10,
});

const virtualRows = rowVirtualizer.getVirtualItems();
const paddingTop = virtualRows[0]?.start ?? 0;
const paddingBottom =
  rowVirtualizer.getTotalSize() - (virtualRows.at(-1)?.end ?? 0);
```

See `virtual.md` for full virtualization patterns.

### Column resizing performance

For smooth 60fps resizing:

- Compute all column widths via CSS variables upfront
- Memoize table body during active resizing
- Avoid calling `column.getSize()` per-cell on every render

---

## Common Pitfalls

1. **Column/data stability (most common bug)**:

   ```typescript
   // BAD — new reference every render -> infinite re-renders
   function MyTable() {
     const columns = [columnHelper.accessor('name', {})]  // new ref each render
     const data = fetchedData || []                         // new [] each render
   }

   // GOOD — stable references
   const columns = useMemo(() => [...], [])
   ```

2. **Missing `getRowId`** — without it, rows use array indices. Selection and reordering
   break across page changes.

3. **Column resizing without memoization** — calling `column.getSize()` inline per cell
   on every frame causes janky resizing.
