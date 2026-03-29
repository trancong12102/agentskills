# Conditional Types — `infer`, Distributivity, Constraints

## The `infer` Keyword

`infer` captures a type within a conditional match. It only works inside the `extends`
clause of a conditional type.

```typescript
// Extract return type
type ReturnOf<T> = T extends (...args: any[]) => infer R ? R : never;

// Extract first parameter
type FirstParam<T> = T extends (first: infer P, ...rest: any[]) => any
  ? P
  : never;

// Unwrap Promise, Array, or passthrough
type Unwrap<T> =
  T extends Promise<infer V> ? V : T extends (infer Item)[] ? Item : T;
```

### Multiple `infer` in one branch

```typescript
// Extract both head and tail of a tuple
type Head<T extends readonly unknown[]> = T extends readonly [
  infer H,
  ...unknown[],
]
  ? H
  : never;

type Tail<T extends readonly unknown[]> = T extends readonly [
  unknown,
  ...infer Rest,
]
  ? Rest
  : never;

// Flip function parameter order
type Flip<T> = T extends (a: infer A, b: infer B) => infer R
  ? (a: B, b: A) => R
  : never;
```

### `infer` with `extends` constraint (TS 4.7+)

Constrain the inferred type inline to avoid a second conditional:

```typescript
// Without constraint — needs two conditionals
type GetString<T> = T extends [infer S, ...unknown[]]
  ? S extends string
    ? S
    : never
  : never;

// With constraint — single conditional
type GetString<T> = T extends [infer S extends string, ...unknown[]]
  ? S
  : never;
```

---

## Distributive Conditional Types

When the checked type is a **naked type parameter** (not wrapped in a tuple, array, or
other structure), the conditional distributes over each union member independently:

```typescript
type ToArray<T> = T extends any ? T[] : never;

type Result = ToArray<string | number>;
// Distributes:  ToArray<string> | ToArray<number>
// Result:       string[] | number[]

// Compare with non-distributive (wrapped):
type ToArraySingle<T> = [T] extends [any] ? T[] : never;
type Result2 = ToArraySingle<string | number>;
// No distribution: (string | number)[]
```

### The `never` trap

`never` is the empty union. Distributing over nothing produces nothing:

```typescript
// WRONG — returns never, not false
type IsArray<T> = T extends any[] ? true : false;
type A = IsArray<never>; // never

// FIX — wrap in tuples to prevent distribution
type IsArray<T> = [T] extends [any[]] ? true : false;
type B = IsArray<never>; // false
```

### Same fix for `IsNever`

```typescript
// WRONG — never distributes to nothing
type IsNever<T> = T extends never ? true : false;
type A = IsNever<never>; // never

// CORRECT — tuple wrapper prevents distribution
type IsNever<T> = [T] extends [never] ? true : false;
type B = IsNever<never>; // true
```

---

## Detecting Union Types

```typescript
type IsUnion<T, TCopy = T> = T extends TCopy
  ? [TCopy] extends [T]
    ? false
    : true
  : never;

type A = IsUnion<string | number>; // true
type B = IsUnion<string>; // false
```

How it works: when `T` distributes, each member checks `[TCopy] extends [T]`. If `TCopy`
is wider than the individual member (because it's the full union), the check fails →
returns `true`.

---

## Recursive Conditional Types

TypeScript supports recursive conditional types but has a depth limit (~50 levels).
Use tail-recursive patterns with accumulator types for better performance:

```typescript
// Recursive string split
type Split<
  S extends string,
  D extends string,
> = S extends `${infer Head}${D}${infer Tail}`
  ? [Head, ...Split<Tail, D>]
  : [S];

type Parts = Split<"a.b.c", ".">; // ["a", "b", "c"]

// Deep path access
type Get<T, Path extends string> = Path extends `${infer Key}.${infer Rest}`
  ? Key extends keyof T
    ? Get<T[Key], Rest>
    : never
  : Path extends keyof T
    ? T[Path]
    : never;

type Value = Get<{ a: { b: { c: number } } }, "a.b.c">; // number
```

---

## Practical Patterns

### Exhaustive event handler map

```typescript
type EventMap = {
  click: { x: number; y: number };
  keydown: { key: string };
  scroll: { offset: number };
};

type Handler<T extends keyof EventMap> = (event: EventMap[T]) => void;

// Force all events to be handled
type AllHandlers = { [K in keyof EventMap]: Handler<K> };
```

### Extract discriminated union member

```typescript
type Action =
  | { type: "add"; item: string }
  | { type: "remove"; id: number }
  | { type: "clear" };

// Extract a single member by discriminant
type AddAction = Extract<Action, { type: "add" }>;
// { type: "add"; item: string }

// Extract the payload of a specific action
type PayloadOf<A extends Action, T extends A["type"]> = Omit<
  Extract<A, { type: T }>,
  "type"
>;

type AddPayload = PayloadOf<Action, "add">; // { item: string }
```
