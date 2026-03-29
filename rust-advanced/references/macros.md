# Macros: Declarative & Procedural

## Declarative Macros (macro_rules!)

### Fragment specifiers

| Specifier     | Matches               | Example              |
| ------------- | --------------------- | -------------------- |
| `$e:expr`     | Any expression        | `42`, `x + 1`, `f()` |
| `$t:ty`       | A type                | `i32`, `Vec<String>` |
| `$i:ident`    | An identifier         | `foo`, `my_var`      |
| `$p:path`     | A path                | `std::io::Error`     |
| `$l:lifetime` | A lifetime            | `'a`, `'static`      |
| `$b:block`    | A block expression    | `{ x + 1 }`          |
| `$s:stmt`     | A statement           | `let x = 5;`         |
| `$tt:tt`      | Any single token tree | Most flexible        |

### Variadic patterns

```rust
macro_rules! vec_of {
    // Match zero or more comma-separated expressions
    ($($x:expr),* $(,)?) => {{
        let mut v = Vec::new();
        $(v.push($x);)*
        v
    }};
}

let v = vec_of![1, 2, 3,]; // trailing comma ok
```

### Repetition operators

| Operator  | Meaning      |
| --------- | ------------ |
| `$(...)*` | Zero or more |
| `$(...)+` | One or more  |
| `$(...)?` | Zero or one  |

### Common patterns

**Builder-like API:**

```rust
macro_rules! map {
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut m = std::collections::HashMap::new();
        $(m.insert($key, $value);)*
        m
    }};
}

let scores = map! {
    "Alice" => 100,
    "Bob" => 85,
};
```

**Enum dispatch:**

```rust
macro_rules! dispatch {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match $self {
            Shape::Circle(inner) => inner.$method($($arg),*),
            Shape::Rect(inner) => inner.$method($($arg),*),
            Shape::Triangle(inner) => inner.$method($($arg),*),
        }
    };
}
```

### Hygiene

`macro_rules!` macros are **partially hygienic**:

- Identifiers created inside the macro don't leak into the caller's scope
- But `$tt:tt` matchers can capture external names
- Items (structs, functions) are NOT hygienic — they're always visible

**Gotcha:** if you need a unique identifier inside a macro, there's no built-in
`gensym`. Use a naming convention like `__macro_internal_` prefix, or switch to
a proc macro for full control.

---

## Procedural Macros

Proc macros are Rust functions that operate on `TokenStream` at compile time.
They live in a dedicated crate with `proc-macro = true`.

### Three kinds

| Kind            | Syntax               | Use case                    |
| --------------- | -------------------- | --------------------------- |
| Derive macro    | `#[derive(MyMacro)]` | Generate impl blocks        |
| Attribute macro | `#[my_attr]`         | Transform annotated items   |
| Function-like   | `my_macro!(...)`     | Full control over expansion |

### Crate setup

```toml
# Cargo.toml for the proc macro crate
[lib]
proc-macro = true

[dependencies]
syn = { version = "2", features = ["full"] }
quote = "1"
proc-macro2 = "1"
```

### Derive macro example

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(MyDebug)]
pub fn my_debug_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, stringify!(#name))
            }
        }
    };

    TokenStream::from(expanded)
}
```

### Attribute macro example

```rust
#[proc_macro_attribute]
pub fn log_calls(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemFn);
    let name = &input.sig.ident;
    let block = &input.block;
    let sig = &input.sig;
    let vis = &input.vis;

    let expanded = quote! {
        #vis #sig {
            println!("Entering {}", stringify!(#name));
            let result = (|| #block)();
            println!("Exiting {}", stringify!(#name));
            result
        }
    };

    TokenStream::from(expanded)
}
```

### Key crates

| Crate         | Purpose                                              |
| ------------- | ---------------------------------------------------- |
| `syn`         | Parse Rust syntax from TokenStream                   |
| `quote`       | Generate TokenStream via quasi-quoting (`quote!`)    |
| `proc-macro2` | TokenStream2 — testable outside proc macro context   |
| `darling`     | Declarative attribute parsing (like serde for attrs) |

### Testing proc macros

Use `proc-macro2::TokenStream` for unit tests (doesn't require the compiler):

```rust
#[test]
fn test_expansion() {
    let input: proc_macro2::TokenStream = quote! {
        struct Foo { x: i32 }
    };

    let output = my_macro_impl(input);

    let expected = quote! {
        impl Debug for Foo { ... }
    };

    assert_eq!(output.to_string(), expected.to_string());
}
```

For integration tests, use `trybuild` to test compile-pass and compile-fail:

```rust
#[test]
fn compile_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/pass/*.rs");
    t.compile_fail("tests/fail/*.rs");
}
```

---

## When to Use Which

| Need                                   | Use                      |
| -------------------------------------- | ------------------------ |
| Simple code generation / repetition    | `macro_rules!`           |
| Derive impls from struct/enum shape    | Derive proc macro        |
| Transform function/struct declarations | Attribute proc macro     |
| DSL with custom syntax                 | Function-like proc macro |
| Conditional compilation                | `cfg!` / `#[cfg(...)]`   |

**Rule of thumb:** start with `macro_rules!`. Only reach for proc macros when
you need to inspect the structure of the input (field names, types, attributes).
