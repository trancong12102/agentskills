# Template Literal Types — String Manipulation, Pattern Matching, Parsing

## Generating Unions from Unions

Multiple union positions in a template literal produce the Cartesian product:

```typescript
type Side = "top" | "bottom";
type Axis = "left" | "right";
type Corner = `${Side}-${Axis}`;
// "top-left" | "top-right" | "bottom-left" | "bottom-right"

type HTTPMethod = "GET" | "POST" | "PUT" | "DELETE";
type Endpoint = "/users" | "/posts";
type Route = `${HTTPMethod} ${Endpoint}`;
// "GET /users" | "GET /posts" | "POST /users" | ... (8 total)
```

---

## String Parsing with `infer`

### Basic extraction

```typescript
type ParseRoute<T extends string> = T extends `${infer Method} ${infer Path}`
  ? { method: Method; path: Path }
  : never;

type R = ParseRoute<"GET /users">; // { method: "GET"; path: "/users" }
```

### Path parameter extraction

```typescript
type ExtractParams<T extends string> =
  T extends `${string}:${infer Param}/${infer Rest}`
    ? Param | ExtractParams<`/${Rest}`>
    : T extends `${string}:${infer Param}`
      ? Param
      : never;

type Params = ExtractParams<"/users/:userId/posts/:postId">;
// "userId" | "postId"
```

### Recursive string operations

```typescript
// Split string into tuple
type Split<
  S extends string,
  D extends string,
> = S extends `${infer Head}${D}${infer Tail}`
  ? [Head, ...Split<Tail, D>]
  : [S];

type Parts = Split<"a.b.c", ".">; // ["a", "b", "c"]

// Join tuple into string
type Join<T extends string[], D extends string> = T extends []
  ? ""
  : T extends [infer H extends string]
    ? H
    : T extends [infer H extends string, ...infer Rest extends string[]]
      ? `${H}${D}${Join<Rest, D>}`
      : never;

type Joined = Join<["a", "b", "c"], ".">; // "a.b.c"
```

---

## Built-in String Intrinsics

These four are compiler intrinsics — they cannot be user-defined:

| Type              | Example               |
| ----------------- | --------------------- |
| `Uppercase<S>`    | `"hello"` → `"HELLO"` |
| `Lowercase<S>`    | `"HELLO"` → `"hello"` |
| `Capitalize<S>`   | `"hello"` → `"Hello"` |
| `Uncapitalize<S>` | `"Hello"` → `"hello"` |

Combine with mapped types for key transformation:

```typescript
type Getters<T> = {
  [K in keyof T as `get${Capitalize<string & K>}`]: () => T[K];
};
```

---

## Type-Safe Event Systems

### Property change events

```typescript
type PropEventSource<T> = {
  on<K extends string & keyof T>(
    eventName: `${K}Changed`,
    callback: (newValue: T[K]) => void,
  ): void;
};

declare const user: PropEventSource<{ name: string; age: number }>;
user.on("nameChanged", (val) => {}); // val: string
user.on("ageChanged", (val) => {}); // val: number
user.on("fooChanged", () => {}); // ERROR — "foo" not in keyof T
```

### Scoped event namespacing

```typescript
type ScopedEvent<
  Scope extends string,
  Events extends string,
> = `${Scope}:${Events}`;

type AuthEvents = ScopedEvent<"auth", "login" | "logout" | "refresh">;
// "auth:login" | "auth:logout" | "auth:refresh"
```

---

## Deep Path Types

### Generate all valid dot-paths

```typescript
type Paths<T, MaxDepth extends number = 5> = _Paths<T, MaxDepth, []>;

type _Paths<
  T,
  MaxDepth extends number,
  Depth extends unknown[],
> = Depth["length"] extends MaxDepth
  ? never
  : T extends object
    ? {
        [K in keyof T & string]:
          | K
          | `${K}.${_Paths<T[K], MaxDepth, [...Depth, unknown]>}`;
      }[keyof T & string]
    : never;

// type-fest provides Paths<T> and Get<T, Path> with proper depth limits
// and circular-reference handling. Use those for production code.
```

---

## CSS / Design Token Types

Template literals excel at modeling structured string domains:

```typescript
type CSSLength = `${number}${"px" | "rem" | "em" | "vh" | "vw" | "%"}`;
type CSSColor = `#${string}` | `rgb(${string})` | `rgba(${string})`;

// Design token paths
type Token = `colors.${string}` | `spacing.${string}` | `fonts.${string}`;

// Responsive props
type Breakpoint = "sm" | "md" | "lg" | "xl";
type ResponsiveProp<T extends string> = T | `${Breakpoint}:${T}`;
```

---

## type-fest String Utilities

Before writing string manipulation types, check type-fest:

| type-fest Type            | Does                               |
| ------------------------- | ---------------------------------- |
| `CamelCase<S>`            | `"foo-bar"` → `"fooBar"`           |
| `KebabCase<S>`            | `"fooBar"` → `"foo-bar"`           |
| `PascalCase<S>`           | `"foo-bar"` → `"FooBar"`           |
| `SnakeCase<S>`            | `"fooBar"` → `"foo_bar"`           |
| `ScreamingSnakeCase<S>`   | `"fooBar"` → `"FOO_BAR"`           |
| `CamelCasedProperties<T>` | Apply CamelCase to all object keys |
| `Split<S, D>`             | Type-level string split            |
| `Join<T, D>`              | Type-level tuple join              |
| `Trim<S>`                 | Remove leading/trailing whitespace |
| `Replace<S, From, To>`    | String replace at type level       |
| `Words<S>`                | Split into word boundaries         |

All have `Deep` variants for recursive application on nested objects.
