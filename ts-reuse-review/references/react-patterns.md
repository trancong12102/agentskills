# React hook reinventions

Load when scanning `.tsx`/`.jsx` files or when `react` is in installed deps. Skip otherwise.

React's hook model encourages ad-hoc helpers that duplicate what `usehooks-ts`, `ahooks`, or `react-use` already ship. The cost of adding one of these libs is tiny (~2kb gzipped, tree-shaken) compared to the bug surface of hand-rolled versions.

Canonical install for suggestions: `usehooks-ts` (small, TS-native) or `ahooks` (large, battle-tested at Alibaba scale). Mention both when making the suggestion; let user pick.

## Mirror / snapshot hooks

| hook                   | reinvention shape                                               | notes                                                        |
| ---------------------- | --------------------------------------------------------------- | ------------------------------------------------------------ |
| `usePrevious(value)`   | `useRef` + `useEffect(() => { ref.current = value }, [value])`  | Classic "track prior prop". Returns previous render's value. |
| `useLatest(value)`     | `useRef` + `useEffect(() => { ref.current = value })` (no deps) | Always-fresh ref for callbacks/timers/subscriptions.         |
| `useIsMounted()`       | `useRef(false)` + set true in effect + cleanup setting false    | Common around setState-after-unmount guards.                 |
| `useFirstMountState()` | `useRef(true)` + set false after first effect                   | Skip first-render effect body.                               |
| `useUnmountedRef()`    | Same as `useIsMounted` but returning the ref directly           | Used in async handlers to check before setState.             |

## Lifecycle abbreviations

| hook              | reinvention shape                          | notes                                                                  |
| ----------------- | ------------------------------------------ | ---------------------------------------------------------------------- |
| `useMount(fn)`    | `useEffect(fn, [])`                        | P3 stylistic — literal `useEffect(..., [])` is fine, do not over-flag. |
| `useUnmount(fn)`  | `useEffect(() => fn, [])`                  | Empty-deps effect returning cleanup only. Worth flagging if ≥3 lines.  |
| `useUpdateEffect` | `useRef(true)` + early-return first render | Skip-first-render variant of useEffect.                                |

## Async / timing

| hook                           | reinvention shape                                                                                       |
| ------------------------------ | ------------------------------------------------------------------------------------------------------- |
| `useDebounce(value, ms)`       | `useState` mirror + `useEffect` with `setTimeout` setting the mirror, `clearTimeout` in cleanup.        |
| `useDebouncedCallback(fn, ms)` | `useRef<NodeJS.Timeout>()` + `useCallback` clearing timer and scheduling call.                          |
| `useThrottle(value, ms)`       | `useRef<number>()` + `useEffect` with timestamp-based gate.                                             |
| `useInterval(fn, ms)`          | `useEffect` + `setInterval`/`clearInterval` with latest-ref for fn. Dan Abramov's canonical post.       |
| `useTimeout(fn, ms)`           | `useEffect` + `setTimeout`/`clearTimeout` with latest-ref for fn.                                       |
| `useAsync(fn, deps)`           | `useState` for loading/error/value + `useEffect` dispatching fn. Superseded by `react-query`/`swr` too. |

## DOM / window

| hook                         | reinvention shape                                              |
| ---------------------------- | -------------------------------------------------------------- |
| `useWindowSize()`            | `useState({w,h})` + `useEffect` attaching `resize` listener.   |
| `useMediaQuery(query)`       | `window.matchMedia` + `useState` + event listener.             |
| `useEventListener(ev, fn)`   | `useRef(fn)` for latest + `useEffect` add/removeEventListener. |
| `useOnClickOutside(ref, fn)` | Global `mousedown` listener checking `ref.current.contains`.   |
| `useIntersectionObserver`    | `IntersectionObserver` + ref callback + state for entry.       |
| `useKeyPress(key)`           | Window `keydown` + `keyup` + state bool.                       |

## State mirrors

| hook                              | reinvention shape                                                                        |
| --------------------------------- | ---------------------------------------------------------------------------------------- | ------------------------ |
| `useLocalStorage(key, initial)`   | `useState` with `localStorage.getItem`/`setItem` + sync across tabs via `storage` event. |
| `useSessionStorage(key, initial)` | Same as above with sessionStorage.                                                       |
| `useToggle(initial)`              | `useState<boolean>` + `useCallback(() => setX(x => !x))`.                                |
| `useCounter(initial)`             | `useState<number>` + inc/dec/reset callbacks.                                            |
| `useArray(initial)`               | `useState<T[]>` + push/remove/update callbacks.                                          |
| `useMap()` / `useSet()`           | `useState<Map                                                                            | Set>` + wrapper setters. |

## Copy / clipboard

| hook                 | reinvention shape                                                                  |
| -------------------- | ---------------------------------------------------------------------------------- |
| `useCopyToClipboard` | `navigator.clipboard.writeText` + fallback `document.execCommand` + state + timer. |

## Preferences / observation

| hook                      | reinvention shape                                                    |
| ------------------------- | -------------------------------------------------------------------- |
| `usePrefersDarkMode`      | `matchMedia('(prefers-color-scheme: dark)')` + listener + state.     |
| `usePrefersReducedMotion` | `matchMedia('(prefers-reduced-motion: reduce)')` + listener + state. |
| `useOnline()`             | `navigator.onLine` + `online`/`offline` listeners + state.           |

## Non-reinvention cases

Do **not** flag:

- `useEffect(fn, [])` on its own — empty-deps effects are valid and replacing with `useMount` is stylistic at best. Only flag when the project already imports a hooks lib and is inconsistent.
- Custom hooks that compose business logic on top of a primitive (e.g., `useAuth` wrapping `useQuery`). Those encode domain decisions, not util reinventions.
- Any hook that reads from Redux / Zustand / context — project-specific state.
- Test-only hooks under `*.test.tsx` / `__tests__/`.

## Suggesting which lib to install

If no hooks lib is installed and the file uses ≥3 reinvented hooks, recommend `usehooks-ts` first (small, fully typed). If project already uses Ant Design or has Chinese/CJK comments/strings, `ahooks` is the safer bet (ecosystem alignment). Never recommend `react-use` — unmaintained since 2023.

Do not fight project convention. If the file imports from `@project/hooks/useSomething`, check `scan-internal-utils.sh` output first and point there rather than suggesting `usehooks-ts`.
