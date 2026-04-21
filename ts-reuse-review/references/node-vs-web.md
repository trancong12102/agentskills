# Node-specific vs Web-standard API choices

Load when the project targets Cloudflare Workers, Vercel Edge, Deno, Bun, or any mixed runtime. Signal: `wrangler.toml` / `vercel.json` with `runtime: edge` / `deno.json` / `bun.lockb` / `package.json` with `"type": "module"` and `"engines"` naming multiple runtimes.

Node-specific APIs (`Buffer`, `fs`, `path`, `crypto`, `stream`) fail silently or throw at cold-start on these runtimes. Web-standard alternatives work everywhere Node 18+ and edge runtimes run, so they're the safer default.

## Binary / encoding

| Node-specific                | Web-standard                                       | notes                                                                          |
| ---------------------------- | -------------------------------------------------- | ------------------------------------------------------------------------------ |
| `Buffer.from(str, 'utf8')`   | `new TextEncoder().encode(str)`                    | Returns Uint8Array. Matched by `buffer-utf8-decode` rule (reverse direction).  |
| `buf.toString('utf8')`       | `new TextDecoder().decode(buf)`                    |                                                                                |
| `Buffer.from(str, 'base64')` | `Uint8Array.from(atob(str), c => c.charCodeAt(0))` | `atob` is global on edge; Buffer isn't.                                        |
| `buf.toString('base64')`     | `btoa(String.fromCharCode(...buf))`                | Matched by `buffer-base64-string` rule. Chunked variant needed for large bufs. |
| `Buffer.from(arr)`           | `new Uint8Array(arr)`                              | Simpler, no Node dep.                                                          |
| `Buffer.concat([a, b])`      | `new Uint8Array([...a, ...b])` for small bufs      | Or use `Blob` concat for streams.                                              |

## Crypto

| Node-specific                                            | Web-standard                                                                                     | notes                                                           |
| -------------------------------------------------------- | ------------------------------------------------------------------------------------------------ | --------------------------------------------------------------- |
| `crypto.randomBytes(n)`                                  | `crypto.getRandomValues(new Uint8Array(n))`                                                      | `crypto` is global on edge (Web Crypto).                        |
| `crypto.randomUUID()`                                    | `crypto.randomUUID()`                                                                            | Same API — Web Crypto exposes UUID since mid-2022.              |
| `crypto.createHash('sha256').update(data).digest()`      | `await crypto.subtle.digest('SHA-256', data)`                                                    | Returns ArrayBuffer — wrap in Uint8Array/hex convert as needed. |
| `crypto.createHmac('sha256', key).update(data).digest()` | `crypto.subtle.sign('HMAC', key, data)` after `importKey`                                        | More boilerplate but runs everywhere.                           |
| `crypto.pbkdf2Sync(pw, salt, n, len, 'sha256')`          | `crypto.subtle.deriveBits({ name: 'PBKDF2', salt, iterations: n, hash: 'SHA-256' }, key, len*8)` |                                                                 |
| `crypto.createCipheriv(...)`                             | `crypto.subtle.encrypt({ name: 'AES-GCM', iv }, key, data)`                                      | Only AES-GCM, AES-CTR, AES-CBC on Web Crypto.                   |

## File system

| Node-specific                          | Web-standard                                                           | notes                                                             |
| -------------------------------------- | ---------------------------------------------------------------------- | ----------------------------------------------------------------- |
| `fs.promises.readFile(p, 'utf8')`      | `Bun.file(p).text()` / Deno `Deno.readTextFile(p)` / platform KV store | No FS on CF Workers — store in R2/KV/D1.                          |
| `fs.readFileSync(p)`                   | avoid — sync FS breaks event loop                                      | Use async even in Node.                                           |
| `path.join(a, b, c)`                   | `new URL(b, baseUrl)` for URL joining                                  | For true filesystem paths, Node `path` is fine in Node-only code. |
| `path.basename(p)` / `path.extname(p)` | `new URL(p).pathname.split('/').at(-1)` for URL paths                  | For file paths in Node, keep `path`.                              |

## Streams

| Node-specific                         | Web-standard                                 | notes                                                     |
| ------------------------------------- | -------------------------------------------- | --------------------------------------------------------- |
| `require('stream')` Readable/Writable | `ReadableStream` / `WritableStream` (WHATWG) | Edge runtimes only have Web streams.                      |
| `pipeline(src, dst)`                  | `src.pipeTo(dst)`                            | `pipeTo` returns a Promise — handle errors with `.catch`. |
| `stream.Transform`                    | `TransformStream`                            | Same concept, different API.                              |

## Timers

| Node-specific                         | Web-standard                        | notes                                                       |
| ------------------------------------- | ----------------------------------- | ----------------------------------------------------------- |
| `setImmediate(fn)`                    | `queueMicrotask(fn)`                | Not exact semantics but closest portable option.            |
| `process.nextTick(fn)`                | `queueMicrotask(fn)`                | Same caveat.                                                |
| `setTimeout` returns `NodeJS.Timeout` | `setTimeout` returns `number` (DOM) | Typing drift — `ReturnType<typeof setTimeout>` is portable. |

## Environment / process

| Node-specific              | Web-standard / platform                                                                    | notes                              |
| -------------------------- | ------------------------------------------------------------------------------------------ | ---------------------------------- |
| `process.env.FOO`          | CF Workers: injected `env` param / Vercel Edge: `process.env` works / Deno: `Deno.env.get` | Abstract behind a platform module. |
| `process.exit(1)`          | edge/Workers: no exit — return Response                                                    | Worker functions must return.      |
| `__dirname` / `__filename` | ESM: `import.meta.url` + `new URL('./', import.meta.url)`                                  | No `__dirname` in ESM.             |
| `process.cwd()`            | Deno `Deno.cwd()` / Bun same / edge: N/A                                                   | Avoid in portable code.            |

## HTTP

| Node-specific               | Web-standard             | notes                                        |
| --------------------------- | ------------------------ | -------------------------------------------- |
| `http.get(url, cb)`         | `fetch(url)`             | Works in Node 18+ natively.                  |
| `new URL(...).searchParams` | same                     | Already Web-standard.                        |
| `querystring.parse(s)`      | `new URLSearchParams(s)` | `querystring` is deprecated in Node since 7. |

## Logging / error

| Node-specific                       | Web-standard                                                  | notes                                          |
| ----------------------------------- | ------------------------------------------------------------- | ---------------------------------------------- |
| `console.log` + `util.inspect(obj)` | `console.log(obj)`                                            | Modern consoles handle objects.                |
| `util.promisify(cb)`                | wrap manually                                                 | For library callbacks in Node-only code, fine. |
| `util.types.isPromise(x)`           | `x instanceof Promise` or `x && typeof x.then === 'function'` |                                                |

## When to keep Node-specific

- File is explicitly Node-only (`bin/` scripts, server entry points pinned to Node runtime, build tooling).
- `wrangler.toml` has `compatibility_flags = ["nodejs_compat"]` — Workers can use `Buffer` etc. under the compat flag.
- `package.json` has `"engines": { "node": ">=18" }` with no edge target.
- Performance-critical hot path where Web Crypto digest vs Node `createHash` differs materially (rare — usually the opposite).

## Report guidance

When flagging a Node-specific API in a file under an edge/workers directory, mark confidence `high` and priority `P1` — this is a correctness gate, not a style choice. When the project has mixed runtime and the file is in a shared directory, `P2` with a note to verify the consumer.
