# Immutable-update reinventions

Load when the diff contains nested spread updates ≥3 levels OR `immer` / `use-immer` / `mutative` is in installed deps.

Spread-based updates are fine for 1–2 levels. Beyond that, readability drops fast and off-by-one keys become common. Immer's `produce()` lets you mutate a draft with plain JS syntax and get back a frozen immutable result. Mutative is a faster drop-in.

## The reinvention

```ts
// updating nested.settings.notifications.email to true
return {
  ...state,
  user: {
    ...state.user,
    nested: {
      ...state.user.nested,
      settings: {
        ...state.user.nested.settings,
        notifications: {
          ...state.user.nested.settings.notifications,
          email: true,
        },
      },
    },
  },
};
```

Matched by `nested-spread-update` rule at 3 levels deep. Every misspelled key between levels silently drops data.

## immer

```ts
import { produce } from "immer";

return produce(state, (draft) => {
  draft.user.nested.settings.notifications.email = true;
});
```

Same output (structural sharing preserved), type-safe, impossible to miss a level. ~3kb gzipped.

### Common reinventions it covers

| manual                                        | immer                                                    |
| --------------------------------------------- | -------------------------------------------------------- |
| N-level spread update                         | `draft.a.b.c = val`                                      |
| Array push returning new array                | `draft.items.push(item)`                                 |
| Remove by index                               | `draft.items.splice(i, 1)`                               |
| Remove by predicate (non-trivial with spread) | `draft.items = draft.items.filter(pred)`                 |
| Insert at position                            | `draft.items.splice(i, 0, item)`                         |
| Map mutation (plain object stand-in)          | `draft.lookup[key] = value` / `delete draft.lookup[key]` |
| Reordering                                    | `draft.items.sort(cmp)`                                  |

All of the above are standard JS mutations applied to a _draft_. Immer returns a new frozen root with structural sharing.

## use-immer (React)

```tsx
import { useImmer } from 'use-immer'

const [state, updateState] = useImmer({ user: { ... } })
updateState((draft) => { draft.user.name = 'new' })
```

Preferred over `useState` for any non-trivial nested state. Suggest when the diff has `useState` with ≥3-level nested update shape.

## mutative (faster alternative)

Drop-in replacement for immer with the same API, 2–3× faster. Suggest if the project profiled immer as a hot path.

```ts
import { create } from "mutative";
const next = create(state, (draft) => {
  draft.a.b.c = 1;
});
```

## Redux Toolkit already ships immer

If `@reduxjs/toolkit` is installed, reducer `case` blocks already use immer under the hood:

```ts
createSlice({
  reducers: {
    setEmail(state, action) {
      state.user.settings.email = action.payload; // this is immer
    },
  },
});
```

So in RTK reducers, spread updates are a double reinvention — they spread inside an immer-powered handler. Flag P1.

## Zustand also ships immer (opt-in)

```ts
import { create } from 'zustand'
import { immer } from 'zustand/middleware/immer'

const useStore = create(immer((set) => ({
  user: { ... },
  setEmail: (email) => set((state) => { state.user.email = email }),
})))
```

If a zustand store does deep spreads, suggest `immer` middleware.

## When NOT to use immer

- Performance-critical hot path with small flat state — spread is faster because no proxy overhead.
- Top-level-only updates (`setState({ ...state, foo: 1 })`). Trivial spread is clear.
- Functional zealots — equally valid to use a lens library (`monocle-ts`) if already in the codebase style.

## Alternative: lens libraries

For heavy immutability work in functional codebases:

| lib          | style                                              |
| ------------ | -------------------------------------------------- |
| `monocle-ts` | Optics (lens/prism/traversal) over immutable data. |
| `ramda`      | Pluggable lens + functional pipelines.             |
| `optics-ts`  | TS-first lens library.                             |

Only suggest if the project already uses one. For the typical React/Node app, immer is the default.

## Priority

- **P2** — 3-level spread in application code.
- **P1** — spread update inside RTK reducer (defeats immer entirely).
- **P3** — 2-level spread that's clear enough to keep.
