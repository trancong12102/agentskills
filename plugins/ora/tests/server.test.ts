// End-to-end: drives the real server over stdio JSON-RPC, running real bash.
//   bun test plugins/ora/tests/server.test.ts
import { afterAll, beforeAll, expect, test } from "bun:test";
import { existsSync, readdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const root = join(import.meta.dir, "..");
let proc: Bun.Subprocess<"pipe", "pipe", "inherit">;
let nextId = 1;
const waiting = new Map<number, (message: any) => void>();

async function readLoop() {
  const decoder = new TextDecoder();
  // Held as parts until a newline arrives, so a large message is not rescanned on every chunk.
  let parts: string[] = [];
  for await (const chunk of proc.stdout) {
    const text = decoder.decode(chunk, { stream: true });
    if (!text.includes("\n")) {
      parts.push(text);
      continue;
    }
    const lines = (parts.join("") + text).split("\n");
    parts = [lines.pop()!];
    for (const line of lines) {
      const message = JSON.parse(line);
      waiting.get(message.id)?.(message);
    }
  }
}

function request(method: string, params: object = {}): Promise<any> {
  const id = nextId++;
  const reply = new Promise((resolve) => waiting.set(id, resolve));
  proc.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", id, method, params })}\n`);
  proc.stdin.flush();
  return reply;
}

async function run(args: Record<string, unknown>): Promise<string> {
  const reply = await request("tools/call", { name: "run", arguments: args });
  return reply.result.content[0].text;
}

// Run inside an ora session or Claude Code's Bash tool, PATH already carries the installed ora's
// bin, which would shadow this checkout's links in the PATH test.
const pathWithoutOra = (process.env.PATH ?? "")
  .split(":")
  .filter((dir) => !existsSync(join(dir, "..", "libexec/toolbox")))
  .join(":");

beforeAll(async () => {
  proc = Bun.spawn(["bun", "run", join(root, "mcp/server.ts")], {
    env: { ...process.env, PATH: pathWithoutOra },
    stdin: "pipe",
    stdout: "pipe",
    stderr: "inherit",
  });
  void readLoop();
  await request("initialize", {
    protocolVersion: "2025-11-25",
    capabilities: {},
    clientInfo: { name: "test", version: "0" },
  });
  proc.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized" })}\n`);
});

afterAll(() => proc.kill());

test("one always-loaded tool whose description fits Claude Code's 2,048-character cut", async () => {
  const { result } = await request("tools/list");
  expect(result.tools.map((t: any) => t.name)).toEqual(["run"]);
  expect(result.tools[0]._meta["anthropic/alwaysLoad"]).toBe(true);
  expect(result.tools[0].description.length).toBeLessThanOrEqual(2048);
});

test("cwd, variables and functions persist between calls", async () => {
  await run({ script: "cd /tmp && X=42 && greet() { echo hi-$1; }", session: "state" });
  expect(await run({ script: 'pwd; echo "$X"; greet you', session: "state" })).toStartWith(
    "/tmp\n42\nhi-you\n",
  );
});

test("stderr is merged in order and the exit code is reported", async () => {
  const out = await run({
    script: "echo out; echo err >&2; exit_code() { return 3; }; exit_code",
    session: "codes",
  });
  expect(out).toStartWith("out\nerr\n");
  expect(out).toContain("[exit 3 ");
});

test("a script that reads stdin gets EOF instead of swallowing later commands", async () => {
  expect(await run({ script: "cat; echo after-cat", session: "stdin" })).toStartWith("after-cat\n");
  expect(await run({ script: "echo still-alive", session: "stdin" })).toStartWith("still-alive\n");
});

test("CRLF lines keep their text, and a \\r redraw keeps only its last state", async () => {
  expect(
    await run({ script: "printf 'a\\r\\nb\\r\\nstep 1\\rstep 2\\n'", session: "crlf" }),
  ).toStartWith("a\nb\nstep 2\n");
});

test("a flood of output comes back whole and leaves the session usable", async () => {
  const out = await run({
    script: "head -c 50000000 /dev/zero | tr '\\0' x; echo; echo the-end",
    session: "flood",
  });
  expect(out.length).toBeGreaterThan(50_000_000);
  expect(out.slice(0, 10)).toBe("xxxxxxxxxx");
  expect(out.slice(-40)).toMatch(/x\nthe-end\n\[exit 0 · [\d.]+s\]$/);
  expect(await run({ script: "echo next", session: "flood" })).toStartWith("next\n");
}, 30_000);

test("a finished script leaves no file behind", async () => {
  await run({ script: "echo done", session: "files" });
  expect(readdirSync(join(tmpdir(), `ora-${proc.pid}`))).toEqual([]);
});

test("a script past its timeout keeps running and an empty call collects the rest", async () => {
  const first = await run({
    script: "echo started; sleep 2; echo finished",
    session: "slow",
    timeout_s: 1,
  });
  expect(first).toStartWith("started\n");
  expect(first).toContain("still running");
  const busy = await run({ script: "echo other", session: "slow" });
  expect(busy).toContain("still running an earlier script");
  const rest = await run({ script: "", session: "slow", timeout_s: 10 });
  expect(rest).toStartWith("finished\n");
  expect(rest).toContain("[exit 0 ");
});

test("reset kills a stuck script and starts a fresh shell", async () => {
  await run({ script: "Y=1; sleep 30", session: "stuck", timeout_s: 1 });
  const fresh = await run({ script: 'echo "y=${Y:-unset}"', session: "stuck", reset: true });
  expect(fresh).toStartWith("y=unset\n");
});

test("a script that exits the shell is reported, and the next call gets a new shell", async () => {
  expect(await run({ script: "exit 7", session: "exits" })).toContain(
    "the shell exited with code 7",
  );
  expect(await run({ script: "echo back", session: "exits" })).toStartWith("back\n");
});

test("separate sessions run in parallel; one session runs its calls one at a time", async () => {
  let start = Date.now();
  await Promise.all([
    run({ script: "sleep 1", session: "par-a" }),
    run({ script: "sleep 1", session: "par-b" }),
  ]);
  expect(Date.now() - start).toBeLessThan(1800);

  start = Date.now();
  const [a, b] = await Promise.all([
    run({ script: "sleep 1; echo one", session: "serial" }),
    run({ script: "sleep 1; echo two", session: "serial" }),
  ]);
  expect(Date.now() - start).toBeGreaterThanOrEqual(1900);
  expect(a).toStartWith("one\n");
  expect(b).toStartWith("two\n");
});

test("the toolbox is on PATH after any native copies", async () => {
  const out = await run({
    script: "command -v uctags; echo \"$PATH\" | tr : '\\n' | tail -1",
    session: "path",
  });
  const bin = join(root, "bin");
  expect(out).toStartWith(`${bin}/uctags\n${bin}\n`);
});

async function leftBehind(stop: (server: Bun.Subprocess<"pipe", "pipe", "inherit">) => void) {
  const server = Bun.spawn(["bun", "run", join(root, "mcp/server.ts")], {
    stdin: "pipe",
    stdout: "pipe",
    stderr: "inherit",
  });
  const send = (message: object) => {
    server.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", ...message })}\n`);
    server.stdin.flush();
  };
  send({
    id: 1,
    method: "initialize",
    params: {
      protocolVersion: "2025-11-25",
      capabilities: {},
      clientInfo: { name: "t", version: "0" },
    },
  });
  send({ method: "notifications/initialized" });
  send({
    id: 2,
    method: "tools/call",
    params: { name: "run", arguments: { script: "sleep 300 & echo pid=$!" } },
  });
  let text = "";
  for await (const chunk of server.stdout) {
    text += new TextDecoder().decode(chunk);
    if (text.includes("pid=")) break;
  }
  const pid = Number(text.match(/pid=(\d+)/)![1]);
  stop(server);
  await server.exited;
  await Bun.sleep(300);
  const workDir = existsSync(join(tmpdir(), `ora-${server.pid}`));
  try {
    process.kill(pid, 0);
    process.kill(pid, "SIGKILL");
    return { child: true, workDir };
  } catch {
    return { child: false, workDir };
  }
}

test("a server closed by its client takes its shells, their children and its files with it", async () => {
  expect(await leftBehind((server) => server.stdin.end())).toEqual({
    child: false,
    workDir: false,
  });
});

test("a server stopped by SIGTERM takes its shells, their children and its files with it", async () => {
  expect(await leftBehind((server) => server.kill("SIGTERM"))).toEqual({
    child: false,
    workDir: false,
  });
});
