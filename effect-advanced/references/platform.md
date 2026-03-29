# Platform — HTTP Client, FileSystem, Command & Runtime

## Architecture

Business logic is written against abstract services from `@effect/platform`. At the
entry point, inject the concrete runtime layer:

```typescript
import { NodeRuntime, NodeContext } from "@effect/platform-node";

NodeRuntime.runMain(myProgram.pipe(Effect.provide(NodeContext.layer)));
```

Swap `BunContext.layer` for Bun. `NodeRuntime.runMain` handles error reporting,
logging, signal management (SIGINT/SIGTERM), and exit codes.

**Stability warning:** HTTP modules (`HttpClient`, `HttpServer`, `HttpRouter`) are
marked unstable — pin versions tightly.

---

## HTTP Client

### Basic request with schema decoding

```typescript
import {
  HttpClient,
  HttpClientResponse,
  FetchHttpClient,
} from "@effect/platform";

const Post = Schema.Struct({ id: Schema.Number, title: Schema.String });

const getPost = (id: number) =>
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient;
    const response = yield* client.get(`/posts/${id}`);
    return yield* HttpClientResponse.schemaBodyJson(Post)(response);
  }).pipe(Effect.scoped); // Always scope HTTP responses
```

### Building a typed API client

```typescript
const ApiClient = Effect.gen(function* () {
  const base = (yield* HttpClient.HttpClient).pipe(
    HttpClient.mapRequest(
      HttpClientRequest.prependUrl("https://api.example.com"),
    ),
    HttpClient.filterStatusOk,
    HttpClient.retryTransient({
      schedule: Schedule.exponential("100 millis").pipe(Schedule.recurs(3)),
    }),
  );

  return {
    getPosts: base
      .get("/posts")
      .pipe(
        Effect.flatMap(HttpClientResponse.schemaBodyJson(Schema.Array(Post))),
        Effect.scoped,
      ),
  };
});
```

### Status-based response branching

```typescript
client.get("/resource").pipe(
  Effect.andThen(
    HttpClientResponse.matchStatus({
      200: HttpClientResponse.schemaBodyJson(SuccessSchema),
      404: () => Effect.fail(new NotFoundError()),
      orElse: (response) => Effect.fail(new UnexpectedStatus(response.status)),
    }),
  ),
);
```

**Gotchas:**

- Always add `Effect.scoped` when using `HttpClientResponse` methods — the response
  holds an open connection until the scope closes
- `filterStatusOk` must be applied on the client, not post-response

---

## FileSystem

```typescript
import { FileSystem } from "@effect/platform";

const program = Effect.gen(function* () {
  const fs = yield* FileSystem.FileSystem;
  const contents = yield* fs.readFileString("./input.txt", "utf-8");
  yield* fs.writeFileString("./output.txt", contents.toUpperCase());
  yield* fs.makeDirectory("./dist", { recursive: true });
  const entries = yield* fs.readDirectory("./src");
});
```

---

## Command Execution

Commands are **lazy** — they define work but don't execute until run:

```typescript
import { Command } from "@effect/platform";

// Simple output capture
const output =
  yield * Command.string(Command.make("git", "log", "--oneline", "-10"));

// Stream output for long-running processes
const lines =
  yield * Command.streamLines(Command.make("tail", "-f", "./app.log"));

// Full process control (requires Scope)
const proc =
  yield *
  Effect.scoped(
    Command.start(Command.make("ffmpeg", "-i", "in.mp4", "out.mp3")),
  );
```

**Gotcha:** `Command.start` requires `Effect.scoped`. Without it, the process handle
leaks. Use `Command.string`/`Command.lines`/`Command.exitCode` for simpler cases.

---

## Framework Integration (Express / Hono / Fastify)

No official adapters exist. Bridge Effect at the route handler boundary:

```typescript
// Hono example
app.get("/users/:id", async (c) => {
  const result = await Effect.runPromise(
    getUser(c.req.param("id")).pipe(
      Effect.provide(AppLayer),
      Effect.catchAll((e) => Effect.succeed({ error: e.message })),
    ),
  );
  return c.json(result);
});
```

For full Effect-native HTTP, use `@effect/platform`'s `HttpRouter` + `HttpServer`:

```typescript
import { HttpRouter, HttpServer, HttpServerResponse } from "@effect/platform";

const router = HttpRouter.empty.pipe(
  HttpRouter.get("/health", HttpServerResponse.text("ok")),
  HttpRouter.post("/users", createUserHandler),
);
```

---

## Database Integration

Recommended stack: `@effect/sql-pg` + `@effect/sql-drizzle`

```typescript
import * as PgDrizzle from "@effect/sql-drizzle/Pg";
import { PgClient } from "@effect/sql-pg";

const PgLive = PgClient.layerConfig({
  url: Config.redacted("DATABASE_URL"),
});

const DrizzleLive = PgDrizzle.layer.pipe(Layer.provide(PgLive));
```

---

## Logging & Observability

### Structured logging

```typescript
Effect.gen(function* () {
  yield* Effect.log("Processing request");
  yield* Effect.logDebug("Raw payload", payload);
  yield* Effect.logWarning("Slow query", { ms: 430 });
  yield* Effect.logError("DB failure", cause);
}).pipe(Effect.annotateLogs({ requestId: "abc-123", userId: "u-456" }));
```

### JSON logger for production

```typescript
program.pipe(Effect.provide(Logger.replace(Logger.defaultLogger, Logger.json)));
```

### Distributed tracing

```typescript
const processOrder = (orderId: string) =>
  Effect.gen(function* () {
    yield* Effect.annotateCurrentSpan("orderId", orderId);
    const order = yield* fetchOrder(orderId).pipe(
      Effect.withSpan("fetchOrder"),
    );
    yield* chargePayment(order).pipe(Effect.withSpan("chargePayment"));
  }).pipe(Effect.withSpan("processOrder"));
```

### OpenTelemetry setup

```typescript
import { NodeSdk } from "@effect/opentelemetry";
import { BatchSpanProcessor } from "@opentelemetry/sdk-trace-base";
import { OTLPTraceExporter } from "@opentelemetry/exporter-trace-otlp-http";

const OtelLive = NodeSdk.layer(() => ({
  resource: { serviceName: "my-api" },
  spanProcessor: new BatchSpanProcessor(
    new OTLPTraceExporter({
      url: "http://collector:4318/v1/traces",
    }),
  ),
}));
```

Logs inside spans are automatically converted to Span Events by the OpenTelemetry
bridge.

---

## Common Pitfalls

1. **Missing `Effect.scoped` on HTTP responses** — the response holds an open
   connection. Always scope it.

2. **`Command.start` without scope** — leaks process handles.

3. **`Effect.runPromise` in production** — doesn't handle signals. Use
   `NodeRuntime.runMain`.

4. **Not pinning `@effect/platform` versions** — HTTP modules are unstable and
   can break in minor releases.
