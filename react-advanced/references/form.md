# TanStack Form v1 — Best Practices & Patterns

## Core Concepts

TanStack Form is headless — manages form state in a reactive store. Each `<form.Field>`
subscribes to its own state slice, so only the affected field re-renders on change.

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

### Schema validation

See `zod.md` for Zod + TanStack Form validation patterns and the error type gotcha
(`string | StandardSchemaV1Issue`).

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

Use `mode="array"` on the parent field — this is required or array mutation methods won't
work. Methods: `pushValue`, `removeValue`, `swapValues`, `moveValue`, `insertValue`,
`replaceValue`, `clearValues`.

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
const firstName = useStore(form.store, (state) => state.values.firstName);
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
