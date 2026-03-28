# XState v5 — Best Practices & Patterns

## Core: setup() + createMachine()

Always use `setup().createMachine()` for full TypeScript inference:

```typescript
import { setup, fromPromise, assign } from 'xstate'

const machine = setup({
  types: {
    context: {} as { count: number },
    events: {} as { type: 'inc' } | { type: 'dec' },
  },
  actions: {
    increment: assign({ count: ({ context }) => context.count + 1 }),
  },
  guards: {
    isPositive: ({ context }) => context.count > 0,
  },
  actors: {
    fetchData: fromPromise(async ({ input }: { input: { id: string } }) => {
      return fetch(`/api/${input.id}`).then(r => r.json())
    }),
  },
}).createMachine({
  context: { count: 0 },
  on: {
    inc: { actions: 'increment' },
    dec: { guard: 'isPositive', actions: 'decrement' },
  },
})
```

### Actor types

- `createMachine(...)` — full state machine / statechart
- `fromPromise(async fn)` — one-shot async operations
- `fromCallback(fn)` — subscriptions, DOM events, WebSockets
- `fromObservable(fn)` — RxJS streams

### Guards combinators

```typescript
import { and, or, not } from 'xstate'

guards: {
  isFormValid: and(['isValidEmail', 'isValidPassword', 'hasAcceptedTerms']),
}
```

### enqueueActions for conditional action sequencing

```typescript
entry: enqueueActions(({ context, enqueue, check }) => {
  enqueue.assign({ count: context.count + 1 })
  if (check('someGuard')) {
    enqueue.sendTo('childActor', { type: 'UPDATE' })
  }
})
```

---

## React Integration

### useMachine / useActor (identical in v5)

```typescript
import { useMachine } from '@xstate/react'

function Counter() {
  const [snapshot, send] = useMachine(counterMachine)
  // snapshot.value — current state
  // snapshot.context — extended state
  // snapshot.matches('idle') — type-safe state check
  return <button onClick={() => send({ type: 'INCREMENT' })}>{snapshot.context.count}</button>
}
```

### Where to define machines

Always at **module level**, outside React components:
```typescript
// machines/counterMachine.ts
export const counterMachine = setup({ ... }).createMachine({ ... })

// components/Counter.tsx
import { counterMachine } from '../machines/counterMachine'
function Counter() {
  const [state, send] = useMachine(counterMachine)
}
```

Pass runtime values as `input`, not by building machines inside components.

---

## XState as useEffect Replacement

Machines eliminate race conditions by making side effects lifecycle-bound:

```typescript
const userMachine = setup({
  actors: {
    fetchUser: fromPromise(async ({ input }: { input: { userId: string } }) => {
      const res = await fetch(`/api/users/${input.userId}`)
      return res.json()
    }),
  },
}).createMachine({
  initial: 'idle',
  context: { user: undefined, error: undefined },
  states: {
    idle: { on: { FETCH: 'loading' } },
    loading: {
      invoke: {
        src: 'fetchUser',
        input: ({ context }) => ({ userId: context.userId }),
        onDone: {
          target: 'success',
          actions: assign({ user: ({ event }) => event.output }),
        },
        onError: {
          target: 'failure',
          actions: assign({ error: ({ event }) => event.error }),
        },
      },
      on: { CANCEL: 'idle' },  // auto-cancels invoked actor on transition
    },
    success: {},
    failure: { on: { RETRY: 'loading' } },
  },
})
```

When the machine leaves `loading` (e.g., via `CANCEL`), the invoked actor is automatically
stopped. No AbortController needed.

---

## Patterns

### Authentication flow

```typescript
const authMachine = setup({
  actors: {
    checkAuth: fromPromise(async () => { /* check stored tokens */ }),
    loginUser: fromPromise(async ({ input }) => { /* login API */ }),
  },
}).createMachine({
  id: 'auth',
  initial: 'initializing',
  context: { user: null, error: null },
  states: {
    initializing: {
      invoke: {
        src: 'checkAuth',
        onDone: { target: 'authenticated', actions: assign({ user: ({ event }) => event.output }) },
        onError: 'unauthenticated',
      },
    },
    unauthenticated: {
      initial: 'idle',
      states: {
        idle: { on: { LOGIN: 'loading' } },
        loading: {
          invoke: {
            src: 'loginUser',
            input: ({ event }) => ({ email: event.email, password: event.password }),
            onDone: { target: '#auth.authenticated', actions: assign({ user: ({ event }) => event.output }) },
            onError: { target: 'idle', actions: assign({ error: ({ event }) => event.error.message }) },
          },
        },
      },
    },
    authenticated: {
      on: { LOGOUT: { target: 'unauthenticated', actions: assign({ user: null }) } },
    },
  },
})
```

### Multi-step wizard

Steps are states. Context accumulates form data:
```typescript
const wizardMachine = setup({ ... }).createMachine({
  initial: 'step1',
  context: { step1: null, step2: null },
  states: {
    step1: { on: { NEXT: { target: 'step2', actions: assign({ step1: ({ event }) => event.data }) } } },
    step2: { on: { BACK: 'step1', NEXT: { target: 'submitting', actions: assign({ step2: ({ event }) => event.data }) } } },
    submitting: { invoke: { src: 'submitForm', onDone: 'success', onError: 'step2' } },
    success: { type: 'final' },
  },
})
```

### Parallel states

For independent concerns that need simultaneous tracking:
```typescript
const playerMachine = createMachine({
  type: 'parallel',
  states: {
    track: { initial: 'paused', states: { paused: { on: { PLAY: 'playing' } }, playing: { on: { STOP: 'paused' } } } },
    volume: { initial: 'normal', states: { normal: { on: { MUTE: 'muted' } }, muted: { on: { UNMUTE: 'normal' } } } },
  },
})
// state.value = { track: 'playing', volume: 'muted' }
```

---

## Global State — createActorContext

The idiomatic solution for app-wide state:

```typescript
import { createActorContext } from '@xstate/react'

export const AuthContext = createActorContext(authMachine)

// App.tsx
<AuthContext.Provider>
  <Router />
</AuthContext.Provider>

// Any child
const user = AuthContext.useSelector((state) => state.context.user)
const actorRef = AuthContext.useActorRef()
actorRef.send({ type: 'LOGOUT' })
```

### Machine composition: invoke vs spawnChild

- **`invoke`** — tied to state lifetime, auto-cancelled on exit (request/response)
- **`spawnChild`** — independent of state (long-lived: WebSockets, timers, workers)

---

## Bridging XState with React Query

Never call React hooks inside a machine. The correct bridge pattern:

```typescript
function CheckoutFlow() {
  const [snapshot, send] = useMachine(checkoutMachine)
  const { data: cart } = useSuspenseQuery(cartQueryOptions)

  // Bridge: push server state into machine via events
  useEffect(() => {
    if (cart) send({ type: 'CART_LOADED', cart })
  }, [cart, send])

  // Machine triggers mutations via actions
  const submitOrder = useMutation({
    mutationFn: createOrder,
    onSuccess: () => send({ type: 'ORDER_CONFIRMED' }),
    onError: (err) => send({ type: 'ORDER_FAILED', error: err.message }),
  })
}
```

### With TanStack Router

Auth machine as global context, Router reads synchronously:
```typescript
export const Route = createFileRoute('/_authenticated')({
  beforeLoad: ({ context }) => {
    const snapshot = context.authActor.getSnapshot()
    if (!snapshot.matches('authenticated')) {
      throw redirect({ to: '/login' })
    }
  },
})
```

---

## When NOT to Use XState

- Simple toggle/boolean — `useState(false)`
- Derived state from props — direct computation
- Single async operation with no cancellation — `useEffect` + `useState` is fine
- Form field values — TanStack Form
- Global simple values (theme, locale) — Zustand or Context

Reach for `useReducer` before XState. XState earns its complexity when `useReducer`
needs `useEffect` to manage side effects — that is the signal.

---

## Common Pitfalls

1. **Impossible states in context** — model states as mutually exclusive machine states,
   not boolean flags in context.
2. **Side effects outside machine actions** — all side effects should be triggered by
   machine actions, entry/exit handlers, or invoked actors.
3. **Defining machines inside components** — wasteful recreation, untestable.
4. **React hooks inside machine services** — runtime error. Use `fromPromise` directly.
5. **Over-engineering simple state** — XState adds boilerplate. Justify it with 3+ states,
   guards, async, or parallel needs.

---

## Testing

Machines are pure TypeScript — test without React:
```typescript
import { createActor } from 'xstate'

test('checkout flow', () => {
  const actor = createActor(checkoutMachine)
  actor.start()
  expect(actor.getSnapshot().matches('cart')).toBe(true)
  actor.send({ type: 'NEXT' })
  expect(actor.getSnapshot().matches('shipping')).toBe(true)
})
```

For React integration, inject mock services via `provide`:
```typescript
checkoutMachine.provide({
  actors: { submitOrder: fromPromise(mockSubmit) },
})
```
