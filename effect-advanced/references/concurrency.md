# Concurrency — Fibers, Fork Variants & Structured Concurrency

## Fibers

Lightweight virtual threads managed by the Effect scheduler. They run on the
JavaScript event loop — not OS threads. Millions can run without native thread
overhead.

**Lifecycle:** created -> running -> suspended -> done/interrupted

---

## High-Level APIs — Use These First

Always prefer these over raw `Effect.fork`:

```typescript
// Bounded parallel execution
Effect.all([task1, task2, task3], { concurrency: 3 });

// Worker pool pattern
Effect.forEach(items, processItem, { concurrency: 10 });

// Race: first wins, others interrupted
Effect.race(effect1, effect2);

// Concurrent zip
Effect.zip(effect1, effect2, { concurrent: true });

// Timeout
Effect.timeout(program, Duration.seconds(5));
```

---

## Fork Variants — Know the Lifecycle

| Function               | Scope          | Auto-cleanup                          |
| ---------------------- | -------------- | ------------------------------------- |
| `Effect.fork`          | Parent's scope | Interrupted when parent terminates    |
| `Effect.forkDaemon`    | Global scope   | Nothing — you must interrupt manually |
| `Effect.forkIn(scope)` | Explicit Scope | Tied to provided scope                |
| `Effect.forkScoped`    | Nearest Scope  | Tied to enclosing resource lifecycle  |

### Structured concurrency

Child fibers forked with `Effect.fork` are scoped to their parent. When the parent
terminates, all supervised children are automatically interrupted. This makes fiber
leaks nearly impossible — unless you use `forkDaemon`.

**Gotcha:** `forkDaemon` leaks fibers if you forget to interrupt them. Nothing will
clean them up.

**Gotcha:** If you fork a long-running task with plain `fork` inside a short-lived
scope, it will be interrupted when the scope closes. Use `forkDaemon` or `forkIn`
with an appropriate scope for background services.

---

## Fiber Interruption

Effect uses **asynchronous interruption** (not cooperative polling):

```typescript
// Blocks until target fully terminates (back-pressuring)
Fiber.interrupt(fiber);

// Fire-and-forget — does not wait for termination
Fiber.interruptFork(fiber);
```

Finalizers in the interrupted fiber **always run** — they are uninterruptible.

### `Fiber.join` vs `Fiber.await`

- `Fiber.join` — re-raises the fiber's error in the caller's error channel
- `Fiber.await` — returns the `Exit` value without re-raising

**Gotcha:** `Fiber.join` can cause premature finalizer execution in edge cases.
Prefer `Fiber.await` when resource safety matters.

---

## Deferred — Cross-Fiber Coordination

Promise-like, but typed and interruptible:

```typescript
const deferred = yield * Deferred.make<string, Error>();

// Producer fiber:
yield * Deferred.succeed(deferred, "result");

// Consumer fiber (blocks until resolved):
const value = yield * Deferred.await(deferred);
```

---

## Semaphore — Bounded Concurrent Access

```typescript
const semaphore = yield * Effect.makeSemaphore(5);
const limitedTask = semaphore.withPermits(1)(heavyTask);
```

Use for connection pooling, rate limiting, or throttling concurrent operations.

---

## Common Pitfalls

1. **Using raw `fork` when `Effect.all` suffices** — high-level APIs handle
   interruption, error propagation, and concurrency limits automatically.

2. **`forkDaemon` without cleanup** — detached fibers run forever. Always pair
   with explicit interruption logic.

3. **`Fiber.join` with scoped resources** — can trigger premature finalizer
   execution. Use `Fiber.await` + `Exit` pattern matching instead.

4. **Forgetting `{ concurrency: N }`** — `Effect.all` and `Effect.forEach` are
   sequential by default. Explicitly set concurrency for parallel execution.

5. **Unbounded concurrency** — `{ concurrency: "unbounded" }` can overwhelm
   external services. Always set a reasonable limit.
