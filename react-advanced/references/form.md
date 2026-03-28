# TanStack Form v1 — Best Practices & Patterns

## Core Concepts

TanStack Form is headless — manages form state (values, errors, touched, dirty, submitting)
and validation in a reactive store. Each `<form.Field>` subscribes to its own state slice,
so only the affected field re-renders on change.

```typescript
import { useForm } from '@tanstack/react-form'

const form = useForm({
  defaultValues: { email: '', age: 0 },
  onSubmit: async ({ value }) => {
    await saveToServer(value)
  },
})

<form.Field name="email">
  {(field) => (
    <input
      value={field.state.value}
      onChange={(e) => field.handleChange(e.target.value)}
      onBlur={() => field.handleBlur()}
    />
  )}
</form.Field>
```

Field state: `field.state.value`, `field.state.meta.errors`, `field.state.meta.isValid`,
`isTouched`, `isDirty`, `isValidating`.

---

## Validation

### Inline validators per field

```typescript
<form.Field
  name="age"
  validators={{
    onChange: ({ value }) => value < 13 ? 'Must be 13+' : undefined,
    onChangeAsyncDebounceMs: 500,
    onChangeAsync: async ({ value }) => {
      const taken = await checkUsernameAvailable(value)
      return taken ? 'Username taken' : undefined
    },
  }}
>
```

### Schema validation (Zod, Valibot, ArkType via Standard Schema)

```typescript
import { z } from 'zod'

// Form-level
const form = useForm({
  defaultValues: { email: '', password: '' },
  validators: {
    onChange: z.object({
      email: z.string().email(),
      password: z.string().min(8),
    }),
  },
})

// Field-level
<form.Field
  name="age"
  validators={{ onChange: z.number().gte(13, 'Must be 13+') }}
>
```

### Cross-field validation

Use `onChangeListenTo` to re-run when another field changes:
```typescript
<form.Field
  name="confirm_password"
  validators={{
    onChangeListenTo: ['password'],
    onChange: ({ value, fieldApi }) =>
      value !== fieldApi.form.getFieldValue('password')
        ? 'Passwords do not match'
        : undefined,
  }}
>
```

---

## Field Arrays

Use `mode="array"` on the parent field. Array mutation methods: `pushValue`, `removeValue`,
`swapValues`, `moveValue`, `insertValue`, `replaceValue`, `clearValues`.

```typescript
<form.Field name="people" mode="array">
  {(field) => (
    <div>
      {field.state.value.map((_, index) => (
        <form.Field key={index} name={`people[${index}].name`}>
          {(subField) => (
            <input
              value={subField.state.value}
              onChange={(e) => subField.handleChange(e.target.value)}
            />
          )}
        </form.Field>
      ))}
      <button type="button" onClick={() => field.pushValue({ name: '', age: 0 })}>
        Add
      </button>
    </div>
  )}
</form.Field>
```

---

## Server-Side Validation (TanStack Start)

```typescript
import { createServerValidate } from '@tanstack/react-form/server'

const serverValidate = createServerValidate({
  validators: {
    onSubmitAsync: async ({ value }) => {
      const errors = await validateOnDB(value)
      return errors ? {
        form: 'Invalid data',
        fields: {
          age: 'Must be 13+',
          'socials[0].url': 'URL does not exist',
          'details.email': 'Required',
        },
      } : null
    },
  },
})
```

---

## Performance

### form.Subscribe with selector

The core optimization primitive — only re-renders when selected slice changes:
```typescript
<form.Subscribe selector={(state) => [state.canSubmit, state.isSubmitting]}>
  {([canSubmit, isSubmitting]) => (
    <button disabled={!canSubmit}>
      {isSubmitting ? 'Saving...' : 'Save'}
    </button>
  )}
</form.Subscribe>
```

### useStore for imperative access

```typescript
const firstName = useStore(form.store, (state) => state.values.firstName)
```

Each `form.Field` is already optimized — subscribes only to its own state slice.

---

## Common Pitfalls

1. **Recreating defaultValues every render** — define outside component or in `useMemo`.
2. **Missing e.stopPropagation() on nested forms** — include both `preventDefault()` and
   `stopPropagation()` in onSubmit.
3. **form.Subscribe without selector** — re-renders on every state change.
4. **Async validation without debounce** — always pair `onChangeAsync` with
   `onChangeAsyncDebounceMs`.
5. **Forgetting handleBlur** — touch state and onBlur validators depend on it.
