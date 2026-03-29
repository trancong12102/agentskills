# type-fest — Complete Utility Catalog

[type-fest](https://github.com/sindresorhus/type-fest) provides 200+ utility types
with **zero runtime cost** — all exports are `export type`, only `.d.ts` files ship.

```typescript
import type { Simplify, Merge, SetRequired } from "type-fest";
```

---

## Object Utilities

### Partial / Required Variants

| Type                    | Description                                   |
| ----------------------- | --------------------------------------------- |
| `SetOptional<T, K>`     | Make specific keys optional, rest unchanged   |
| `SetRequired<T, K>`     | Make specific keys required, rest unchanged   |
| `SetReadonly<T, K>`     | Make specific keys `readonly`                 |
| `SetNonNullable<T, K>`  | Remove `null \| undefined` from specific keys |
| `PartialDeep<T>`        | All keys and nested keys become optional      |
| `RequiredDeep<T>`       | All keys and nested keys become required      |
| `SetRequiredDeep<T, K>` | `SetRequired` applied recursively             |

**Key detail:** `PartialDeep` does NOT recurse into arrays by default. Pass
`{ recurseIntoArrays: true }` as the second type arg to opt in.

### Picking / Omitting

| Type                         | Description                                 |
| ---------------------------- | ------------------------------------------- |
| `Except<T, K>`               | Stricter `Omit` — keys must exist on `T`    |
| `PickDeep<T, Path>`          | Deep pick using dot-path keys               |
| `OmitDeep<T, Path>`          | Deep omit using dot-path keys               |
| `OmitIndexSignature<T>`      | Remove index signatures, keep explicit keys |
| `ConditionalPick<T, Cond>`   | Pick keys whose values extend `Cond`        |
| `ConditionalExcept<T, Cond>` | Exclude keys whose values extend `Cond`     |
| `ConditionalKeys<T, Cond>`   | Returns union of keys matching `Cond`       |
| `DistributedPick<T, K>`      | `Pick` that distributes over union types    |
| `DistributedOmit<T, K>`      | `Omit` that distributes over union types    |

### Merging

| Type                       | Description                                      |
| -------------------------- | ------------------------------------------------ |
| `Merge<A, B>`              | B's keys override A (unlike `&` which → `never`) |
| `MergeDeep<A, B, Opts>`    | Deep merge with configurable array strategy      |
| `MergeExclusive<A, B>`     | XOR — either A's keys or B's, never both         |
| `OverrideProperties<T, O>` | Like `Merge` but O's keys must exist on T        |
| `Spread<A, B>`             | Models `{...a, ...b}` at type level              |

### Mutability

| Type              | Description                                           |
| ----------------- | ----------------------------------------------------- |
| `Writable<T>`     | Remove all `readonly` modifiers                       |
| `WritableDeep<T>` | Remove `readonly` recursively                         |
| `ReadonlyDeep<T>` | Add `readonly` recursively (objects/arrays/maps/sets) |

### Constraint Helpers

| Type                      | Description                                |
| ------------------------- | ------------------------------------------ |
| `RequireAtLeastOne<T, K>` | At least one of `K` must be present        |
| `RequireExactlyOne<T, K>` | Exactly one of `K` must be present         |
| `RequireAllOrNone<T, K>`  | All of `K` present, or none                |
| `RequireOneOrNone<T, K>`  | At most one of `K` can be present          |
| `Exact<Shape, T>`         | Reject excess properties (beyond literals) |
| `NonEmptyObject<T>`       | Disallow empty `{}`                        |

### Key Introspection

| Type                | Description                       |
| ------------------- | --------------------------------- |
| `OptionalKeysOf<T>` | Union of optional keys            |
| `RequiredKeysOf<T>` | Union of required keys            |
| `ReadonlyKeysOf<T>` | Union of `readonly` keys          |
| `WritableKeysOf<T>` | Union of non-readonly keys        |
| `ValueOf<T>`        | Union of all value types          |
| `KeysOfUnion<T>`    | All keys across all union members |

### Deep Path Access

| Type                  | Description                                     |
| --------------------- | ----------------------------------------------- |
| `Paths<T, Opts>`      | Union of all dot-paths (max depth configurable) |
| `Get<T, Path>`        | Resolve type at a dot-prop path                 |
| `Schema<T, Value>`    | Replace all leaf values with `Value`            |
| `SetFieldType<T,P,V>` | Set type at a specific deep path                |

**Note:** `Paths` has `maxRecursionDepth` option (default 5, max 10) for circular types.

### Simplification

| Type              | Description                                |
| ----------------- | ------------------------------------------ |
| `Simplify<T>`     | Flatten intersections into readable object |
| `SimplifyDeep<T>` | Same but recursively for nested types      |

---

## Union & Conditional Utilities

| Type                      | Description                                      |
| ------------------------- | ------------------------------------------------ |
| `LiteralUnion<Lit, Base>` | Preserve autocomplete for `'a' \| 'b' \| string` |
| `UnionToIntersection<T>`  | `A \| B` → `A & B`                               |
| `TaggedUnion<Tag, Map>`   | Build discriminated union from record            |
| `ExclusifyUnion<T>`       | Add `never` markers for mutual exclusion         |

### Type Guards

| Type            | Description                  |
| --------------- | ---------------------------- |
| `IsAny<T>`      | `true` if `T` is `any`       |
| `IsNever<T>`    | `true` if `T` is `never`     |
| `IsUnknown<T>`  | `true` if `T` is `unknown`   |
| `IsEqual<A, B>` | Exact type equality          |
| `IsUnion<T>`    | `true` if `T` is a union     |
| `IsLiteral<T>`  | `true` if `T` is any literal |

### Boolean Logic

| Type          | Description           |
| ------------- | --------------------- |
| `And<A, B>`   | Type-level `&&`       |
| `Or<A, B>`    | Type-level `\|\|`     |
| `If<C, T, E>` | Ternary at type level |

---

## Branded / Opaque Types

| Type                     | Description                          |
| ------------------------ | ------------------------------------ |
| `Tagged<T, Tag, Meta>`   | Nominal tag with optional metadata   |
| `UnwrapTagged<T>`        | Strip tags, recover base type        |
| `GetTagMetadata<T, Tag>` | Retrieve metadata for a specific tag |
| `Opaque<T, Token>`       | Legacy — use `Tagged` for new code   |

`Tagged` supports **composable multi-tag** — a value can carry multiple independent tags:

```typescript
import type { Tagged } from "type-fest";
type UserId = Tagged<string, "UserId">;
type Validated = Tagged<UserId, "Validated">;
// Validated carries both "UserId" and "Validated" tags
```

---

## String / Template Literal Types

### Case Conversion

| Type                      | Example                  |
| ------------------------- | ------------------------ |
| `CamelCase<S>`            | `"foo-bar"` → `"fooBar"` |
| `KebabCase<S>`            | `"fooBar"` → `"foo-bar"` |
| `PascalCase<S>`           | `"foo-bar"` → `"FooBar"` |
| `SnakeCase<S>`            | `"fooBar"` → `"foo_bar"` |
| `ScreamingSnakeCase<S>`   | `"fooBar"` → `"FOO_BAR"` |
| `CamelCasedProperties<T>` | Apply to all object keys |
| `*Deep` variants          | Recursive application    |

### String Manipulation

| Type             | Description                  |
| ---------------- | ---------------------------- |
| `Split<S, D>`    | Type-level string split      |
| `Join<T, D>`     | Type-level tuple join        |
| `Trim<S>`        | Remove whitespace            |
| `Replace<S,F,T>` | String replace at type level |
| `Words<S>`       | Split into word boundaries   |

---

## Array / Tuple Utilities

| Type                     | Description                        |
| ------------------------ | ---------------------------------- |
| `FixedLengthArray<T, N>` | Array of exactly `N` elements      |
| `NonEmptyTuple<T>`       | Tuple with at least one element    |
| `ArrayElement<T>`        | Element type of an array           |
| `LastArrayElement<T>`    | Last element of a tuple            |
| `ArraySlice<T, S, E>`    | Slice a tuple type                 |
| `Entries<T>`             | `[key, value]` pairs for an object |

---

## Numeric Utilities

| Type                   | Description                   |
| ---------------------- | ----------------------------- |
| `Integer<N>`           | Constrain to integer literals |
| `NonNegative<N>`       | Constrain to non-negative     |
| `IntRange<Start, End>` | Union of integers in range    |
| `GreaterThan<A, B>`    | Type-level `A > B`            |
| `Sum<A, B>`            | Type-level addition           |
| `Subtract<A, B>`       | Type-level subtraction        |

**Caveat:** Numeric types work on small literals only — large numbers degrade TS
performance or produce `never`.

---

## Function Utilities

| Type                  | Description                             |
| --------------------- | --------------------------------------- |
| `AsyncReturnType<T>`  | Unwrap resolved type of async function  |
| `SetReturnType<T, R>` | Override return type                    |
| `Asyncify<T>`         | Convert sync to async (wrap in Promise) |
| `Promisable<T>`       | `T \| Promise<T>`                       |
| `Arrayable<T>`        | `T \| T[]`                              |

---

## JSON / Serialization

| Type                  | Description                            |
| --------------------- | -------------------------------------- |
| `Jsonify<T>`          | What `T` becomes after JSON round-trip |
| `Jsonifiable`         | Types safe for `JSON.stringify`        |
| `StructuredCloneable` | Types safe for `structuredClone()`     |
| `PackageJson`         | Complete typed `package.json` schema   |
| `TsConfigJson`        | Complete typed `tsconfig.json` schema  |

### `Jsonify` details

Handles: `Date` → `string`, `undefined` → dropped, class `.toJSON()` → resolved,
`interface` open-index issue → resolved. Essential when typing API response
deserialization.

---

## Improved Built-ins

| type-fest             | Replaces        | Why                                    |
| --------------------- | --------------- | -------------------------------------- |
| `Except<T, K>`        | `Omit<T, K>`    | Keys must exist on `T` (catches typos) |
| `Merge<A, B>`         | `A & B`         | Conflicting keys override, not `never` |
| `Simplify<T>`         | (none)          | Flattens intersections for readability |
| `LiteralUnion<L, B>`  | `L \| string`   | Preserves autocomplete                 |
| `ExtractStrict<T, U>` | `Extract<T, U>` | Rejects `any` types                    |
