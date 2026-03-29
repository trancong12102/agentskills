# Unsafe Rust & FFI

## The Five Unsafe Superpowers

`unsafe` unlocks exactly these five operations — nothing else. It does **not**
disable the borrow checker, type checker, or any other safety mechanism.

1. **Dereference a raw pointer** (`*const T` / `*mut T`)
2. **Call an unsafe function or method**
3. **Access or modify a mutable static variable**
4. **Implement an unsafe trait**
5. **Access fields of a union**

---

## Mandatory SAFETY Comments

Every `unsafe` block must have a `// SAFETY:` comment explaining why the
invariants are upheld. This is not optional — it's the convention enforced by
`clippy::undocumented_unsafe_blocks`.

```rust
let ptr: *const u8 = slice.as_ptr();

// SAFETY: `ptr` is derived from a valid slice, and `offset` is within
// the slice bounds (checked by the assertion above). The resulting
// pointer is properly aligned for u8 (alignment = 1).
unsafe {
    let value = *ptr.add(offset);
}
```

### What to document

- Why the pointer is valid (non-null, aligned, points to initialized memory)
- Why aliasing rules are satisfied (no `&mut` aliases exist)
- Why the lifetime is correct (data outlives the reference)
- Why the type invariants hold (e.g., valid UTF-8 for `String::from_raw_parts`)

---

## Minimizing Unsafe Scope

```rust
// WRONG: huge unsafe block
unsafe {
    let ptr = get_pointer();
    let len = compute_length();  // safe — doesn't need unsafe
    validate(len);               // safe — doesn't need unsafe
    let slice = std::slice::from_raw_parts(ptr, len);
    process(slice);              // safe — doesn't need unsafe
}

// RIGHT: only the unsafe operation is in the unsafe block
let ptr = get_pointer();
let len = compute_length();
validate(len);

// SAFETY: ptr is valid for `len` bytes (validated above),
// properly aligned, and the data won't be mutated for 'a.
let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
process(slice);
```

---

## Safe Abstractions Over Unsafe

The pattern: unsafe internals, safe public API with enforced invariants.

```rust
pub struct SafeBuffer {
    ptr: *mut u8,
    len: usize,
    cap: usize,
}

impl SafeBuffer {
    pub fn new(cap: usize) -> Self {
        let layout = Layout::array::<u8>(cap).unwrap();
        // SAFETY: layout is non-zero (cap > 0 checked above)
        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() { handle_alloc_error(layout); }
        Self { ptr, len: 0, cap }
    }

    pub fn push(&mut self, byte: u8) {
        assert!(self.len < self.cap, "buffer full");
        // SAFETY: len < cap, so ptr.add(len) is within allocation
        unsafe { self.ptr.add(self.len).write(byte); }
        self.len += 1;
    }

    pub fn as_slice(&self) -> &[u8] {
        // SAFETY: ptr is valid for len bytes, properly aligned,
        // and no mutable references exist (we have &self)
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl Drop for SafeBuffer {
    fn drop(&mut self) {
        let layout = Layout::array::<u8>(self.cap).unwrap();
        // SAFETY: ptr was allocated with this layout in new()
        unsafe { dealloc(self.ptr, layout); }
    }
}
```

---

## transmute — Last Resort

`std::mem::transmute` reinterprets the bits of one type as another. It is the
most dangerous unsafe operation.

### Prefer alternatives

| Instead of transmute...  | Use                                            |
| ------------------------ | ---------------------------------------------- |
| Integer to enum          | Match + `TryFrom` impl                         |
| `&T` to `&U` same layout | `bytemuck::cast_ref` (with `Pod` + `Zeroable`) |
| Pointer casts            | `ptr as *const U` or `ptr.cast::<U>()`         |
| `&[u8]` to `&str`        | `std::str::from_utf8`                          |
| Extending lifetime       | **Never** — redesign instead                   |

### When you must transmute

Always use turbofish to make both types explicit:

```rust
// SAFETY: `Color` is #[repr(u8)] and value is in range 0..=2
let color = unsafe { std::mem::transmute::<u8, Color>(value) };
```

### Things transmute cannot do safely

- **Extend a lifetime** — always unsound, even if it "works"
- **Transmute between types of different sizes** — UB
- **Transmute references to break aliasing** — UB
- **Create invalid enum discriminants** — UB

---

## FFI with bindgen

### Step 1: Generate bindings

```toml
# build-dependencies
[build-dependencies]
bindgen = "0.71"
```

```rust
// build.rs
fn main() {
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))
        .expect("Couldn't write bindings");
}
```

### Step 2: Wrap in safe Rust API

```rust
// src/ffi.rs — raw bindings (generated)
#![allow(non_upper_case_globals, non_camel_case_types, dead_code)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// src/lib.rs — safe wrapper
mod ffi;

pub struct Library {
    handle: *mut ffi::lib_handle,
}

impl Library {
    pub fn new() -> Result<Self, Error> {
        // SAFETY: lib_create returns null on failure, valid handle on success
        let handle = unsafe { ffi::lib_create() };
        if handle.is_null() {
            return Err(Error::InitFailed);
        }
        Ok(Self { handle })
    }

    pub fn process(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        // SAFETY: handle is valid (checked in new()),
        // data.as_ptr() is valid for data.len() bytes
        let result = unsafe {
            ffi::lib_process(self.handle, data.as_ptr(), data.len())
        };
        if result < 0 {
            return Err(Error::ProcessFailed(result));
        }
        // ... copy result into Vec<u8>
        Ok(output)
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        // SAFETY: handle is valid and has not been freed
        unsafe { ffi::lib_destroy(self.handle); }
    }
}

// Send + Sync only if the C library is thread-safe
unsafe impl Send for Library {}
unsafe impl Sync for Library {}
```

### FFI Checklist

- [ ] All raw pointers validated before dereference
- [ ] All string conversions handle null terminators (`CStr`/`CString`)
- [ ] Ownership transfer documented (who frees what)
- [ ] Alignment requirements satisfied for all passed pointers
- [ ] `Send`/`Sync` only implemented if C library is actually thread-safe
- [ ] `Drop` impl frees all C-allocated resources
- [ ] Error codes checked on every FFI call
- [ ] No Rust panics across FFI boundary (`catch_unwind` at the boundary)
