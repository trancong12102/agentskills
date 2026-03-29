# Schema — Definition, Transforms, Branded Types & Advanced Patterns

## Import from `effect`, not `@effect/schema`

`@effect/schema` is deprecated. As of Effect 3.10, Schema is in core:

```typescript
import { Schema } from "effect";
```

---

## Schema Type Parameters

`Schema<A, I, R>`:

- `A` — decoded (domain) type
- `I` — encoded (wire) type (default: same as `A`)
- `R` — context requirements (default: `never`)

**Golden rule:** `encode(decode(x)) === x` — schemas must be roundtrip-consistent.

---

## Struct Definition

```typescript
const User = Schema.Struct({
  id: Schema.Number,
  name: Schema.String,
  role: Schema.Literal("admin", "user"),
});
type User = typeof User.Type; // decoded type
type UserEncoded = typeof User.Encoded; // encoded type
```

### Extending structs

```typescript
const Timestamps = Schema.Struct({
  createdAt: Schema.Date,
  updatedAt: Schema.Date,
});

const Post = Schema.Struct({
  ...User.fields,
  ...Timestamps.fields,
  body: Schema.String,
});
```

---

## Branded Types

Makes structurally identical types opaque at the type level:

```typescript
const UserId = Schema.String.pipe(Schema.brand("UserId"));
type UserId = typeof UserId.Type; // string & Brand<"UserId">

// Only values parsed through UserId can be used where UserId is expected
const id: UserId = Schema.decodeSync(UserId)("user-123");
```

---

## Discriminated (Tagged) Unions

```typescript
const Circle = Schema.Struct({
  _tag: Schema.Literal("Circle"),
  radius: Schema.Number,
});
const Square = Schema.Struct({
  _tag: Schema.Literal("Square"),
  side: Schema.Number,
});
const Shape = Schema.Union(Circle, Square);
```

**Union ordering matters.** Members are tested in declaration order. Put the most
specific schema first:

```typescript
// WRONG: Member1 { a } matches first, silently drops "b"
Schema.Union(Member1, Member2);

// RIGHT: more specific (more fields) first
Schema.Union(Member2, Member1);
```

---

## Transformations

### `Schema.transform` — bidirectional, synchronous

```typescript
const DateFromString = Schema.transform(Schema.String, Schema.DateFromSelf, {
  decode: (s) => new Date(s),
  encode: (d) => d.toISOString(),
});
```

### `Schema.transformOrFail` — fallible transforms

```typescript
const SafeDate = Schema.transformOrFail(Schema.String, Schema.DateFromSelf, {
  decode: (s, _, ast) => {
    const d = new Date(s);
    return isNaN(d.getTime())
      ? ParseResult.fail(new ParseResult.Type(ast, s))
      : ParseResult.succeed(d);
  },
  encode: (d) => ParseResult.succeed(d.toISOString()),
});
```

**Constraint:** `transform` decode/encode cannot be async or access Effect context.
For async validation, use `transformOrFail` with Effect-returning functions.

---

## Filters (Validation Without Type Change)

```typescript
const PositiveNumber = Schema.Number.pipe(
  Schema.filter((n) => n > 0, { message: () => "must be positive" }),
);

const NonEmptyString = Schema.String.pipe(Schema.minLength(1));
```

---

## Optional with Default

```typescript
Schema.Struct({
  count: Schema.optional(Schema.Number, { default: () => 0 }),
});
// missing or undefined input -> 0
```

---

## Key Remapping

```typescript
const Person = Schema.Struct({
  age: Schema.propertySignature(Schema.NumberFromString).pipe(
    Schema.fromKey("AGE"),
  ), // external "AGE" -> internal "age"
});
```

---

## Decode / Encode Functions

| Function                     | Returns                 | Use case                         |
| ---------------------------- | ----------------------- | -------------------------------- |
| `Schema.decodeUnknownSync`   | Throws on failure       | Tests, guaranteed-valid contexts |
| `Schema.decodeUnknownEither` | `Either<A, ParseError>` | One-shot validation              |
| `Schema.decodeUnknown`       | `Effect<A, ParseError>` | Async transforms, services       |
| `Schema.encodeSync`          | Throws on failure       | Serialization                    |

---

## Advanced Patterns

### Recursive schemas

Use `Schema.suspend` to break the reference cycle:

```typescript
interface Category {
  name: string;
  subcategories: ReadonlyArray<Category>;
}

const CategorySchema: Schema.Schema<Category> = Schema.Struct({
  name: Schema.String,
  subcategories: Schema.Array(Schema.suspend(() => CategorySchema)),
});
```

### Class-based schemas

```typescript
class Person extends Schema.Class<Person>("Person")({
  name: Schema.String,
  age: Schema.NumberFromString,
}) {}

const person = new Person({ name: "Alice", age: 30 });
// Person.decode, Person.encode, Person.make available
```

### `TemplateLiteralParser` — parse + validate

```typescript
const Route = Schema.TemplateLiteralParser("/users/", Schema.NumberFromString);
Schema.decodeSync(Route)("/users/42"); // => ["/users/", 42]
Schema.encodeSync(Route)(["/users/", 42]); // => "/users/42"
```

### `pickLiteral` — narrow from a wider literal

```typescript
const Fruit = Schema.Literal("apple", "banana", "cherry");
const CitrusFree = Fruit.pipe(Schema.pickLiteral("apple", "cherry"));
```

---

## Common Pitfalls

1. **Union member ordering** — more general schema first silently drops fields from
   more specific members. Always put specific schemas first.

2. **`decodeUnknownSync` without try/catch** — it throws, not returns `Either`.

3. **Circular schemas without `Schema.suspend`** — TypeScript inference hangs.

4. **Getting the type** — use `typeof schema.Type`, not `z.infer`-style.

5. **`Schema.extend` with conflicting fields** — silently takes the last one.
   Use spread with explicit override instead.

6. **`exactOptionalPropertyTypes`** — with this TS flag, `name?: string` means
   absent-only, not `string | undefined`. Schemas may behave differently depending
   on this flag.
