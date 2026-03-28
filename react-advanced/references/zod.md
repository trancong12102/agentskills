# Zod v4 — Schema Validation in the TanStack Ecosystem

Zod is the schema validation layer used across this stack:

- **TanStack Router** — `validateSearch` for type-safe search params
- **TanStack Form** — field and form-level validation via Standard Schema
- **TanStack Start** — `createServerFn` validator for server function inputs

Zod v4 implements Standard Schema — no adapters or wrappers needed for any TanStack
integration. Pass schemas directly.

---

## TanStack Router: Search Params

### `.catch()` vs `.default()` — always use `.catch()` for search params

This is the most common Zod mistake in this stack:

- `.default(val)` only handles `undefined` (missing key)
- `.catch(val)` handles **any validation failure** (malformed URL values, wrong types)

```typescript
// BAD — .default() fails on ?page=abc (not undefined, just invalid)
z.number().default(1);

// GOOD — .catch() recovers from any parse failure
z.number().catch(1);

// BEST — handles both missing and invalid
z.number().default(1).catch(1);
```

### Recommended search params pattern

```typescript
const searchSchema = z.object({
  page: z.number().catch(1),
  filter: z.string().catch(""),
  sort: z.enum(["newest", "oldest", "price"]).catch("newest"),
  tab: z.enum(["all", "active", "archived"]).optional().catch(undefined),
});

export const Route = createFileRoute("/products")({
  validateSearch: searchSchema, // pass directly — no adapter needed
});
```

### When to intentionally omit `.catch()`

Omit `.catch()` when you want invalid search params to trigger the route's
`errorComponent` (e.g., a bad deep-link should show an error, not silently degrade):

```typescript
validateSearch: z.object({
  variant: z.enum(['small', 'medium', 'large']),
  // No .catch() → invalid value triggers errorComponent
}),
errorComponent: ({ error }) => <div>Invalid URL: {error.message}</div>,
```

### `z.stringbool()` for boolean search params

```typescript
const searchSchema = z.object({
  debug: z.stringbool().catch(false),
  expanded: z.stringbool().catch(false),
});
// ?debug=true → true, ?debug=1 → true, ?debug=yes → true
```

---

## TanStack Form: Validation

### Field-level validation

```typescript
<form.Field
  name="email"
  validators={{
    onChange: z.email({ error: 'Invalid email' }),
    onBlur: z.string().min(1, { error: 'Required' }),
  }}
>
  {(field) => (
    <div>
      <input
        value={field.state.value}
        onBlur={field.handleBlur}
        onChange={(e) => field.handleChange(e.target.value)}
      />
      {field.state.meta.isTouched && field.state.meta.errors.map((err, i) => (
        <span key={i} role="alert">
          {typeof err === 'string' ? err : err?.message}
        </span>
      ))}
    </div>
  )}
</form.Field>
```

### Error display gotcha — handle both string and object errors

`field.state.meta.errors` contains `string | StandardSchemaV1Issue`. Always handle both:

```typescript
const errorMessage = field.state.meta.errors
  .map((e) => (typeof e === "string" ? e : e?.message))
  .filter(Boolean)
  .join(", ");
```

### Cross-field validation with `.refine()`

```typescript
const form = useForm({
  defaultValues: { password: "", confirmPassword: "" },
  validators: {
    onSubmit: z
      .object({
        password: z.string().min(8),
        confirmPassword: z.string(),
      })
      .refine((data) => data.password === data.confirmPassword, {
        message: "Passwords must match",
        path: ["confirmPassword"],
      }),
  },
});
```

---

## `z.input` vs `z.output` — When It Matters

```typescript
const FormSchema = z.object({
  age: z.string().transform(Number),
});

type FormInput = z.input<typeof FormSchema>; // { age: string }  ← form default values
type FormOutput = z.output<typeof FormSchema>; // { age: number }  ← validated data
// z.infer<T> === z.output<T>
```

Use `z.input` for form `defaultValues` and API request types.
Use `z.output` (or `z.infer`) for validated data after parsing.

---

## Common Pitfalls

1. **Using `.default()` instead of `.catch()` for search params** — `.default()` only
   handles `undefined` (missing key). Malformed URL values like `?page=abc` still throw.
   Always use `.catch()` for search params.

2. **Not handling both error types from TanStack Form** — `field.state.meta.errors`
   contains `string | StandardSchemaV1Issue`. Rendering `err` directly may show
   `[object Object]`. Always check `typeof err === 'string' ? err : err?.message`.

3. **Using v3 error API in v4** — `.flatten()` and `.format()` are deprecated and may
   be removed. Use `z.prettifyError()` or `z.treeifyError()`.

4. **Using v3 chained format validators** — `z.string().email()` still works but is
   deprecated. Use top-level `z.email()` for tree-shaking.

5. **Forgetting `z.coerce.*` input type changed** — `z.coerce.number()` input is
   `unknown` in v4 (was `number` in v3). Use `z.coerce.number<string>()` if you need
   a specific input type for type safety.

---

## v3 → v4 Gotchas That Cause Silent Bugs

These are the changes most likely to break existing code without obvious errors:

### Error precedence reversed

```typescript
const schema = z.string({ error: () => "Schema error" });
schema.parse(12, { error: () => "Parse-time error" });
// v3: "Parse-time error" (contextual wins)
// v4: "Schema error" (schema-level wins)
```

### `z.coerce.*` input type changed to `unknown`

```typescript
const schema = z.coerce.number();
type Input = z.input<typeof schema>;
// v3: number
// v4: unknown — may break type checks silently
// Fix: z.coerce.number<string>() to specify input type
```

### `.default().optional()` now applies defaults

```typescript
const schema = z.object({ a: z.string().default("tuna").optional() });
schema.parse({});
// v3: {}            ← default NOT applied
// v4: { a: "tuna" } ← default IS applied
```

---

## v3 → v4 Key API Changes

| v3                                                | v4                                                                  |
| ------------------------------------------------- | ------------------------------------------------------------------- |
| `message`, `invalid_type_error`, `required_error` | Single `error` param                                                |
| `error.flatten()`, `error.format()`               | `z.prettifyError(error)`, `z.treeifyError(error)`                   |
| `z.string().email()`                              | `z.email()` (top-level, tree-shakable)                              |
| `z.object().strict()`                             | `z.strictObject()`                                                  |
| `z.object().passthrough()`                        | `z.looseObject()`                                                   |
| `schema.merge(other)`                             | `schema.extend(other.shape)`                                        |
| `z.lazy()` for recursive types                    | `z.object()` with getter: `get children() { return z.array(Self) }` |
| `z.union([z.literal(1), z.literal(2)])`           | `z.literal([1, 2])`                                                 |
| `schema._def`                                     | `schema._zod.def`                                                   |
