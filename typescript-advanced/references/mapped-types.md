# Mapped Types — Key Remapping, Filtering, Deep Recursion

## Key Remapping with `as` (TS 4.1+)

The `as` clause in mapped types transforms or filters keys during iteration.

```typescript
// Prefix all keys
type Prefixed<T> = {
  [K in keyof T as `get${Capitalize<string & K>}`]: () => T[K];
};

interface User {
  name: string;
  age: number;
}
type UserGetters = Prefixed<User>;
// { getName: () => string; getAge: () => number }
```

### Why `string & K`?

`keyof T` returns `string | number | symbol`. Template literals only accept `string`.
The intersection `string & K` filters to string keys only.

---

## Filtering Keys

Return `never` from `as` to exclude a key from the result:

```typescript
// Keep only string-valued properties
type OnlyStrings<T> = {
  [K in keyof T as T[K] extends string ? K : never]: T[K];
};

// Keep only methods
type MethodsOf<T> = {
  [K in keyof T as T[K] extends (...args: any[]) => any ? K : never]: T[K];
};

// Remove keys starting with underscore
type PublicKeys<T> = {
  [K in keyof T as K extends `_${string}` ? never : K]: T[K];
};
```

---

## Generating Event Handlers

Combine key remapping + template literals for automatic event wiring:

```typescript
type EventHandlers<T> = {
  [K in keyof T as `on${Capitalize<string & K>}Change`]?: (val: T[K]) => void;
};

type Form = { name: string; email: string; age: number };
type FormHandlers = EventHandlers<Form>;
// { onNameChange?: (val: string) => void; onEmailChange?: ... }
```

---

## Deep Recursive Mapped Types

### DeepReadonly

```typescript
type DeepReadonly<T> = T extends (...args: any[]) => any
  ? T // don't recurse into functions
  : T extends object
    ? { readonly [K in keyof T]: DeepReadonly<T[K]> }
    : T;
```

### DeepPartial

```typescript
type DeepPartial<T> = T extends object
  ? { [K in keyof T]?: DeepPartial<T[K]> }
  : T;
```

**Gotcha:** hand-rolled versions miss edge cases. `ReadonlyDeep` and `PartialDeep`
from type-fest handle `Map`, `Set`, `ReadonlyArray`, circular references, and class
instances correctly. Prefer type-fest for production code.

---

## Mapped Type Modifiers

### Adding/removing `readonly` and `?`

```typescript
// Add readonly to all keys
type Frozen<T> = { readonly [K in keyof T]: T[K] };

// Remove readonly from all keys
type Mutable<T> = { -readonly [K in keyof T]: T[K] };

// Make all keys required
type AllRequired<T> = { [K in keyof T]-?: T[K] };

// Combine: mutable + required
type Concrete<T> = { -readonly [K in keyof T]-?: T[K] };
```

The `-` prefix removes the modifier. The `+` prefix (default) adds it.

---

## Homomorphic Mapped Types

A mapped type that iterates `keyof T` (where `T` is a type parameter) preserves the
modifiers (`readonly`, `?`) of the original type. This is called a homomorphic mapped
type:

```typescript
type Homomorphic<T> = { [K in keyof T]: T[K] }; // preserves readonly/?
type NonHomomorphic = { [K in "a" | "b"]: string }; // does NOT preserve
```

This matters when chaining mapped types — modifiers flow through homomorphic types
automatically.

---

## Combining Mapped Types with Conditional Types

### Type-safe pick by value type

```typescript
type PickByValue<T, V> = {
  [K in keyof T as T[K] extends V ? K : never]: T[K];
};

interface Mixed {
  name: string;
  age: number;
  active: boolean;
  email: string;
}

type StringProps = PickByValue<Mixed, string>;
// { name: string; email: string }
```

### Make specific keys required, rest unchanged

```typescript
type WithRequired<T, K extends keyof T> = Omit<T, K> & Required<Pick<T, K>>;

// type-fest alternative: SetRequired<T, K> — handles display better via Simplify
```

### Record with known + dynamic keys

```typescript
// Explicit known keys + index signature for extras
type Config = {
  [key: string]: unknown;
  host: string;
  port: number;
};

// Alternative: intersection pattern
type Config = { host: string; port: number } & Record<string, unknown>;
```
