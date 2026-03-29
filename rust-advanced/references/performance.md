# Performance: Zero-Cost Abstractions, SIMD & Optimization

## The Zero-Cost Abstraction Principle

Iterator chains (`filter().map().sum()`) compile to the same machine code as
hand-written loops. The compiler fuses the chain into a single pass with no
intermediate allocations.

```rust
// These produce identical assembly:
let sum: i64 = data.iter().filter(|x| **x > 0).map(|x| x * 2).sum();

let mut sum: i64 = 0;
for x in &data {
    if *x > 0 { sum += x * 2; }
}
```

**Prefer iterator chains** for readability — the compiler handles optimization.

---

## When Zero-Cost Breaks: The SIMD Trap

Stateful iterator chains (where each `next()` depends on previous state) can
block auto-vectorization. The compiler cannot batch independent elements when
there's a data dependency between iterations.

### Signs of blocked vectorization

- Custom `Iterator::next()` that reads and writes to `&mut self` state
- Reduction with conditional accumulation depending on the running total
- Recursive iterator adapters

### Fixes

1. **Use `fold`/`try_fold`** instead of external `next()` loops — gives the
   compiler a single function body to vectorize:

   ```rust
   // SLOW: opaque next() calls prevent vectorization
   let sum: f64 = custom_iter.sum();

   // FAST: fold exposes the inner loop
   let sum: f64 = data.iter().fold(0.0, |acc, &x| acc + x);
   ```

2. **Restructure as plain slice iteration** when the iterator abstraction
   prevents optimization:

   ```rust
   // When iterator overhead matters:
   for chunk in data.chunks(4) {
       // Process chunks — SIMD-friendly
   }
   ```

3. **Explicit SIMD** via `std::arch` with target-feature gates:

   ```rust
   #[cfg(target_arch = "x86_64")]
   use std::arch::x86_64::*;

   #[target_feature(enable = "avx2")]
   unsafe fn sum_avx2(data: &[f32]) -> f32 { ... }
   ```

   Note: `std::simd` (portable SIMD) is still nightly-only as of early 2026.
   For stable code, use `std::arch` or the `wide` crate.

---

## Heap vs Stack Allocation

| Allocation         | When to use                                     |
| ------------------ | ----------------------------------------------- |
| Stack (default)    | Small, fixed-size types (< 1KB rule of thumb)   |
| `Box<T>`           | Recursive types, large structs, trait objects   |
| `Box<[T]>`         | Fixed-size slices (no capacity overhead vs Vec) |
| `Vec<T>`           | Dynamic-size collections                        |
| `SmallVec<[T; N]>` | Usually-small, occasionally-large collections   |
| `ArrayVec<T, N>`   | Bounded-size, stack-only, no heap fallback      |

### Pre-allocation

```rust
// WRONG: reallocates as it grows
let mut v = Vec::new();
for i in 0..1000 { v.push(i); }

// RIGHT: single allocation
let mut v = Vec::with_capacity(1000);
for i in 0..1000 { v.push(i); }

// Also RIGHT: collect with size hint
let v: Vec<_> = (0..1000).collect();  // iterator provides size hint
```

### String pre-allocation

```rust
// WRONG: repeated allocation in hot loop
let mut result = String::new();
for item in items {
    result += &format!("{item}, ");
}

// RIGHT: pre-allocate or use write!
use std::fmt::Write;
let mut result = String::with_capacity(items.len() * 20);
for item in items {
    write!(result, "{item}, ").unwrap();
}
```

---

## Arena Allocation (bumpalo)

Arena allocators provide O(1) allocation and O(1) bulk deallocation. Ideal for
workloads where all allocations share the same lifetime.

```rust
use bumpalo::Bump;

fn parse_request(data: &[u8]) -> Response {
    let arena = Bump::new();

    // All allocations freed when arena is dropped
    let headers = arena.alloc_slice_copy(parse_headers(data));
    let body = arena.alloc_str(parse_body(data));

    process(headers, body)
} // arena dropped — everything freed in one operation
```

**Best for:** compilers, parsers, request handlers, tree/graph construction
where the entire structure is built and discarded together.

---

## Data Parallelism with rayon

Replace `.iter()` with `.par_iter()` for automatic work-stealing parallelism:

```rust
use rayon::prelude::*;

// Sequential
let sum: u64 = data.iter().map(|x| expensive(x)).sum();

// Parallel — same API, uses all CPU cores
let sum: u64 = data.par_iter().map(|x| expensive(x)).sum();
```

### When to use rayon vs tokio

| Workload           | Use                      | Why                            |
| ------------------ | ------------------------ | ------------------------------ |
| CPU-bound parallel | `rayon`                  | Work-stealing, cache-friendly  |
| I/O-bound async    | `tokio`                  | Non-blocking, high concurrency |
| CPU in async ctx   | `spawn_blocking` + rayon | Don't block the async runtime  |

```rust
// CPU work inside async context
async fn process_data(data: Vec<f64>) -> Vec<f64> {
    tokio::task::spawn_blocking(move || {
        data.par_iter().map(|x| expensive(*x)).collect()
    }).await.unwrap()
}
```

---

## Cache Optimization

### Struct layout

```rust
// Default: Rust may reorder fields for optimal alignment
struct Data {
    a: u8,    // 1 byte
    b: u64,   // 8 bytes
    c: u16,   // 2 bytes
}
// Rust may reorder to: b, c, a → 8 + 2 + 1 + padding = 16 bytes

// Force C-compatible layout (for FFI or manual control)
#[repr(C)]
struct CData {
    a: u8,
    b: u64,
    c: u16,
}
// Layout: a(1) + pad(7) + b(8) + c(2) + pad(6) = 24 bytes
```

### Struct-of-Arrays vs Array-of-Structs

```rust
// Array-of-Structs: bad for iterating over one field (cache misses)
struct Particles {
    particles: Vec<Particle>,  // each Particle has x, y, z, mass, color...
}

// Struct-of-Arrays: cache-friendly for per-field iteration
struct Particles {
    x: Vec<f32>,
    y: Vec<f32>,
    z: Vec<f32>,
    mass: Vec<f32>,
}
// Iterating over x values touches contiguous memory — SIMD-friendly
```

### Alignment for SIMD

```rust
#[repr(C, align(32))]
struct SimdAligned {
    data: [f32; 8],  // 8 × f32 = 32 bytes, aligned for AVX
}
```

---

## Profiling Before Optimizing

**Never optimize without profiling first.** Use these tools:

| Tool            | What it measures          | When to use               |
| --------------- | ------------------------- | ------------------------- |
| `criterion`     | Microbenchmarks (ns/iter) | Comparing implementations |
| `flamegraph`    | CPU time distribution     | Finding hotspots          |
| `perf`/`dtrace` | System-level profiling    | Production profiling      |
| `dhat`          | Heap allocation profiling | Finding allocation bloat  |
| `cargo-bloat`   | Binary size analysis      | Reducing binary size      |

```rust
// Criterion benchmark example
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_sort(c: &mut Criterion) {
    c.bench_function("sort_1000", |b| {
        b.iter(|| {
            let mut data = (0..1000).rev().collect::<Vec<_>>();
            data.sort();
        })
    });
}

criterion_group!(benches, bench_sort);
criterion_main!(benches);
```
