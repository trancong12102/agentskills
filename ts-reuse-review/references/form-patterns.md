# Form state reinventions

Load when the diff includes a React form component AND `react-hook-form`, `formik`, `@tanstack/react-form`, or similar is installed. Skip for non-form UI.

Every React codebase eventually reinvents form state: tracking values, errors, touched/dirty flags, submit state, async validation, field arrays. Every mature form lib handles all of this.

## The reinvention

```tsx
const [values, setValues] = useState({ email: "", password: "" });
const [errors, setErrors] = useState<Record<string, string>>({});
const [touched, setTouched] = useState<Record<string, boolean>>({});
const [submitting, setSubmitting] = useState(false);

const handleChange = (field: string) => (e: React.ChangeEvent) => {
  setValues((v) => ({ ...v, [field]: e.target.value }));
};
const handleBlur = (field: string) => () => {
  setTouched((t) => ({ ...t, [field]: true }));
  validate(field, values[field]).then((err) =>
    setErrors((e) => ({ ...e, [field]: err })),
  );
};
```

Every field = 2–3 handler props + mirror state. Scales quadratically in boilerplate.

## react-hook-form (preferred if installed)

Uncontrolled inputs by default (better perf), controlled via `Controller` when needed.

```tsx
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";

const schema = z.object({
  email: z.string().email(),
  password: z.string().min(8),
});
type FormValues = z.infer<typeof schema>;

function LoginForm() {
  const {
    register,
    handleSubmit,
    formState: { errors, isSubmitting },
  } = useForm<FormValues>({
    resolver: zodResolver(schema),
  });
  return (
    <form onSubmit={handleSubmit(onSubmit)}>
      <input {...register("email")} />
      {errors.email && <span>{errors.email.message}</span>}
      <input type="password" {...register("password")} />
      <button disabled={isSubmitting}>Submit</button>
    </form>
  );
}
```

Common reinventions it covers:

| feature                       | manual shape                              | rhf replacement                                 |
| ----------------------------- | ----------------------------------------- | ----------------------------------------------- |
| Field value tracking          | `onChange` wiring each input              | `{...register('name')}`                         |
| Touched state                 | `onBlur` handlers + Record<string, bool>  | `formState.touchedFields`                       |
| Dirty detection               | Compare snapshot to current values        | `formState.dirtyFields` + `isDirty`             |
| Validation                    | Custom per-field fn + effect              | `resolver: zodResolver(schema)` (single source) |
| Async validation              | Debounced effect + loading                | `validate` callback + `formState.isValidating`  |
| Field arrays (list of inputs) | Manual array state + splice/push helpers  | `useFieldArray({ control, name: 'items' })`     |
| Error messages                | Conditional JSX reading from errors state | `formState.errors.field?.message`               |
| Submit state                  | `isSubmitting` boolean + try/catch        | `formState.isSubmitting`                        |
| Reset after submit            | Loop clearing each field                  | `reset()` / `reset(defaultValues)`              |
| Depend field on field         | Effect watching value                     | `watch('a')` → `setValue('b', ...)`             |

## formik (preferred if installed)

Controlled inputs, more boilerplate than rhf but simpler mental model.

```tsx
<Formik
  initialValues={{ email: "" }}
  validationSchema={toFormikValidationSchema(schema)}
  onSubmit={onSubmit}
>
  {({ handleSubmit }) => (
    <form onSubmit={handleSubmit}>
      <Field name="email" />
    </form>
  )}
</Formik>
```

Honor project choice. Do not suggest migrating formik → react-hook-form (large refactor, no functional gain).

## @tanstack/react-form

Framework-agnostic form engine, adapters for React / Solid / Vue. Powerful validation with async + field-level.

```tsx
const form = useForm({
  defaultValues: { email: "" },
  validators: { onChange: schema },
});
```

Suggest only if installed — less mainstream.

## Conform (progressive enhancement forms)

For Next.js / Remix / SSR-first apps, `@conform-to/react` + `@conform-to/zod` submit as real forms, validate on server, enhance on client.

```tsx
const [form, fields] = useForm({
  lastResult,
  onValidate: ({ formData }) => parseWithZod(formData, { schema }),
});
```

Flag manual `useState` forms in Next.js server-component trees — they break progressive enhancement.

## Controlled vs uncontrolled

| manual pattern                                                            | lib-idiomatic                                           |
| ------------------------------------------------------------------------- | ------------------------------------------------------- |
| `<input value={x} onChange={e => setX(e.target.value)} />` on every field | `{...register('x')}` (rhf) — no re-render per keystroke |
| `defaultValue={x}` with manual ref reading                                | `register` handles ref + default                        |
| Custom debounced validator via `useEffect`                                | `mode: 'onBlur'` or `mode: 'onChange'` + `delayError`   |

## When NOT to suggest a form lib

- Single input, no validation, 1-off (search box, newsletter).
- Headless form state managed by a backend lib (tRPC mutation + router redirect — no client form state at all).
- Server components that submit via native `<form action={serverAction}>` — no client JS state.

Threshold: ≥3 fields OR any validation / async submit → recommend form lib.

## Priority

- **P2** — `useState` trio with validation + ≥3 fields. Form lib removes most of it.
- **P3** — simpler forms where rhf saves a few lines.
- **P1** — reinvented validation that disagrees with the zod schema elsewhere in the codebase (divergence bug).
