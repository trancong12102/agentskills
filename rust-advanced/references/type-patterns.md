# Type System Patterns

## Newtype Pattern

Wrap a primitive to create a distinct type with zero runtime cost. Prevents
mixing up semantically different values of the same underlying type.

```rust
struct UserId(u64);
struct OrderId(u64);

fn process_order(user: UserId, order: OrderId) {
    // Compiler prevents: process_order(order_id, user_id)
}
```

### Implementing common traits

```rust
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct UserId(u64);

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "user:{}", self.0)
    }
}

impl From<u64> for UserId {
    fn from(id: u64) -> Self { Self(id) }
}
```

### Deref for transparent access

Use `Deref` when the newtype should behave like its inner type in most contexts:

```rust
use std::ops::Deref;

struct Email(String);

impl Deref for Email {
    type Target = str;
    fn deref(&self) -> &str { &self.0 }
}

let email = Email("user@example.com".into());
println!("{}", email.len()); // calls str::len() via Deref
```

**Warning:** `Deref` abuse can be confusing. Only use when the newtype truly
IS-A wrapper over the inner type, not just HAS-A relationship.

### Newtype for orphan rule workaround

Implement foreign traits on foreign types by wrapping:

```rust
struct Wrapper(Vec<String>);

impl fmt::Display for Wrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.0.join(", "))
    }
}
```

---

## Typestate Pattern

Encode state machine states as type parameters. Invalid transitions become
compile-time errors.

### Basic pattern

```rust
use std::marker::PhantomData;

// States — zero-sized types (no runtime cost)
struct Draft;
struct Published;
struct Archived;

struct Article<State> {
    title: String,
    body: String,
    _state: PhantomData<State>,
}

// Constructors always return Draft
impl Article<Draft> {
    fn new(title: String, body: String) -> Self {
        Article { title, body, _state: PhantomData }
    }

    fn publish(self) -> Article<Published> {
        // `self` is consumed — Draft article no longer exists
        Article { title: self.title, body: self.body, _state: PhantomData }
    }
}

impl Article<Published> {
    fn archive(self) -> Article<Archived> {
        Article { title: self.title, body: self.body, _state: PhantomData }
    }

    fn view(&self) -> &str {
        &self.body
    }
}

impl Article<Archived> {
    fn unarchive(self) -> Article<Published> {
        Article { title: self.title, body: self.body, _state: PhantomData }
    }
}

// Usage:
let draft = Article::new("Hello".into(), "World".into());
// draft.view();        // ERROR: view() not available on Draft
// draft.archive();     // ERROR: archive() not available on Draft
let published = draft.publish();
let _ = published.view(); // OK
let archived = published.archive();
// published.view();    // ERROR: published was consumed
```

### Typestate with shared methods

Methods available in all states go on the unconstrained impl:

```rust
impl<S> Article<S> {
    fn title(&self) -> &str { &self.title }
    fn body(&self) -> &str { &self.body }
}
```

### Builder pattern with typestate

Ensure required fields are set before building:

```rust
struct NoName;
struct HasName;
struct NoEmail;
struct HasEmail;

struct UserBuilder<N, E> {
    name: Option<String>,
    email: Option<String>,
    age: Option<u32>,
    _name: PhantomData<N>,
    _email: PhantomData<E>,
}

impl UserBuilder<NoName, NoEmail> {
    fn new() -> Self {
        UserBuilder {
            name: None, email: None, age: None,
            _name: PhantomData, _email: PhantomData,
        }
    }
}

impl<E> UserBuilder<NoName, E> {
    fn name(self, name: String) -> UserBuilder<HasName, E> {
        UserBuilder {
            name: Some(name), email: self.email, age: self.age,
            _name: PhantomData, _email: PhantomData,
        }
    }
}

impl<N> UserBuilder<N, NoEmail> {
    fn email(self, email: String) -> UserBuilder<N, HasEmail> {
        UserBuilder {
            name: self.name, email: Some(email), age: self.age,
            _name: PhantomData, _email: PhantomData,
        }
    }
}

// build() only available when BOTH name and email are set
impl UserBuilder<HasName, HasEmail> {
    fn build(self) -> User {
        User {
            name: self.name.unwrap(),
            email: self.email.unwrap(),
            age: self.age,
        }
    }
}

// Optional fields — available in any state
impl<N, E> UserBuilder<N, E> {
    fn age(mut self, age: u32) -> Self {
        self.age = Some(age);
        self
    }
}

// Usage:
let user = UserBuilder::new()
    .name("Alice".into())
    .email("alice@example.com".into())
    .age(30)
    .build(); // OK

// UserBuilder::new().build(); // ERROR: build() not available
// UserBuilder::new().name("Alice".into()).build(); // ERROR: no email
```

---

## PhantomData

Zero-sized type that tells the compiler about type relationships without
storing any data.

### Common uses

**1. Marking ownership for lifetime correctness:**

```rust
struct Iter<'a, T> {
    ptr: *const T,
    end: *const T,
    _marker: PhantomData<&'a T>, // "I borrow T for 'a"
}
```

**2. Variance control:**

| Marker                    | Variance      | When to use                   |
| ------------------------- | ------------- | ----------------------------- |
| `PhantomData<T>`          | Covariant     | Container "owns" a T          |
| `PhantomData<fn(T)>`      | Contravariant | Container "consumes" T (rare) |
| `PhantomData<fn(T) -> T>` | Invariant     | Must be exact type            |
| `PhantomData<*const T>`   | Invariant     | Raw pointer semantics         |

**3. Unused type parameters (typestate):**

```rust
struct Connection<State> {
    stream: TcpStream,
    _state: PhantomData<State>, // State is only a type-level tag
}
```

---

## Const Generics

Type parameters that are values instead of types:

```rust
struct Buffer<const N: usize> {
    data: [u8; N],
    len: usize,
}

impl<const N: usize> Buffer<N> {
    fn new() -> Self {
        Buffer { data: [0; N], len: 0 }
    }

    fn push(&mut self, byte: u8) -> Result<(), ()> {
        if self.len >= N { return Err(()); }
        self.data[self.len] = byte;
        self.len += 1;
        Ok(())
    }
}

let small: Buffer<64> = Buffer::new();   // 64-byte stack buffer
let large: Buffer<4096> = Buffer::new(); // 4KB stack buffer
```

### Matrix with const generics

```rust
struct Matrix<const ROWS: usize, const COLS: usize> {
    data: [[f64; COLS]; ROWS],
}

impl<const ROWS: usize, const COLS: usize> Matrix<ROWS, COLS> {
    fn transpose(&self) -> Matrix<COLS, ROWS> {
        let mut result = Matrix { data: [[0.0; ROWS]; COLS] };
        for r in 0..ROWS {
            for c in 0..COLS {
                result.data[c][r] = self.data[r][c];
            }
        }
        result
    }
}

// Only square matrices can compute trace
impl<const N: usize> Matrix<N, N> {
    fn trace(&self) -> f64 {
        (0..N).map(|i| self.data[i][i]).sum()
    }
}

// Type system prevents dimension mismatches:
// let a: Matrix<2, 3> = ...;
// a.trace(); // ERROR: trace() only on Matrix<N, N>
```

### Const generic bounds (limited)

Currently, const generics work with integer types (`usize`, `i32`, `bool`, `char`).
Complex const expressions (e.g., `N + 1`, `N * M`) require nightly
`#![feature(generic_const_exprs)]`. For stable code, use trait-based workarounds
or accept the limitation.

---

## Type-Level Programming

### Encoding constraints as types

```rust
// Non-empty vector — can't construct empty
struct NonEmpty<T> {
    first: T,
    rest: Vec<T>,
}

impl<T> NonEmpty<T> {
    fn new(first: T) -> Self {
        NonEmpty { first, rest: Vec::new() }
    }

    fn first(&self) -> &T { &self.first }

    fn push(&mut self, item: T) {
        self.rest.push(item);
    }

    fn len(&self) -> usize { 1 + self.rest.len() }
}
```

### Zero-sized types as capability tokens

```rust
struct AdminToken;

impl AdminToken {
    // Only obtainable through authentication
    fn authenticate(password: &str) -> Option<Self> {
        if verify_password(password) { Some(AdminToken) } else { None }
    }
}

// This function can only be called with proof of admin access
fn delete_user(user_id: UserId, _proof: &AdminToken) -> Result<()> {
    // ...
}

// Usage:
let token = AdminToken::authenticate("password")
    .ok_or(Error::Unauthorized)?;
delete_user(user_id, &token)?;
// delete_user(user_id, ???); // Can't call without a token
```
