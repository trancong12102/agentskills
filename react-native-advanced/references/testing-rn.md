# Testing React Native — RNTL, React Query, Expo Router, MSW

For core testing patterns (React Query fresh client, XState pure machine testing, TanStack
Form UI testing), see `react-advanced`'s `references/testing.md`. This file covers
RN-specific testing with @testing-library/react-native.

---

## Key Differences from Web Testing

| Area               | @testing-library/react (web) | @testing-library/react-native (RN)         |
| ------------------ | ---------------------------- | ------------------------------------------ |
| Import             | `@testing-library/react`     | `@testing-library/react-native`            |
| Host elements      | HTML tags (`div`, `button`)  | RN components (`View`, `Text`)             |
| `getByRole`        | Full ARIA roles              | Subset: `button`, `header`, `link`         |
| Text input         | `userEvent.type`             | `userEvent.type` or `fireEvent.changeText` |
| Assertions         | `toBeInTheDocument()`        | `toBeOnTheScreen()`                        |
| Router testing     | `MemoryRouter` wrapper       | `renderRouter` from expo-router            |
| Event firing order | DOM-standard                 | Platform-dependent (iOS ≠ Android)         |

---

## React Query Testing Setup

Same core principle as web — fresh `QueryClient` per test:

```typescript
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, renderHook } from '@testing-library/react-native'

function createTestQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: { retry: false, gcTime: Infinity },
      mutations: { retry: false },
    },
  })
}

function createQueryWrapper() {
  const client = createTestQueryClient()
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  )
}

// Testing a hook
const { result } = renderHook(() => useMyHook(), {
  wrapper: createQueryWrapper(),
})
await waitFor(() => expect(result.current.isSuccess).toBe(true))

// Testing a component
render(<MyComponent />, { wrapper: createQueryWrapper() })
```

---

## Expo Router Testing

Use `renderRouter` from `expo-router/testing-library` — do not put test files inside
the `app/` directory.

```typescript
import { renderRouter, screen } from 'expo-router/testing-library'
import { fireEvent, waitFor } from '@testing-library/react-native'

test('navigates to profile', async () => {
  renderRouter(
    {
      index: () => <Home />,
      'profile/[id]': () => <Profile />,
    },
    { initialUrl: '/' },
  )

  expect(screen).toHavePathname('/')

  fireEvent.press(screen.getByText('Go to Profile'))
  await waitFor(() => {
    expect(screen).toHavePathname('/profile/123')
  })
})
```

### Available Jest matchers on `screen`

| Matcher                      | What it checks           |
| ---------------------------- | ------------------------ |
| `toHavePathname('/route')`   | Current pathname         |
| `toHaveSegments(['[id]'])`   | Route segment array      |
| `toHaveRouterState({ ... })` | Full router state object |

### Testing auth redirects

```typescript
test('redirects to login when not authenticated', async () => {
  // Set auth state to unauthenticated
  useAuthStore.setState({ session: null })

  renderRouter(
    {
      index: () => <Home />,
      'sign-in': () => <SignIn />,
      _layout: () => <RootLayout />,
    },
    { initialUrl: '/' },
  )

  await waitFor(() => {
    expect(screen).toHavePathname('/sign-in')
  })
})
```

Always wrap post-navigation assertions with `waitFor` — pathname state can lag after
redirects.

---

## Testing TanStack Form (TextInput)

Use `userEvent.type()` for realistic event simulation:

```typescript
import { render, screen, userEvent, waitFor } from '@testing-library/react-native'

test('submits form with correct values', async () => {
  const onSubmit = jest.fn()
  const user = userEvent.setup()

  render(<MyForm onSubmit={onSubmit} />)

  await user.type(screen.getByLabelText('Name'), 'Alice')
  await user.type(screen.getByLabelText('Email'), 'alice@test.com')
  await user.press(screen.getByRole('button', { name: 'Submit' }))

  await waitFor(() => {
    expect(onSubmit).toHaveBeenCalledWith(
      expect.objectContaining({ name: 'Alice', email: 'alice@test.com' }),
    )
  })
})
```

`fireEvent.changeText` also works for direct value setting:

```typescript
fireEvent.changeText(screen.getByPlaceholderText("Enter name"), "Alice");
```

**iOS vs Android event order:** iOS fires `keyPress → change → changeText`. Android fires
`change → changeText → keyPress`. `userEvent.type()` handles this — avoid asserting on
raw event sequences.

---

## Testing XState in RN

XState v5 is fully isomorphic — testing is **identical** to web. Test machines as pure
TypeScript, no React/DOM needed:

```typescript
import { createActor } from "xstate";

test("transitions correctly", () => {
  const actor = createActor(toggleMachine);
  actor.start();
  expect(actor.getSnapshot().value).toBe("inactive");
  actor.send({ type: "TOGGLE" });
  expect(actor.getSnapshot().value).toBe("active");
});
```

For React integration tests, test through UI interactions instead of inspecting machine
state directly.

---

## MSW in React Native

### For Jest tests (unit/integration) — use `msw/node`

Standard Node.js interceptors, no polyfills needed:

```typescript
// src/mocks/server.ts
import { setupServer } from "msw/node";
import { http, HttpResponse } from "msw";

const handlers = [
  http.get("https://api.example.com/users", () => {
    return HttpResponse.json([{ id: 1, name: "Alice" }]);
  }),
];

export const server = setupServer(...handlers);

// jest.setup.ts
beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());
```

### For runtime dev mocking (simulator) — use `msw/native`

Requires polyfills:

```bash
npm install react-native-url-polyfill fast-text-encoding
```

```typescript
// msw.polyfills.js
import "fast-text-encoding";
import "react-native-url-polyfill/auto";

// src/mocks/server.ts (runtime)
import { setupServer } from "msw/native"; // NOT msw/node
```

**Common error:** `Unable to resolve module 'http'` means you used `msw/node` where
`msw/native` is required. `msw/node` uses Node.js APIs, `msw/native` uses interceptors
compatible with React Native's fetch.

---

## Common Pitfalls

1. **Importing from `@testing-library/react` instead of `/react-native`** — different
   host element expectations, different assertion matchers.

2. **Using `toBeInTheDocument()` instead of `toBeOnTheScreen()`** — the DOM matcher
   doesn't exist in RNTL. Install `@testing-library/jest-native` for RN matchers.

3. **Not wrapping navigation assertions in `waitFor`** — route transitions are async.
   `expect(screen).toHavePathname(...)` may fail without `waitFor`.

4. **Shared QueryClient across tests** — always create fresh per test. See React Query
   testing setup above.

5. **Test files inside `app/` directory** — Expo Router treats files in `app/` as routes.
   Put tests in `__tests__/` or co-locate outside `app/`.

6. **Mocking `expo-router` instead of using `renderRouter`** — `renderRouter` provides
   a real in-memory router with full navigation state. Manual mocks lose navigation
   behavior.
