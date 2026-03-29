# Error Handling: thiserror, anyhow & Patterns

## Library Errors with thiserror v2

`thiserror` 2.0 generates `std::error::Error` implementations from annotated
enums. Zero heap allocation for the error enum itself.

### Defining error types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("parse error at line {line}: {message}")]
    Parse { line: usize, message: String },

    #[error("not found: {0}")]
    NotFound(String),

    #[error(transparent)]  // delegates Display + source to inner error
    Other(#[from] anyhow::Error),
}
```

### Key attributes

| Attribute               | Effect                                             |
| ----------------------- | -------------------------------------------------- |
| `#[error("...")]`       | Generates `Display` impl with format string        |
| `#[from]`               | Generates `From<T>` impl for automatic `?` convert |
| `#[source]`             | Marks field as the error source (for chaining)     |
| `#[error(transparent)]` | Delegates Display + source to inner error          |

### thiserror v2 vs v1 changes

- `#[error(transparent)]` now properly delegates `source()` — in v1 it could
  return `None` for some wrapper types
- `#[from]` works with `#[backtrace]` more reliably
- Minor: error message formatting is more consistent with `std::fmt`

---

## Application Errors with anyhow

`anyhow::Error` is a heap-allocated, type-erased error wrapper. Always add
context before propagating.

### Adding context

```rust
use anyhow::{Context, Result};

fn load_config(path: &str) -> Result<Config> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read config: {path}"))?;

    let config: Config = toml::from_str(&text)
        .context("config file has invalid TOML syntax")?;

    validate(&config)
        .context("config validation failed")?;

    Ok(config)
}
```

### When to downcast

```rust
fn handle_error(err: &anyhow::Error) {
    // Check for specific error types
    if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
        match io_err.kind() {
            ErrorKind::NotFound => { /* handle */ }
            ErrorKind::PermissionDenied => { /* handle */ }
            _ => { /* fallback */ }
        }
    }

    // Print full error chain
    eprintln!("Error: {err:#}"); // '#' flag prints the full chain
}
```

---

## Result Combinators

Prefer combinators over nested `match` for short transformation chains:

```rust
// Transform the Ok value
let doubled = result.map(|x| x * 2);

// Transform the Err value
let mapped = result.map_err(|e| AppError::from(e));

// Chain fallible operations
let final_result = result
    .and_then(|x| validate(x))
    .and_then(|x| process(x));

// Provide fallback on error
let value = result.unwrap_or_else(|e| {
    log::warn!("using default due to: {e}");
    default_value()
});

// Convert Option to Result
let item = maybe_item.ok_or_else(|| AppError::NotFound("item".into()))?;
```

### Iterator + Result patterns

```rust
// WRONG: collects all results, even after first error
let results: Vec<Result<i32, _>> = strings.iter()
    .map(|s| s.parse::<i32>())
    .collect();

// RIGHT: fails fast on first error
let values: Result<Vec<i32>, _> = strings.iter()
    .map(|s| s.parse::<i32>())
    .collect();

// Filter out errors, keeping only successes
let values: Vec<i32> = strings.iter()
    .filter_map(|s| s.parse::<i32>().ok())
    .collect();

// Partition into successes and failures
let (oks, errs): (Vec<_>, Vec<_>) = strings.iter()
    .map(|s| s.parse::<i32>())
    .partition(Result::is_ok);
```

---

## Error Design Guidelines

### Hierarchical errors for crate boundaries

```rust
// crate::db
#[derive(Error, Debug)]
pub enum DbError {
    #[error("connection failed")]
    Connection(#[source] std::io::Error),
    #[error("query failed: {0}")]
    Query(String),
}

// crate::api — wraps DbError
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("database error")]
    Db(#[from] DbError),
    #[error("validation: {0}")]
    Validation(String),
    #[error("unauthorized")]
    Unauthorized,
}
```

### Don't use `Box<dyn Error>` as your error type

It loses the ability to match on specific variants. Use `thiserror` enums for
structured errors, `anyhow` for application-level error propagation.

### Performance

- `thiserror` enums: 0 bytes heap allocation (stack-allocated enum)
- `anyhow::Error`: ~48+ bytes per error (heap-allocated box)
- Don't use `anyhow` in hot loops — prefer `thiserror` for performance-critical
  error paths

### The `#[must_use]` pattern

Always add `#[must_use]` to functions returning `Result` if the caller might
accidentally ignore the error:

```rust
#[must_use]
pub fn delete_file(path: &Path) -> Result<(), IoError> { ... }
```

`Result` already has `#[must_use]` on the type, but adding it to the function
provides a more specific warning message.
