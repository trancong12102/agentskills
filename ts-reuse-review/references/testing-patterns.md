# Testing reinventions (vitest / jest)

Load when `vitest` or `jest` is in installed deps AND the diff includes files under `**/*.{test,spec}.{ts,tsx,js,jsx}` or `__tests__/`. Skip otherwise — testing rules are opinionated and don't apply to source code.

Scope is deliberately narrow — test files are where most "rolled my own" reinventions live because devs treat tests as scratch code. Flag high-confidence cases only.

## Mocking

| reinvention                                                               | vitest/jest replacement                                 | notes                                                                    |
| ------------------------------------------------------------------------- | ------------------------------------------------------- | ------------------------------------------------------------------------ |
| `import * as mod from './x'; mod.fn = (() => ...) as any`                 | `vi.spyOn(mod, 'fn').mockImplementation(() => ...)`     | Reassigning ESM exports is a type error and doesn't reset between tests. |
| `jest.fn(() => value)` then asserting call count via `.mock.calls.length` | `expect(fn).toHaveBeenCalledTimes(n)`                   | Matcher form gives better failure messages.                              |
| Manual mock module: `const mockService = { getUser: () => ... }`          | `vi.mock('./service', () => ({ getUser: vi.fn(...) }))` | Enables `.mockResolvedValue` etc.                                        |
| Re-implementing a class to stub methods                                   | `vi.fn()` for methods + `vi.spyOn(instance, 'method')`  |                                                                          |

## Setup / teardown

| reinvention                                                     | replacement                                                                | notes                              |
| --------------------------------------------------------------- | -------------------------------------------------------------------------- | ---------------------------------- |
| Setup-per-test via `beforeEach(async () => { db.reset(); ...})` | Scoped fixture via `test.extend({ db })` (vitest) or `describe.concurrent` | Cleaner isolation + parallel-safe. |
| Global fixture via module-level `let user`                      | `beforeEach(async () => { user = await ... })`                             | Avoids stale state across files.   |
| Custom `cleanup()` called manually at test end                  | `afterEach(cleanup)` (React Testing Library does this by default).         |                                    |

## Async / timers

| reinvention                                              | replacement                                                           | notes                                                    |
| -------------------------------------------------------- | --------------------------------------------------------------------- | -------------------------------------------------------- |
| `await new Promise(r => setTimeout(r, 100))` inside test | `vi.useFakeTimers(); vi.advanceTimersByTime(100); vi.useRealTimers()` | Deterministic, fast, works with debounce/throttle tests. |
| Polling loop `while (!ready) { await sleep(10) }`        | `await vi.waitFor(() => expect(cond).toBe(true))`                     | Bounded, typed timeout, cleaner failures.                |
| `await Promise.resolve()` to flush microtasks            | `await vi.waitFor(...)` or `await flushPromises()` util               | Chaining `.resolve()` is fragile.                        |

## Assertions / matchers

| reinvention                                                     | replacement                              | notes                                                    |
| --------------------------------------------------------------- | ---------------------------------------- | -------------------------------------------------------- |
| `expect(JSON.stringify(a)).toBe(JSON.stringify(b))`             | `expect(a).toEqual(b)`                   | Deep-equal matcher, reports diff, handles NaN correctly. |
| `expect(arr.includes(x)).toBe(true)`                            | `expect(arr).toContain(x)`               | Better error messages.                                   |
| `expect(arr.length).toBe(n)`                                    | `expect(arr).toHaveLength(n)`            | Same.                                                    |
| `expect(typeof x).toBe('string')`                               | `expect(x).toEqual(expect.any(String))`  | Pairs with `.toMatchObject`.                             |
| `expect(() => fn()).toThrow()` after manually calling fn in try | `expect(() => fn()).toThrow(ErrorClass)` | Direct matcher — no manual try/catch.                    |
| `expect(spy.mock.calls[0][0]).toEqual(arg)`                     | `expect(spy).toHaveBeenCalledWith(arg)`  |                                                          |

## Snapshot / inline

| reinvention                                                       | replacement                              | notes                                                                  |
| ----------------------------------------------------------------- | ---------------------------------------- | ---------------------------------------------------------------------- |
| `expect(output).toEqual({ long: 'hand-written object literal' })` | `expect(output).toMatchInlineSnapshot()` | If the output is mostly static and verbose. Run `--update` to capture. |
| External `__snapshots__/` file for tiny scalar snapshots          | Prefer inline for readability.           |                                                                        |

## React Testing Library

| reinvention                                                  | replacement                                            | notes                                                               |
| ------------------------------------------------------------ | ------------------------------------------------------ | ------------------------------------------------------------------- |
| `container.querySelector('[data-testid=foo]')`               | `screen.getByTestId('foo')`                            | Throws with a helpful message, not `null`.                          |
| `container.querySelector('button').textContent === 'Submit'` | `screen.getByRole('button', { name: 'Submit' })`       | Role-first queries align with a11y best practice.                   |
| `fireEvent.click(button)`                                    | `await userEvent.click(button)`                        | `user-event` emits the full event sequence (pointer, mouse, click). |
| Polling `getByText` in a loop for async content              | `await screen.findByText(...)` or `await waitFor(...)` | Built-in retry.                                                     |

## MSW / HTTP mocking

If `msw` installed:

| reinvention                                                        | replacement                                                        | notes                                                      |
| ------------------------------------------------------------------ | ------------------------------------------------------------------ | ---------------------------------------------------------- |
| `global.fetch = vi.fn(() => Promise.resolve({ json: () => ... }))` | `server.use(http.get('/api/x', () => HttpResponse.json({ ... })))` | MSW gives request assertions + network-level interception. |

## Non-reinvention — don't flag

- `test('description', () => ...)` vs `it(...)` — stylistic.
- Custom test matchers in `src/test/matchers.ts` — project extensions, not reinventions.
- Setup files under `src/test/setup.ts` — framework convention.
- Factory functions `makeUser(overrides)` — builders are fine, idiomatic.

## Priority guidance

- **P1**: any mock reassignment that's type-unsafe or leaks between tests.
- **P2**: promise-sleep in tests (flaky on CI), `JSON.stringify` equality (poor diff).
- **P3**: matcher style upgrades (`toHaveLength` vs `.length`).

Keep P3 noise low — test files are often green-bar-reliant and style churn annoys maintainers.
