---
name: rust-advanced
description: "Advanced Rust patterns for ownership, traits, async, error handling, macros, type system tricks, unsafe, and performance. Use when tackling complex Rust problems — not basic syntax, but multi-concern tasks like designing cancellation-safe async services, choosing between trait objects and generics, building typestated APIs, structuring error hierarchies across crate boundaries, writing proc macros, or optimizing hot paths with zero-cost abstractions. Do not use for basic Rust syntax, simple CLI tools, or beginner ownership questions."
---

# Rust Advanced: Patterns, Conventions & Pitfalls

This skill defines rules, conventions, and architectural decisions for building
production Rust applications. It is intentionally opinionated to prevent common
pitfalls and enforce patterns that scale.

For detailed API documentation of any crate mentioned here, use other appropriate
tools (documentation lookup, web search, etc.) — this skill focuses on **how** and
**why** to use these patterns, not full API surfaces.

## Table of Contents

1. [Ownership & Borrowing Rules](#ownership--borrowing-rules)
2. [Error Handling Strategy](#error-handling-strategy)
3. [Trait System Conventions](#trait-system-conventions)
4. [Async Rust Rules](#async-rust-rules)
5. [Type System Patterns](#type-system-patterns)
6. [Performance Decision Framework](#performance-decision-framework)
7. [Unsafe Policy](#unsafe-policy)
8. [Common Pitfalls](#common-pitfalls)
9. [Reference Files](#reference-files)

---

## Ownership & Borrowing Rules

### Interior mutability — decision flowchart

```text
Need shared mutation?
  YES → Single-threaded or multi-threaded?
    Single-threaded → Is T: Copy?
      YES → Cell<T> (zero overhead, no borrow tracking)
      NO  → RefCell<T> (runtime borrow checking, panics on violation)
    Multi-threaded → High contention?
      NO  → Arc<Mutex<T>> (simple, correct)
      YES → Arc<RwLock<T>> (many readers, few writers)
             or lock-free types (crossbeam, atomic)
  NO → Use normal ownership / borrowing
```

### Smart pointer selection

| Type          | When to use                                              |
| ------------- | -------------------------------------------------------- |
| `Box<T>`      | Recursive types, large stack values, trait objects       |
| `Rc<T>`       | Single-threaded shared ownership (trees, graphs)         |
| `Arc<T>`      | Multi-threaded shared ownership                          |
| `Cow<'a, T>`  | Sometimes borrowed, sometimes owned — avoid eager clones |
| `Pin<Box<T>>` | Self-referential types, async futures                    |

### The Cow rule

Accept `Cow<str>` or `Cow<[T]>` when a function sometimes modifies its input and
sometimes passes it through unchanged. This avoids allocating when no modification
is needed. Prefer `&str` in function arguments when you never need ownership.

---

## Error Handling Strategy

### The golden rule: libraries use `thiserror`, applications use `anyhow`

| Context              | Crate       | Why                                                      |
| -------------------- | ----------- | -------------------------------------------------------- |
| Library crate        | `thiserror` | Callers need to match on specific error variants         |
| Binary / application | `anyhow`    | Errors bubble up to user-facing messages with context    |
| Internal modules     | `thiserror` | Type-safe error variants for the parent module to handle |
| FFI boundary         | Custom enum | Must map to C-compatible error codes                     |

### Required patterns

1. **Always add context** when propagating with `?` in application code:

   ```rust
   fs::read_to_string(path)
       .with_context(|| format!("failed to read config: {path}"))?;
   ```

2. **Use `#[from]` for automatic conversions** in library error enums:

   ```rust
   #[derive(thiserror::Error, Debug)]
   pub enum DbError {
       #[error("connection failed: {0}")]
       Connection(#[from] std::io::Error),
       #[error("query failed: {reason}")]
       Query { reason: String },
   }
   ```

3. **Prefer `Result` combinators** over nested `match` for short chains:
   `map`, `map_err`, `and_then`, `unwrap_or_else`.

4. **Never `unwrap()` in library code.** Use `expect()` only when the invariant
   is documented and provably upheld.

---

## Trait System Conventions

### Trait objects vs generics — decision rule

```text
Need runtime polymorphism (heterogeneous collection, plugin system)?
  YES → dyn Trait (Box<dyn Trait> or &dyn Trait)
  NO  → impl Trait / generics (zero-cost, monomorphized)
```

### Key patterns

- **Associated types** over generics when there is exactly one natural
  implementation per type (e.g., `Iterator::Item`).
- **Sealed traits** when you need to prevent downstream crates from implementing
  your trait — essential for semver stability.
- **Blanket implementations** to extend functionality to all types satisfying a
  bound (e.g., `impl<T: Display> ToString for T`).
- **Supertraits** when your trait logically requires another trait's guarantees
  (e.g., `trait Printable: Debug + Display`).

### Object safety rules

A trait is object-safe (can be used as `dyn Trait`) only if:

- No methods return `Self`
- No methods have generic type parameters
- All methods take `self`, `&self`, or `&mut self`

If you need `dyn Trait + async`, use `#[async_trait]` or return
`Box<dyn Future>` manually — native async in traits is not yet object-safe.

---

## Async Rust Rules

### Runtime: Tokio is the default

Use `tokio` with `#[tokio::main]` and `#[tokio::test]`. For CPU-bound work
inside an async context, use `tokio::task::spawn_blocking` or `rayon`.

### Native async traits — drop `#[async_trait]` where possible

Since Rust 1.75, `async fn` in traits works natively. Use native syntax unless
you need `dyn Trait` with async methods.

### The Send/Sync rule

Futures passed to `tokio::spawn` must be `Send`. The #1 cause of non-Send
futures: holding a `MutexGuard` (or any `!Send` type) across an `.await` point.

**Fix:** drop the guard before awaiting, or scope the lock in a block:

```rust
{
    let mut guard = lock.lock().unwrap();
    guard.push(42);
} // guard dropped
do_async_thing().await; // future is Send
```

### Cancellation safety — the most dangerous async footgun

Any future can be dropped at any `.await` point (especially in `tokio::select!`).
Know which operations are cancel-safe:

| Operation                    | Cancel-safe? |
| ---------------------------- | ------------ |
| `mpsc::Receiver::recv`       | Yes          |
| `AsyncReadExt::read`         | Yes          |
| `AsyncWriteExt::write_all`   | **No**       |
| `AsyncBufReadExt::read_line` | **No**       |

For cancel-unsafe code: wrap in `tokio::spawn` (dropping a `JoinHandle` does not
cancel the spawned task) or use `tokio_util::sync::CancellationToken` for
cooperative cancellation.

### Structured concurrency: use `JoinSet`

```rust
let mut set = tokio::task::JoinSet::new();
for url in urls {
    set.spawn(fetch(url));
}
while let Some(result) = set.join_next().await {
    result??;
}
```

---

## Type System Patterns

### Newtype — zero-cost domain types

Wrap primitives to create distinct types. Prevents mixing `UserId` with `OrderId`:

```rust
struct UserId(u64);
struct OrderId(u64);
// fn process(user: UserId, order: OrderId) — compiler prevents swaps
```

### Typestate — compile-time state machine

Encode lifecycle states as type parameters. Invalid transitions become compile errors:

```rust
struct Connection<S> { socket: TcpStream, _state: PhantomData<S> }
struct Disconnected;
struct Connected;

impl Connection<Disconnected> {
    fn connect(self) -> Result<Connection<Connected>> { ... }
}
impl Connection<Connected> {
    fn send(&self, data: &[u8]) -> Result<()> { ... }
    // send() is unavailable on Connection<Disconnected>
}
```

### Const generics — array sizes as type parameters

```rust
struct Matrix<const ROWS: usize, const COLS: usize> {
    data: [[f64; COLS]; ROWS],
}
impl<const N: usize> Matrix<N, N> {
    fn trace(&self) -> f64 { (0..N).map(|i| self.data[i][i]).sum() }
}
```

### PhantomData variance

| Marker                    | Variance      | Use for                 |
| ------------------------- | ------------- | ----------------------- |
| `PhantomData<T>`          | Covariant     | "Owns" a T conceptually |
| `PhantomData<fn(T)>`      | Contravariant | Consumes T (rare)       |
| `PhantomData<fn(T) -> T>` | Invariant     | Must be exact type      |
| `PhantomData<*const T>`   | Invariant     | Raw pointer semantics   |

---

## Performance Decision Framework

```text
Is this a hot path (profiled, not guessed)?
  NO  → Write clear, idiomatic code. Don't optimize.
  YES → Which bottleneck?
    CPU-bound computation → rayon::par_iter() for data parallelism
    Many small allocations → Arena allocator (bumpalo)
    Iterator chain not vectorizing → Check for stateful dependencies,
      use fold/try_fold, or restructure as plain slice iteration
    Cache misses → #[repr(C)] + align, struct-of-arrays layout
    Heap allocation → Box<[T]> instead of Vec<T> when size is fixed,
      stack allocation for small types, SmallVec for usually-small vecs
```

### The zero-cost rule

Iterator chains (`filter().map().sum()`) compile to the same code as hand-written
loops — prefer them for readability. But stateful iterator chains can block
auto-vectorization; see `references/performance.md` for SIMD details.

---

## Unsafe Policy

1. **Minimize scope** — wrap only the minimum number of lines in `unsafe {}`.
2. **Mandatory `// SAFETY:` comment** on every `unsafe` block explaining
   why the invariants are upheld.
3. **Prefer safe abstractions** — `as` casts, `bytemuck::cast`, `from_raw_parts`
   over `transmute`. Use `transmute` only as last resort with turbofish syntax.
4. **FFI boundary rule:** generate bindings with `bindgen`, wrap in a thin safe
   Rust API, document every invariant.
5. **Never use `unsafe` to bypass the borrow checker.** If you think you need to,
   redesign the data structure.

---

## Common Pitfalls

1. **Holding `MutexGuard` across `.await`** — makes the future `!Send`, breaks
   `tokio::spawn`. Scope the lock in a block before awaiting.

2. **`RefCell` double borrow panic** — `borrow_mut()` panics if any borrow is
   live. Use `try_borrow_mut()` when borrow lifetimes aren't fully controlled.

3. **`Mutex` deadlock** — Rust's `Mutex` is non-reentrant. Never lock the same
   mutex twice on one thread. Acquire multiple locks in consistent order.

4. **`collect::<Vec<Result<T, E>>>()` vs `collect::<Result<Vec<T>, E>>()`** —
   the second form fails fast on first error and is almost always what you want.

5. **Accepting `&String` instead of `&str`** — `&String` auto-derefs to `&str`
   but not vice versa. Always accept `&str` in function signatures.

6. **`unwrap()` in library code** — crashes the caller. Use `?` with proper
   error types, or `expect()` with documented invariant.

7. **Forgetting `#[must_use]` on `Result`-returning functions** — callers may
   silently ignore errors. The compiler warns, but custom types need the attribute.

8. **Using `std::sync::Mutex` in async code** — blocks the executor thread.
   Use `tokio::sync::Mutex` for async contexts.

9. **`String::from` in hot loops** — allocates each iteration. Pre-allocate
   with `String::with_capacity()` or use `Cow<str>`.

10. **Ignoring cancellation safety in `select!`** — the non-winning future is
    dropped. Cancel-unsafe operations lose data silently.

11. **`clone()` as first instinct** — usually a sign of fighting the borrow
    checker. Restructure ownership or use references first.

12. **`Box<dyn Error>` instead of proper error enum** — loses the ability to
    match on specific variants. Use `thiserror` for structured errors.

---

## Reference Files

Read the relevant reference file when working with a specific topic:

| File                           | When to read                                                   |
| ------------------------------ | -------------------------------------------------------------- |
| `references/ownership.md`      | Interior mutability, smart pointers, Cow, Pin, lifetime tricks |
| `references/traits.md`         | Trait objects, sealed traits, blanket impls, HRTB, variance    |
| `references/error-handling.md` | thiserror v2, anyhow, Result combinators, error design         |
| `references/async-rust.md`     | Tokio runtime, cancellation, JoinSet, Send/Sync, select!       |
| `references/performance.md`    | Zero-cost, SIMD, arena allocation, rayon, cache optimization   |
| `references/unsafe-ffi.md`     | Unsafe superpowers, FFI with bindgen, transmute, raw pointers  |
| `references/macros.md`         | Declarative macros, proc macros, derive macros, syn/quote      |
| `references/type-patterns.md`  | Newtype, typestate, PhantomData, const generics, builder       |
