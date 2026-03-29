# Async Rust: Tokio, Cancellation & Structured Concurrency

## Tokio Runtime

### Setup

```rust
#[tokio::main]
async fn main() {
    // Multi-threaded runtime by default
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Single-threaded — for WASM, embedded, or test isolation
}

#[tokio::test]
async fn my_test() {
    // Each test gets its own runtime
}
```

### spawn vs spawn_blocking

| Function         | Thread pool | Use for                    |
| ---------------- | ----------- | -------------------------- |
| `tokio::spawn`   | Async       | I/O-bound futures (`Send`) |
| `spawn_blocking` | Blocking    | CPU-bound or sync I/O      |
| `rayon::spawn`   | Rayon       | CPU-parallel computation   |

**Never block the async runtime** with CPU work or synchronous I/O:

```rust
// WRONG: blocks the executor
async fn bad() {
    let result = expensive_cpu_work(); // blocks
}

// RIGHT: offload to blocking thread pool
async fn good() {
    let result = tokio::task::spawn_blocking(|| expensive_cpu_work()).await?;
}
```

---

## Send + Sync Bounds

Futures passed to `tokio::spawn` must be `Send`. Common causes of non-Send:

| Cause                             | Fix                                       |
| --------------------------------- | ----------------------------------------- |
| `MutexGuard` held across `.await` | Drop guard before `.await`                |
| `Rc<T>` in async context          | Use `Arc<T>` instead                      |
| `RefCell<T>` borrow across await  | Use `tokio::sync::Mutex`                  |
| Non-Send type in closure          | Move the non-Send work outside the future |

### The guard-across-await pattern

```rust
// WRONG: MutexGuard lives across await point
async fn bad(data: &Mutex<Vec<u8>>) {
    let mut guard = data.lock().unwrap();
    guard.push(42);
    do_async().await;  // ERROR: future is not Send
}

// RIGHT: scope the guard
async fn good(data: &Mutex<Vec<u8>>) {
    {
        let mut guard = data.lock().unwrap();
        guard.push(42);
    } // guard dropped here
    do_async().await;  // OK: future is Send
}

// ALTERNATIVE: use tokio::sync::Mutex (async-aware)
async fn also_good(data: &tokio::sync::Mutex<Vec<u8>>) {
    let mut guard = data.lock().await;
    guard.push(42);
    // guard can live across await with tokio's Mutex
}
```

### std::sync::Mutex vs tokio::sync::Mutex

| Feature                   | `std::sync::Mutex`      | `tokio::sync::Mutex`   |
| ------------------------- | ----------------------- | ---------------------- |
| Blocking behavior         | Blocks OS thread        | Yields to runtime      |
| Guard across `.await`     | Not Send — error        | Send — allowed         |
| Performance (uncontended) | Faster (no async)       | Slightly slower        |
| When to use               | Short critical sections | Long or async sections |

**Rule:** use `std::sync::Mutex` for quick data access (microseconds), switch
to `tokio::sync::Mutex` only when you need to hold the lock across await points
or when contention is high.

---

## Cancellation Safety

### The problem

In `tokio::select!`, when one branch completes, all other branches' futures are
**dropped** — cancelled mid-execution. Any state changes made before the last
`.await` point in the dropped future are lost.

### Cancel-safe vs cancel-unsafe operations

| Operation                      | Safe?  | Why                                     |
| ------------------------------ | ------ | --------------------------------------- |
| `mpsc::Receiver::recv()`       | Yes    | Message stays in channel if cancelled   |
| `oneshot::Receiver::recv()`    | Yes    | Value stays in channel                  |
| `AsyncReadExt::read()`         | Yes    | No partial state to lose                |
| `AsyncReadExt::read_exact()`   | **No** | Partial read data lost                  |
| `AsyncWriteExt::write_all()`   | **No** | Partial write — data partially sent     |
| `AsyncBufReadExt::read_line()` | **No** | Partial line data accumulated then lost |
| `tokio::time::sleep()`         | Yes    | No state                                |

### Making cancel-unsafe code safe

**Strategy 1: wrap in `tokio::spawn`** — dropping a `JoinHandle` does NOT cancel
the task:

```rust
let handle = tokio::spawn(cancel_unsafe_write(socket, data));
// Even if handle is dropped, the task continues to completion
```

**Strategy 2: use a `CancellationToken`** for cooperative cancellation:

```rust
use tokio_util::sync::CancellationToken;

let token = CancellationToken::new();
let child = token.child_token();

tokio::spawn(async move {
    tokio::select! {
        _ = child.cancelled() => {
            // Clean up gracefully
        }
        result = do_work() => {
            // Normal completion
        }
    }
});

// Later: request cancellation
token.cancel();
```

**Strategy 3: pre-compute before select!**

```rust
// WRONG: cancel-unsafe inside select!
loop {
    tokio::select! {
        line = reader.read_line(&mut buf) => { ... }
        _ = shutdown.recv() => break,
    }
}

// RIGHT: use a fuse pattern or pre-read
loop {
    let read_fut = reader.read_line(&mut buf);
    tokio::pin!(read_fut);

    tokio::select! {
        result = &mut read_fut => { ... }
        _ = shutdown.recv() => break,
    }
}
```

---

## Structured Concurrency

### JoinSet — spawn and collect

```rust
use tokio::task::JoinSet;

let mut set = JoinSet::new();

for url in urls {
    set.spawn(async move { fetch(&url).await });
}

let mut results = Vec::new();
while let Some(result) = set.join_next().await {
    results.push(result??); // ?? unwraps JoinError then app error
}
```

### JoinSet with abort-on-first-error

```rust
let mut set = JoinSet::new();
for task in tasks {
    set.spawn(process(task));
}

while let Some(result) = set.join_next().await {
    match result? {
        Ok(value) => handle(value),
        Err(e) => {
            set.abort_all(); // cancel remaining tasks
            return Err(e);
        }
    }
}
```

### Graceful shutdown pattern

```rust
use tokio::signal;

async fn run_server() -> Result<()> {
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::broadcast::channel(1);

    let server = tokio::spawn(serve(shutdown_tx.subscribe()));
    let worker = tokio::spawn(background_work(shutdown_tx.subscribe()));

    signal::ctrl_c().await?;
    let _ = shutdown_tx.send(());

    // Wait for graceful shutdown with timeout
    tokio::time::timeout(
        Duration::from_secs(30),
        async {
            let _ = server.await;
            let _ = worker.await;
        }
    ).await.ok();

    Ok(())
}
```

---

## Async Traits

### Native async traits (Rust 1.75+)

```rust
trait Fetcher {
    async fn fetch(&self, url: &str) -> Result<String>;
}

impl Fetcher for HttpClient {
    async fn fetch(&self, url: &str) -> Result<String> {
        self.get(url).await?.text().await
    }
}
```

### When you still need `#[async_trait]`

Native async traits use `impl Future` in return position, which is **not
object-safe**. For `dyn Trait` with async methods:

```rust
use async_trait::async_trait;

#[async_trait]
trait DynFetcher: Send + Sync {
    async fn fetch(&self, url: &str) -> Result<String>;
}

// Now you can use Box<dyn DynFetcher>
fn create_fetcher() -> Box<dyn DynFetcher> { ... }
```

**Migration rule:** if you don't need `dyn Trait`, remove `#[async_trait]` and
use native syntax.
