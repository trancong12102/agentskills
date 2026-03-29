# Stream — Operators, Chunking & Backpressure

## Core Type

`Stream<A, E, R>` — emits zero or more `A` values, may fail with `E`, requires
context `R`. Pull-based: downstream drives execution, providing automatic backpressure.

Internally, streams emit `Chunk<A>` batches (immutable, optimized array).

---

## Stream vs AsyncIterable

| Aspect              | `AsyncIterable` | `Stream`               |
| ------------------- | --------------- | ---------------------- |
| Error typing        | Untyped `throw` | Typed `E`              |
| Resource management | Manual          | `Scope`-based          |
| Concurrency ops     | None built-in   | Built-in               |
| Backpressure        | Manual          | Automatic (pull-based) |

---

## Creation

```typescript
Stream.fromArray([1, 2, 3]);
Stream.fromIterable(iter);
Stream.fromEffect(effect); // single-value stream
Stream.range(1, 100);
Stream.iterate(0, (n) => n + 1); // infinite
Stream.fromQueue(queue); // from Queue
```

---

## Transformation

```typescript
Stream.map(s, f);
Stream.mapEffect(s, f, { concurrency: 5 }); // concurrent mapping
Stream.flatMap(s, f);
Stream.filter(s, predicate);
Stream.take(s, n); // CRITICAL for infinite streams
Stream.rechunk(s, n); // control chunk sizes
```

---

## Consumption

```typescript
Stream.runCollect(s); // materialize to Chunk — NEVER on infinite streams
Stream.run(s, sink); // preferred for large/infinite data
Stream.runForEach(s, f); // side-effectful consumption
Stream.runDrain(s); // consume, discard values
```

---

## Concurrent Mapping

The `concurrency` option controls parallel effects. Without it, mapping is sequential:

```typescript
// Bounded concurrency, ordering preserved
Stream.mapEffect(s, heavyTask, { concurrency: 10 });
```

---

## Resourceful Streams

If you acquire a resource inside a stream (e.g., file handle), use
`Stream.acquireRelease`:

```typescript
const lines = Stream.acquireRelease(openFile("data.csv"), (handle) =>
  Effect.sync(() => handle.close()),
).pipe(
  Stream.flatMap((handle) =>
    Stream.fromAsyncIterable(handle.lines(), identity),
  ),
);
```

---

## Backpressure with Queue

When feeding a `Queue` from a stream:

```typescript
// Bounded queue — stream pauses when full (backpressure)
const queue = yield * Queue.bounded<string>(100);

// Unbounded — only when producers are known to be slower than consumers
const queue = yield * Queue.unbounded<string>();
```

---

## Common Pitfalls

1. **`runCollect` on infinite streams** — never call without a prior `take`. It will
   never terminate and consume unbounded memory.

2. **Forgetting `{ concurrency: N }` in `mapEffect`** — defaults to sequential.
   Set concurrency explicitly for parallel processing.

3. **Chunk size mismatch** — default chunk sizes affect throughput and memory. Use
   `Stream.rechunk(n)` to tune for your workload.

4. **Resource leaks in streams** — resources acquired inside a stream must use
   `Stream.acquireRelease`, not raw `Effect.acquireRelease`.

5. **Unbounded queues** — `Queue.unbounded()` provides no backpressure. Memory grows
   without limit if producer is faster than consumer.
