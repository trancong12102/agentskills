---
name: typescript-advanced
description: "Advanced TypeScript type system patterns for generics, conditional types, mapped types, template literals, and utility types. Use when implementing complex type logic, creating reusable type utilities, or enforcing type safety beyond basic annotations — discriminated unions with exhaustive checks, branded/opaque types for domain safety, satisfies vs as const decisions, NoInfer for inference control, module augmentation for third-party types, or choosing between hand-rolled types and type-fest utilities. Do not use for basic TypeScript syntax or simple type annotations."
---

# TypeScript Advanced: Patterns, Pitfalls & type-fest

This skill defines the rules, conventions, and architectural decisions for writing advanced
TypeScript. It is intentionally opinionated to prevent common type-level bugs and enforce
patterns that produce safe, maintainable code.

For detailed API documentation of TypeScript features, use other appropriate tools
(documentation lookup, web search, etc.) — this skill focuses on **when**, **why**, and
**how** to use advanced type features correctly.

## Type Safety Philosophy

### `any` vs `unknown` vs `never` — the only rule you need

| Type      | Assignable from | Assignable to          | Operations     | Use for                             |
| --------- | --------------- | ---------------------- | -------------- | ----------------------------------- |
| `any`     | anything        | anything               | all (UNSAFE)   | Never in new code                   |
| `unknown` | anything        | only `unknown` / `any` | none unnarowed | External inputs, JSON, user data    |
| `never`   | nothing         | anything               | none           | Exhaustive checks, unreachable code |

**Rule: never use `any` in new code.** Use `unknown` for external boundaries and narrow
before operating. Use `never` for exhaustiveness and impossible states.

### Prefer unions over enums

```typescript
// Avoid — numeric enums are structurally assignable to number (footgun)
enum Direction {
  Up,
  Down,
  Left,
  Right,
}
function go(d: Direction) {}
go(42); // no error — TypeScript allows any number!

// Prefer — exhaustive, tree-shakeable, no runtime artifact
type Direction = "up" | "down" | "left" | "right";
```

String enums are safer than numeric but still carry runtime overhead and import friction.
String literal unions are the default choice unless you need reverse mapping.

### `interface` vs `type` — decision table

| Scenario                                  | Use         | Why                                               |
| ----------------------------------------- | ----------- | ------------------------------------------------- |
| Object shapes, class contracts            | `interface` | Declaration merging, better error messages        |
| Unions, intersections, mapped/conditional | `type`      | Only `type` supports these                        |
| Third-party augmentation needed           | `interface` | Only interfaces support declaration merging       |
| Public API types (libraries)              | `interface` | Consumers can augment; better display in tooltips |
| Internal computed types                   | `type`      | More expressive, no accidental merging            |

---

## Discriminated Unions & Exhaustive Checks

### The `never` exhaustiveness pattern

Every `switch` / `if-else` chain on a discriminated union must handle all variants.
Use the `never` assignment to get a compile-time error when a new variant is added:

```typescript
type Result<T> =
  | { status: "ok"; data: T }
  | { status: "error"; error: Error }
  | { status: "loading" };

function handle<T>(result: Result<T>): string {
  switch (result.status) {
    case "ok":
      return JSON.stringify(result.data);
    case "error":
      return result.error.message;
    case "loading":
      return "Loading...";
    default:
      const _exhaustive: never = result;
      return _exhaustive; // compile error if a variant is unhandled
  }
}
```

### Rules for discriminated unions

- **Discriminant must be a literal type** — `string`, `number`, `boolean` literals. Wide
  types like `string` do not narrow.
- **Keep the discriminant property name consistent** across all members (`kind`, `type`,
  `status`).
- **Avoid optional discriminants** — `status?: "ok" | "error"` breaks narrowing.

---

## Branded Types — Nominal Safety in a Structural System

TypeScript is structural: `UserId` (a `string`) and `OrderId` (a `string`) are
interchangeable by default. Branded types break this at the type level with zero
runtime overhead.

### Recommended pattern: `unique symbol` brand

```typescript
declare const __brand: unique symbol;
type Brand<T, B> = T & { readonly [__brand]: B };

type UserId = Brand<string, "UserId">;
type OrderId = Brand<string, "OrderId">;

// Constructor = the single trust boundary, validate here
const toUserId = (id: string): UserId => id as UserId;
const toOrderId = (id: string): OrderId => id as OrderId;

function getUser(id: UserId) {
  /* ... */
}
getUser(toUserId("abc")); // ok
getUser(toOrderId("abc")); // ERROR — OrderId not assignable to UserId
getUser("abc"); // ERROR — string not assignable to UserId
```

### When to use branded types

- **IDs** — `UserId`, `OrderId`, `ProductId` prevent cross-assignment
- **Units** — `Meters`, `Feet`, `USD`, `EUR` prevent arithmetic mistakes
- **Validated strings** — `Email`, `URL`, `Slug` encode that validation has happened
- **Opaque tokens** — `JWTToken`, `APIKey` prevent accidental logging/display

### type-fest alternative

Use `Tagged<T, Tag>` from type-fest for multi-tag composition and metadata:

```typescript
import type { Tagged, GetTagMetadata } from "type-fest";
type UserId = Tagged<string, "UserId">;
type AdminId = Tagged<UserId, "Admin">; // composable — both tags preserved
```

---

## Modern Inference Tools

### `satisfies` — validate without widening (TS 4.9+)

```typescript
type Theme = Record<"primary" | "secondary", string | string[]>;

// Type annotation: loses specific types
const t1: Theme = { primary: "#000", secondary: ["#111", "#222"] };
t1.secondary.map((s) => s); // ERROR — string | string[] has no .map

// satisfies: validates structure, keeps specific inference
const t2 = { primary: "#000", secondary: ["#111", "#222"] } satisfies Theme;
t2.secondary.map((s) => s); // ok — inferred as string[]
```

**Use `satisfies` when:** you want config validation (catch typos in keys) but also need
autocomplete on specific values.

### `const` type parameters — generic literal inference (TS 5.0+)

```typescript
// Without const: T = string[]
function routes<T extends string[]>(r: T): T {
  return r;
}

// With const: T = readonly ["users", "posts"]
function routes<const T extends string[]>(r: T): T {
  return r;
}
const r = routes(["users", "posts"]); // readonly ["users", "posts"]
```

**Use `const` type parameters when:** building registries, config factories, or any
generic where preserving literal types at the call site matters.

### `NoInfer<T>` — control inference sources (TS 5.4+)

Prevents a parameter from contributing to type inference — it reads `T` but doesn't
influence what `T` becomes:

```typescript
function createFSM<const TState extends string>(config: {
  states: TState[];
  initial: NoInfer<TState>; // must be from states, can't introduce new values
}) {
  /* ... */
}

createFSM({ states: ["idle", "running"], initial: "idle" }); // ok
createFSM({ states: ["idle", "running"], initial: "stopped" }); // ERROR
```

**Use `NoInfer` when:** a function has multiple parameters sharing a type param, and one
should be constrained to what the others infer — not contribute new candidates.

---

## type-fest: Don't Reinvent the Wheel

[type-fest](https://github.com/sindresorhus/type-fest) provides 200+ utility types with
zero runtime cost (types-only). Always check type-fest before writing a custom utility.

```typescript
import type { Simplify, Merge, SetRequired, LiteralUnion } from "type-fest";
```

### Decision table: built-in vs type-fest

| Need                                      | Built-in              | type-fest                                |
| ----------------------------------------- | --------------------- | ---------------------------------------- |
| Make keys optional/required               | `Partial`, `Required` | `SetOptional`, `SetRequired` (per-key)   |
| Deep partial/readonly                     | —                     | `PartialDeep`, `ReadonlyDeep`            |
| Merge two types (override, not intersect) | —                     | `Merge`, `MergeDeep`                     |
| Flatten intersection for readability      | —                     | `Simplify`                               |
| String union with autocomplete + `string` | —                     | `LiteralUnion`                           |
| At-least/exactly-one constraint           | —                     | `RequireAtLeastOne`, `RequireExactlyOne` |
| Nominal/branded types                     | —                     | `Tagged`, `UnwrapTagged`                 |
| JSON round-trip type                      | —                     | `Jsonify`                                |
| Strict omit (key must exist)              | `Omit` (loose)        | `Except` (strict)                        |
| Deep dot-path access                      | —                     | `Paths`, `Get`                           |
| Exact object (reject excess props)        | —                     | `Exact`                                  |
| Pick/omit by value type                   | —                     | `ConditionalPick`, `ConditionalExcept`   |
| Package.json / tsconfig types             | —                     | `PackageJson`, `TsConfigJson`            |
| Case conversion for keys                  | —                     | `CamelCasedProperties`, etc.             |

### Most commonly needed utilities

- **`Simplify<T>`** — flattens `A & B & C` into readable `{ ...all keys }`. Use on any
  intersection that produces unreadable hover tooltips.
- **`Merge<A, B>`** — `A & B` produces `never` when keys conflict; `Merge` cleanly
  overrides. Use instead of `&` when types share key names.
- **`LiteralUnion<Literal, Base>`** — `'a' | 'b' | string` kills autocomplete;
  `LiteralUnion` preserves it. Essential for extensible string APIs.
- **`SetRequired<T, K>` / `SetOptional<T, K>`** — toggle specific keys without
  maintaining duplicate interfaces.
- **`Jsonify<T>`** — models `JSON.parse(JSON.stringify(x))`. Catches `Date` → `string`,
  `undefined` → dropped, interface open-index issues.

---

## Common Pitfalls

1. **`any` leaks silently** — one `any` propagates through assignments, generics, and
   return types. A single `any` in a utility type makes all downstream types unsound.
   Use `unknown` + narrowing instead.

2. **Excess property checks only apply to literals** — assigning through a variable
   bypasses excess property checks entirely. Don't rely on them for runtime safety.

   ```typescript
   interface Point {
     x: number;
     y: number;
   }
   const obj = { x: 1, y: 2, z: 3 };
   const p: Point = obj; // no error — z slips through
   ```

3. **Distributive conditional type on `never`** — `T extends X ? A : B` where `T` is
   `never` returns `never` (not `B`). Wrap in tuples: `[T] extends [X]`.

4. **`Omit` doesn't check key existence** — `Omit<T, "typo">` silently succeeds. Use
   `Except` from type-fest for strict key checking.

5. **Type widening with `let`** — `let x = "hello"` is `string`, not `"hello"`. Use
   `const`, `as const`, or `satisfies` to preserve literals.

6. **`&` intersection with conflicting keys** — `{ a: string } & { a: number }` makes
   `a: never`. Use `Merge` from type-fest instead.

7. **Enum numeric assignability** — `enum Foo { A, B }` allows `const x: Foo = 999`.
   Use string literal unions instead.

8. **`interface` accidental merging** — two `interface User {}` declarations in the same
   scope silently merge. Use `type` for internal types that should not be extended.

9. **`const enum` under `isolatedModules`** — esbuild, SWC, Babel all use
   `isolatedModules`. `const enum` in `.d.ts` or library code breaks these builds.

10. **Forgetting `readonly` on array parameters** — `function f(arr: string[])` allows
    mutation. Use `readonly string[]` for params you don't intend to mutate.

11. **Structural subtyping function params** — method syntax `push(x: T)` is bivariant
    (unsound). Use function property syntax `push: (x: T) => void` under
    `strictFunctionTypes` for correct variance.

12. **Reinventing type-fest utilities** — check type-fest before writing `DeepPartial`,
    `DeepReadonly`, `Merge`, branded types, or key manipulation types. The library
    handles edge cases (circular refs, readonly arrays, maps/sets) that hand-rolled
    versions miss.

---

## Reference Files

Read the relevant reference file when working with a specific pattern:

| File                                | When to read                                                    |
| ----------------------------------- | --------------------------------------------------------------- |
| `references/conditional-types.md`   | `infer`, distributive conditionals, constraining with `extends` |
| `references/mapped-types.md`        | Key remapping, filtering, template literal key manipulation     |
| `references/template-literals.md`   | String manipulation at type level, pattern matching, parsing    |
| `references/module-augmentation.md` | Declaration merging, extending third-party types, global scope  |
| `references/type-fest.md`           | Full type-fest utility catalog by category with usage examples  |
