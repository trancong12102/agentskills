# zod deep patterns

Load when `zod` (or `@zod/mini`, `@zod/core`) is in installed deps AND the diff contains manual validation, discriminated-union parsing, or API-boundary type guards. Skip if project uses `valibot`/`yup`/`arktype` — those have their own shapes (see `project-libs.md`).

## Regex → zod string methods

zod ships battle-tested validators for strings most code re-does by hand. Prefer:

| hand-rolled             | zod method                                   | rule                         |
| ----------------------- | -------------------------------------------- | ---------------------------- |
| Email regex             | `z.string().email()`                         | `email-regex`                |
| UUID v4 regex           | `z.string().uuid()`                          | `regex-uuid`                 |
| IPv4 / IPv6 regex       | `z.string().ip()` / `.ip({ version: 'v4' })` | `regex-ipv4`                 |
| ISO 8601 datetime regex | `z.string().datetime({ offset: true })`      | `regex-iso-datetime`         |
| ISO 8601 date-only      | `z.string().date()` (zod v3.23+)             | `regex-iso-datetime` variant |
| ISO 8601 time-only      | `z.string().time()` (zod v3.23+)             |                              |
| CIDR `192.168.0.0/24`   | `z.string().cidr()`                          |                              |
| CUID / CUID2            | `z.string().cuid()` / `.cuid2()`             |                              |
| ULID                    | `z.string().ulid()`                          |                              |
| nanoid                  | `z.string().nanoid()`                        |                              |
| base64                  | `z.string().base64()`                        |                              |
| `new URL(s)` try/catch  | `z.string().url()`                           | `url-validate-try`           |
| emoji regex             | `z.string().emoji()`                         |                              |
| min/max length check    | `z.string().min(N).max(M)`                   |                              |

All of these return `ZodString` — chain as needed: `z.string().email().min(5)`.

## Number / bigint

| hand-rolled                         | zod method                                                                            |
| ----------------------------------- | ------------------------------------------------------------------------------------- |
| `Number.isInteger(x) && x >= 0`     | `z.number().int().nonnegative()`                                                      |
| `x > 0 && x < 100`                  | `z.number().gt(0).lt(100)`                                                            |
| `x % 5 === 0`                       | `z.number().multipleOf(5)`                                                            |
| `typeof x === 'bigint' && x > 0n`   | `z.bigint().positive()`                                                               |
| Parse numeric string from form data | `z.coerce.number().int()`                                                             |
| Number string with fixed precision  | `z.string().regex(/^\d+\.\d{2}$/)` → keep as string; or `z.number().multipleOf(0.01)` |

## Enum / literal

| hand-rolled                                     | zod method                |
| ----------------------------------------------- | ------------------------- |
| `['a', 'b', 'c'].includes(x)`                   | `z.enum(['a', 'b', 'c'])` |
| `x === 'a' \|\| x === 'b'`                      | `z.enum(['a', 'b'])`      |
| `Object.values(MyEnum).includes(x)` for TS enum | `z.nativeEnum(MyEnum)`    |

## Object shape

| hand-rolled                                              | zod method                                                                   |
| -------------------------------------------------------- | ---------------------------------------------------------------------------- |
| Manual type guard `x is User`                            | `UserSchema.safeParse(x).success` + `type User = z.infer<typeof UserSchema>` |
| `typeof x.foo === 'string' && typeof x.bar === 'number'` | `z.object({ foo: z.string(), bar: z.number() })`                             |
| Unknown-key rejection                                    | `.strict()` on object schema                                                 |
| Pass-through unknown keys                                | `.passthrough()`                                                             |
| Pick a subset of fields                                  | `UserSchema.pick({ id: true, email: true })`                                 |
| Omit                                                     | `UserSchema.omit({ password: true })`                                        |
| Partial (all optional)                                   | `UserSchema.partial()`                                                       |
| Deep partial                                             | `UserSchema.deepPartial()` (v3) or manual for v4                             |
| Required (all non-optional)                              | `UserSchema.required()`                                                      |
| Merge two schemas                                        | `A.merge(B)` or `A.and(B)`                                                   |
| Extend with more fields                                  | `A.extend({ extra: z.string() })`                                            |

## Discriminated unions

Flagged by `zod-discriminated-union`. Canonical shape:

```ts
const Event = z.discriminatedUnion("type", [
  z.object({ type: z.literal("click"), x: z.number(), y: z.number() }),
  z.object({ type: z.literal("key"), key: z.string() }),
  z.object({
    type: z.literal("resize"),
    width: z.number(),
    height: z.number(),
  }),
]);

type Event = z.infer<typeof Event>; // { type: 'click'; x: number; y: number } | ...
```

Benefits over manual `if/else`: one schema, exhaustive at type level, one parse call, error accumulation.

## Transform + refine + brand

| feature        | use for                                         | example                                                                              |
| -------------- | ----------------------------------------------- | ------------------------------------------------------------------------------------ |
| `.transform`   | parse + convert (e.g., string → Date)           | `z.string().datetime().transform(s => new Date(s))`                                  |
| `.refine`      | custom validation with typed error              | `z.string().refine(s => /^\d+$/.test(s), 'must be digits')`                          |
| `.superRefine` | multi-field validation                          | `schema.superRefine((data, ctx) => { if (data.a > data.b) ctx.addIssue(...) })`      |
| `.brand`       | nominal typing (`Email` distinct from `string`) | `z.string().email().brand<'Email'>()` → `z.infer` produces `string & BRAND<'Email'>` |
| `.default`     | fallback when missing                           | `z.string().default('anonymous')`                                                    |
| `.catch`       | fallback on parse error                         | `z.number().catch(0)`                                                                |
| `.preprocess`  | coerce before parse                             | `z.preprocess(v => typeof v === 'string' ? Number(v) : v, z.number())`               |
| `z.coerce.*`   | built-in coerce                                 | `z.coerce.date()` for form-submitted datetimes                                       |

## API boundary pattern

The high-ROI zod use case: any data crossing a trust boundary (HTTP, fs, IPC, storage) gets `.parse` or `.safeParse` once.

```ts
const UserResponse = z.object({
  user: z.object({ id: z.string().uuid(), email: z.string().email() }),
});

const res = await fetch(url);
const data: unknown = await res.json();
const parsed = UserResponse.safeParse(data);
if (!parsed.success) throw new ValidationError(parsed.error);
return parsed.data.user; // fully typed
```

Flag any `await res.json()` that writes to a typed variable without `.parse`/`.safeParse` between them. Priority P2 (likely bug hiding).

## zod-to-\* codegen

If installed, these shortcut further reinvention:

| lib                     | purpose                                                       |
| ----------------------- | ------------------------------------------------------------- |
| `zod-to-json-schema`    | Generate JSON Schema for OpenAPI docs. Don't hand-write both. |
| `zod-to-openapi`        | Directly emit OpenAPI schemas from zod.                       |
| `hono-openapi` / `trpc` | Use zod schemas as the single source of truth for server IO.  |
| `drizzle-zod`           | Generate zod schemas from drizzle table definitions.          |
| `@hookform/resolvers`   | Plug zod into react-hook-form validation.                     |

## When NOT to use zod

- Internal helpers that accept values from inside the same module. TypeScript types are enough.
- Hot paths where every call goes through zod. zod is ~µs per call; fine for HTTP but not for per-frame state reducers. Cache the schema; do not construct in the hot path.
- Simple typeof checks on primitives — `typeof x === 'string'` is fine if you own both sides of the call.
