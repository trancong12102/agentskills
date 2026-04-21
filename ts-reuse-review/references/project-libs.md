# Project-installed libs + internal utils

Dynamic catalog. Loaded from `package.json` + workspace `packages/*/package.json` via `scripts/detect-libs.sh`. Any lib in this catalog that appears in installed deps becomes preferred over the fixed external catalog.

## Preference order (recap from SKILL.md)

1. Native API
2. Project-installed lib (this file)
3. Fixed external catalog (`references/external-libs.md`)
4. Install suggestion (only if fixed-catalog lib is missing)

Internal helper check (`scripts/scan-internal-utils.sh`) runs before emitting any replacement. A workspace helper beats every external suggestion.

## Installed-lib detection groups

### General utils — prefer installed over fixed

| installed lib | takes over from   | notes                                                                                               |
| ------------- | ----------------- | --------------------------------------------------------------------------------------------------- |
| `lodash-es`   | es-toolkit subset | suggest switching to es-toolkit if both installed; lodash-es is heavier, not ESM-first pre-v4.17.21 |
| `lodash`      | es-toolkit        | legacy CJS; suggest migrating                                                                       |
| `ramda`       | es-toolkit        | functional curry style; keep if already project style                                               |
| `remeda`      | es-toolkit        | modern pipe-first; prefer if installed                                                              |
| `radash`      | es-toolkit        | modern alternative; keep if installed                                                               |

When multiple of these are installed, prefer the style the project already uses (grep imports across `src/**` for dominant choice).

### Date — prefer installed over fixed

| installed lib | takes over from | notes                                                           |
| ------------- | --------------- | --------------------------------------------------------------- |
| `dayjs`       | date-fns        | mutable chain API; do not suggest switch — honor project choice |
| `luxon`       | date-fns        | Intl-backed, immutable; prefer if installed                     |
| `date-fns`    | fixed           | native case                                                     |
| `moment`      | date-fns        | legacy; flag P3 suggesting migration to date-fns, do not block  |

### Schema — prefer installed over fixed

| installed lib                       | takes over from | notes                                                                                    |
| ----------------------------------- | --------------- | ---------------------------------------------------------------------------------------- |
| `valibot`                           | zod             | smaller bundle; prefer if installed — same shape, `v.object({...})` vs `z.object({...})` |
| `yup`                               | zod             | older; keep if installed, do not suggest switch                                          |
| `superstruct`                       | zod             | similar; keep                                                                            |
| `arktype`                           | zod             | TS-type-based; keep                                                                      |
| `@effect/schema` or `effect/Schema` | zod             | Effect ecosystem; prefer if `effect` installed                                           |
| `runtypes`                          | zod             | keep if installed                                                                        |

### Async / effects

| installed lib                            | relevant utils                                                                                            | reinventions it covers                                                                    |
| ---------------------------------------- | --------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| `effect`                                 | `Effect.retry`, `Effect.timeout`, `Effect.schedule`, `Effect.all`, `Schema.decode`, `Stream.*`, `Match.*` | takes precedence over es-toolkit retry, manual Promise.race timeout, manual schema checks |
| `neverthrow`                             | `Result<T,E>`, `ResultAsync`                                                                              | try/catch throwing across layers                                                          |
| `ts-pattern`                             | `match(x).with(...).exhaustive()`                                                                         | switch-case on discriminated unions, nested if/else on tagged unions                      |
| `rxjs`                                   | `debounceTime`, `throttleTime`, `retry`, `timer`, `from`, `of`                                            | stream-shaped reinventions; only flag if existing file uses RxJS                          |
| `p-retry`, `p-queue`, `p-limit`, `p-map` | retry/queue/concurrency                                                                                   | hand-rolled retry loops, Promise.all with concurrency limits                              |

### HTTP / query

| installed lib           | relevant utils                                             | reinventions it covers                    |
| ----------------------- | ---------------------------------------------------------- | ----------------------------------------- |
| `ky` / `ofetch`         | fetch wrappers with retries, timeouts                      | manual `fetch` + retry loop               |
| `@tanstack/react-query` | `useQuery`, `useMutation`, `queryClient.invalidateQueries` | `useEffect` + `useState` for server state |
| `swr`                   | `useSWR`                                                   | same as above                             |
| `@tanstack/query-core`  | standalone query cache                                     | manual request cache Maps                 |

### Collections / immutability

| installed lib | relevant utils          | reinventions it covers            |
| ------------- | ----------------------- | --------------------------------- |
| `immer`       | `produce(draft, fn)`    | manual deep-spread update helpers |
| `immutable`   | `Map`, `List`, `Record` | only if project uses it           |

### State

| installed lib | relevant utils       | reinventions it covers                |
| ------------- | -------------------- | ------------------------------------- |
| `zustand`     | `create(set => ...)` | hand-rolled context + reducer pattern |
| `jotai`       | `atom`, `useAtom`    | context stores for atomic state       |
| `xstate`      | machines             | reducer-based state machines          |

### React hooks libraries

| installed lib    | relevant utils                                                                                                                                          | notes                                                                |
| ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- |
| `usehooks-ts`    | `usePrevious`, `useDebounce`, `useInterval`, `useTimeout`, `useLocalStorage`, `useOnClickOutside`, `useEventListener`, `useWindowSize`, `useMediaQuery` | Small, TS-native. Default suggestion.                                |
| `ahooks`         | Superset of above + `useRequest`, `useLatest`, `useMemoizedFn`, `useCreation`, `useUpdateEffect`                                                        | Larger (Alibaba-maintained). Suggest when project already uses antd. |
| `react-use`      | —                                                                                                                                                       | Unmaintained since 2023. Do not suggest.                             |
| `@mantine/hooks` | Similar surface to `usehooks-ts`                                                                                                                        | Suggest when project uses `@mantine/core`.                           |

Matched by `use-previous-manual` + `use-latest-ref` rules (plus anything future under `references/react-patterns.md`).

### Testing

| installed lib            | relevant utils                                                                             | reinventions it covers                                          |
| ------------------------ | ------------------------------------------------------------------------------------------ | --------------------------------------------------------------- |
| `vitest`                 | `vi.fn()`, `vi.spyOn`, `vi.mock`, `vi.useFakeTimers`, `vi.waitFor`, `test.extend` fixtures | Manual mocks, polling loops, `await sleep()` in tests.          |
| `jest`                   | `jest.fn()`, `jest.spyOn`, `jest.mock`, `jest.useFakeTimers`                               | Same as vitest at the matcher level.                            |
| `@testing-library/react` | `screen.getByRole`, `waitFor`, `userEvent`, `findBy*`                                      | `container.querySelector`, `fireEvent`, polling for async text. |
| `msw`                    | `http.get`, `HttpResponse.json`, `server.use`                                              | `global.fetch = vi.fn(...)`, manual response stubs.             |
| `@vitest/expect`         | match matchers                                                                             | `JSON.stringify`-based equality.                                |

Load `references/testing-patterns.md` when a test file is being scanned and any of these are installed.

### Platform / runtime

| installed lib           | relevant utils                                   | notes                                                                 |
| ----------------------- | ------------------------------------------------ | --------------------------------------------------------------------- |
| `@effect/platform`      | `HttpClient`, `HttpServer`, `FileSystem`, `Path` | When `effect` installed and scanning server/platform code.            |
| `@effect/platform-node` | Node-specific `HttpClient`/`FileSystem` impl     | Same as above but Node-target.                                        |
| `hono`                  | `app.get/post`, `c.json`, `zValidator`           | Manual Worker `fetch` handlers + request parsing.                     |
| `@hono/zod-validator`   | Input validation for Hono routes                 | Manual try/catch + zod parse inside each route.                       |
| `wrangler`              | Worker tooling                                   | Signal: target Cloudflare Workers — load `references/node-vs-web.md`. |

### Forms

| installed lib                           | relevant utils                                       | reinventions it covers                                                        |
| --------------------------------------- | ---------------------------------------------------- | ----------------------------------------------------------------------------- |
| `react-hook-form`                       | `useForm`, `register`, `useFieldArray`, `Controller` | useState per field, manual onChange/onBlur wiring, custom validation effects. |
| `@hookform/resolvers`                   | `zodResolver`, `valibotResolver`, `yupResolver`      | Manual per-field validators that duplicate a zod schema.                      |
| `formik`                                | `<Formik>`, `<Field>`, `useFormik`                   | Same as rhf.                                                                  |
| `@tanstack/react-form`                  | `useForm` (framework-agnostic engine)                | Fine-grained reactive fields.                                                 |
| `@conform-to/react` + `@conform-to/zod` | `useForm`, `parseWithZod`                            | Progressive-enhancement forms in Next/Remix.                                  |

Load `references/form-patterns.md` when any of these is installed and the diff touches a form component.

### Query / data fetching

| installed lib                        | relevant utils                                                          | reinventions it covers                                                     |
| ------------------------------------ | ----------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| `@tanstack/react-query`              | `useQuery`, `useMutation`, `useInfiniteQuery`, `queryClient.invalidate` | `useState + useEffect + fetch` trio, cache invalidation, poll, pagination. |
| `@tanstack/react-query-devtools`     | DevTools                                                                | Manual logging of request state.                                           |
| `swr`                                | `useSWR`, `useSWRInfinite`, `mutate`                                    | Same as react-query, smaller.                                              |
| `@reduxjs/toolkit/query`             | `createApi`, `fetchBaseQuery`, generated hooks                          | Manual Redux slices for async.                                             |
| `ts-rest`                            | Typed contracts between client/server                                   | Hand-rolled type-safe fetch wrappers.                                      |
| `@trpc/client` + `@trpc/react-query` | Typed client calls via proxy                                            | Same as ts-rest.                                                           |

Load `references/query-patterns.md` when any of these is installed or the `react-fetch-useeffect` rule fires.

### Immutability

| installed lib      | relevant utils                        | reinventions it covers                        |
| ------------------ | ------------------------------------- | --------------------------------------------- |
| `immer`            | `produce`, `castDraft`                | 3-level spread updates.                       |
| `use-immer`        | `useImmer`, `useImmerReducer`         | React state with nested updates.              |
| `mutative`         | `create` (drop-in, faster than immer) | Same as immer; suggest only if perf-profiled. |
| `@reduxjs/toolkit` | `createSlice` (ships immer)           | Spread inside reducers — redundant.           |

Load `references/immer-immutability.md` when any of these is installed OR `nested-spread-update` rule fires.

### Validation (load-on-demand)

When `zod` / `valibot` / `yup` / `arktype` installed AND zod-family rules fire (`email-regex`, `regex-uuid`, `regex-ipv4`, `regex-iso-datetime`, `zod-discriminated-union`, `url-validate-try`, `manual-schema-guard`), load `references/zod-patterns.md`. Substitute `v.*` for valibot or the lib's equivalents as needed.

## Detection script output shape

`scripts/detect-libs.sh` prints NDJSON, one line per installed relevant lib:

```json
{"lib":"es-toolkit","group":"general","version":"1.21.0","source":"package.json"}
{"lib":"effect","group":"async","version":"3.5.0","source":"packages/server/package.json"}
{"lib":"zod","group":"schema","version":"3.23.0","source":"package.json"}
```

Parse this output at Step 2 of the SKILL workflow. Use the `group` field to decide which catalog entries get the "installed" boost.

## Internal util detection

`scripts/scan-internal-utils.sh <candidate-fn-names>` searches:

- `src/**/utils/**/*.{ts,tsx,js,jsx,mjs,cjs}`
- `src/**/lib/**/*.{ts,tsx,js,jsx,mjs,cjs}`
- `src/**/helpers/**/*.{ts,tsx,js,jsx,mjs,cjs}`
- `packages/*/src/**/*.{ts,tsx,js,jsx,mjs,cjs}`
- `shared/**/*.{ts,tsx,js,jsx,mjs,cjs}`
- `common/**/*.{ts,tsx,js,jsx,mjs,cjs}`
- root `utils.{ts,js}`, `helpers.{ts,js}`, `lib.{ts,js}`

For each candidate fn name from pattern matches (e.g., `chunk`, `debounce`, `groupBy`), the script greps `export function <name>`, `export const <name> =`, `export { <name> }`. Output NDJSON:

```json
{"name":"chunk","path":"src/utils/array.ts","line":12,"kind":"function"}
{"name":"debounce","path":"packages/shared/timer.ts","line":34,"kind":"const-arrow"}
```

Match found → replace external suggestion with `use existing: <path>:<line>`.

Match not found → proceed with external / native recommendation.

## Edge cases

- **Shadowed names**: If `groupBy` is defined in both `src/utils/array.ts` and `packages/shared/collection.ts`, list all matches and let the user pick — do not guess.
- **Re-exports**: If the internal helper is `export { groupBy } from 'es-toolkit'`, treat as the external lib call, not a new helper. Use import path from the re-export file.
- **Type-only exports**: `export type Chunk = ...` does not count as a helper. Require value exports.
- **Project style already avoids a lib**: If no imports of `es-toolkit` exist in `src/**` but the file under review imports `lodash`, match the file's style — suggest lodash equivalents first, es-toolkit as P3 migration hint.
