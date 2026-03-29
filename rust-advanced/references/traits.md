# Trait System: Advanced Patterns

## Trait Objects vs Generics

### When to use each

**Generics (`impl Trait` / `T: Trait`):**

- Monomorphized at compile time — zero runtime overhead
- Each concrete type generates its own copy of the function
- Can cause code bloat with many concrete types
- Required when using associated types or const generics

**Trait objects (`dyn Trait`):**

- Single compiled version with runtime vtable dispatch
- Required for heterogeneous collections (`Vec<Box<dyn Render>>`)
- Required for plugin systems where types are unknown at compile time
- Small overhead: indirect function call through vtable pointer

```rust
// Generic — monomorphized, zero overhead
fn process(item: impl Serialize) { ... }

// Trait object — dynamic dispatch, single compiled version
fn process_any(items: &[Box<dyn Serialize>]) { ... }
```

---

## Associated Types vs Generic Parameters

Use **associated types** when there's exactly one natural implementation per type:

```rust
// GOOD: one Item type per Iterator implementation
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

// BAD: allows multiple impls per type, confusing API
trait Iterator<T> {
    fn next(&mut self) -> Option<T>;
}
```

Use **generic parameters** when a type can implement the trait multiple times:

```rust
// GOOD: a type can convert From many different types
trait From<T> {
    fn from(value: T) -> Self;
}
```

---

## Sealed Traits

Prevent external crates from implementing your trait — critical for maintaining
semver compatibility when adding new methods.

```rust
mod private {
    pub trait Sealed {}
}

pub trait MyApi: private::Sealed {
    fn stable_method(&self);
    // Can add methods later without breaking downstream
}

// Only your crate can implement private::Sealed
impl private::Sealed for ConcreteType {}
impl MyApi for ConcreteType {
    fn stable_method(&self) { ... }
}
```

---

## Blanket Implementations

Extend functionality to all types satisfying a bound:

```rust
// Standard library pattern: any Display type gets ToString for free
impl<T: Display> ToString for T {
    fn to_string(&self) -> String {
        format!("{}", self)
    }
}

// Your own blanket impl
trait Loggable {
    fn log(&self);
}

impl<T: Debug> Loggable for T {
    fn log(&self) {
        println!("{:?}", self);
    }
}
```

**Gotcha:** blanket impls are global — you can't override them for specific types.
Design carefully; a blanket impl can prevent downstream crates from implementing
your trait on their types.

---

## Supertraits

Require implementors to also implement another trait:

```rust
trait Printable: Debug + Display {
    fn pretty_print(&self) {
        println!("{}", self); // Can use Display methods
    }
}

// Implementors MUST implement Debug + Display + Printable
```

**Supertrait vs bound:** `trait A: B` means "implementing A requires B".
`fn foo<T: A + B>()` means "the caller must satisfy both A and B". The difference
matters for trait objects — `dyn A` where `A: B` automatically satisfies `B`.

---

## Extension Traits

Add methods to types you don't own without needing the orphan rule workaround:

```rust
trait StringExt {
    fn truncate_with_ellipsis(&self, max_len: usize) -> String;
}

impl StringExt for str {
    fn truncate_with_ellipsis(&self, max_len: usize) -> String {
        if self.len() <= max_len {
            self.to_string()
        } else {
            format!("{}...", &self[..max_len.saturating_sub(3)])
        }
    }
}
```

Convention: name the trait `{Type}Ext` (e.g., `IteratorExt`, `StringExt`).

---

## Marker Traits

Zero-method traits that communicate properties to the compiler:

| Trait   | Meaning                                  | Auto-derived? |
| ------- | ---------------------------------------- | ------------- |
| `Send`  | Safe to transfer between threads         | Yes           |
| `Sync`  | Safe to share references between threads | Yes           |
| `Sized` | Type has known size at compile time      | Yes (default) |
| `Unpin` | Type can be moved after pinning          | Yes           |
| `Copy`  | Bitwise copy is semantically valid       | Opt-in        |

**Negative implementations** for opting out:

```rust
// "My type is NOT Send" (rare — usually for raw pointer wrappers)
impl !Send for MyType {}
```

---

## Object Safety Rules

A trait can be used as `dyn Trait` only if:

1. **No methods return `Self`** — `Self` has unknown size behind `dyn`
2. **No generic type parameters on methods** — vtable can't have infinite entries
3. **All methods have a receiver** (`self`, `&self`, `&mut self`)
4. **No associated constants or types with `Self` bounds**

**Workaround for `Self`-returning methods:** add `where Self: Sized` to exclude
them from the vtable:

```rust
trait Cloneable {
    fn clone_box(&self) -> Box<dyn Cloneable>;
    fn regular_clone(&self) -> Self where Self: Sized; // excluded from dyn
}
```

---

## Higher-Ranked Trait Bounds (HRTB)

`for<'a>` means "for every possible lifetime 'a":

```rust
// The closure must work for ANY borrowed string, not a specific one
fn apply<F>(f: F, data: &[String])
where
    F: for<'a> Fn(&'a str) -> bool,
{
    for s in data {
        if f(s) { println!("{s}"); }
    }
}
```

Most commonly seen with closures accepting references. The `Fn(&str) -> bool`
sugar automatically desugars to `for<'a> Fn(&'a str) -> bool`.

You need explicit `for<'a>` when:

- The lifetime appears in a trait bound, not just a function signature
- Working with trait objects that must accept any lifetime
- Complex lifetime relationships between multiple parameters
