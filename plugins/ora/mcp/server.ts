import { mkdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { McpServer } from "@modelcontextprotocol/server";
import { serveStdio } from "@modelcontextprotocol/server/stdio";
import { z } from "zod";

const PLUGIN_BIN = join(import.meta.dir, "..", "bin");
const WORK_DIR = join(tmpdir(), `ora-${process.pid}`);
const NONCE = crypto.randomUUID().replaceAll("-", "");
const IDLE_MS = 30 * 60_000;

mkdirSync(WORK_DIR, { recursive: true });

const DESCRIPTION = `Run a bash script in a persistent shell and get its combined stdout and stderr. Use it to search, read and analyse code, data and documents, and to fetch from the web; pipe and filter inside the script so only what answers the question is printed. Commands that change the user's files or git state belong in the Bash tool, where the user's permission rules apply.

On PATH, each installed on first use if missing (\`<cmd> --help\` for flags):
- code search: rg, ugrep, fd, bfs, ast-grep (structural search; \`ast-grep outline <path>\` lists symbols), uctags (Universal Ctags, JSON symbol tables), scc, difft
- code intelligence for JS/TS, Python and Go, once the project's dependencies are installed: scip, scip-typescript, scip-python, scip-go, depcruise, knip
- data: jq, yq, duckdb (SQL over JSON, CSV, Parquet), syft (dependency inventory)
- web and documents: tinyfish (authenticated; \`tinyfish fetch content get <url>...\` fetches up to 10 URLs in parallel as JSON, each page's markdown in \`.results[].text\`; \`tinyfish search query <q>\`; browser agents), lightpanda (\`lightpanda fetch --dump markdown <url>\` renders JavaScript pages), markitdown (PDF and Office to markdown), pdftotext, pdfinfo, yt-dlp, hf (\`hf papers read <arxiv-id>\`)
- hosts and runtimes: gh, glab, git, curl, uv/uvx, bun/bunx
Anything else: \`mise x <tool>@latest -- <cmd>\`, cached for later sessions.

The shell keeps its working directory, variables and functions between calls. Calls with the same \`session\` share that state and run one at a time; give independent or parallel work its own \`session\` name. A script still running at \`timeout_s\` keeps running: call again with an empty script to wait for it.`;

type Session = {
  proc: Bun.Subprocess<"pipe", "pipe", "pipe">;
  buf: string;
  notify: (() => void) | null;
  marker: string | null;
  lastUsed: number;
  exited: number | null;
};

const sessions = new Map<string, Session>();
// Chained per name, so calls to one session queue even while it is being created.
const locks = new Map<string, Promise<unknown>>();
let counter = 0;

function startSession(): Session {
  const proc = Bun.spawn(["bash", "--noprofile", "--norc"], {
    // Its own process group, so killing the group also ends whatever the script left running.
    detached: true,
    cwd: process.env.CLAUDE_PROJECT_DIR || process.cwd(),
    stdin: "pipe",
    stdout: "pipe",
    stderr: "pipe",
    env: {
      ...process.env,
      // Appended, so a native copy of any tool wins over the mise-backed link.
      PATH: `${process.env.PATH ?? "/usr/bin:/bin"}:${PLUGIN_BIN}`,
      PAGER: "cat",
      GIT_PAGER: "cat",
      GH_PAGER: "cat",
      GLAB_PAGER: "cat",
      MANPAGER: "cat",
      GIT_TERMINAL_PROMPT: "0",
      NO_COLOR: "1",
      TERM: "dumb",
      // jiti (knip, config loaders) otherwise caches into the explored repo's node_modules/.cache.
      JITI_FS_CACHE: "false",
    },
  });
  const session: Session = {
    proc,
    buf: "",
    notify: null,
    marker: null,
    lastUsed: Date.now(),
    exited: null,
  };
  const pump = async (stream: ReadableStream<Uint8Array>) => {
    const decoder = new TextDecoder();
    for await (const chunk of stream) {
      session.buf += decoder.decode(chunk, { stream: true });
      session.notify?.();
    }
  };
  void pump(proc.stdout);
  void pump(proc.stderr);
  void proc.exited.then((code) => {
    session.exited = code;
    session.notify?.();
  });
  return session;
}

function kill(session: Session) {
  try {
    process.kill(-session.proc.pid, "SIGKILL");
  } catch {
    // The group is already gone.
  }
}

function clean(text: string): string {
  return (
    text
      .replace(/\x1b\[[0-9;?]*[A-Za-z]/g, "")
      // Progress bars redraw with \r; keep only what the line finally said.
      .split("\n")
      .map((line) => line.slice(line.lastIndexOf("\r") + 1))
      .join("\n")
  );
}

async function waitForMarker(session: Session, timeoutMs: number) {
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    const at = session.marker ? session.buf.indexOf(session.marker) : -1;
    if (at !== -1) {
      const end = session.buf.indexOf("\n", at);
      const code = Number(session.buf.slice(at + session.marker!.length, end).trim());
      const output = session.buf.slice(0, at);
      session.buf = session.buf.slice(end + 1);
      session.marker = null;
      return { output, code, done: true as const };
    }
    if (session.exited !== null) {
      const output = session.buf;
      session.buf = "";
      return { output, code: session.exited, done: "exited" as const };
    }
    const left = deadline - Date.now();
    if (left <= 0) {
      const output = session.buf;
      session.buf = "";
      return { output, code: null, done: false as const };
    }
    await new Promise<void>((resolve) => {
      const timer = setTimeout(resolve, left);
      session.notify = () => {
        clearTimeout(timer);
        resolve();
      };
    });
    session.notify = null;
  }
}

async function run(name: string, script: string, timeoutS: number, reset: boolean) {
  if (reset) {
    const old = sessions.get(name);
    if (old) kill(old);
    sessions.delete(name);
  }
  let session = sessions.get(name);
  if (!session) {
    session = startSession();
    sessions.set(name, session);
  }
  session.lastUsed = Date.now();
  const started = Date.now();

  if (session.marker && script.trim()) {
    return `[session "${name}" is still running an earlier script; call with an empty script to wait for it, or with reset: true to kill it and start a fresh shell]`;
  }
  if (!session.marker) {
    if (!script.trim()) return "[nothing is running in this session]";
    const file = join(WORK_DIR, `script-${++counter}.sh`);
    writeFileSync(file, script);
    session.marker = `__ORA_${NONCE}_${counter}__`;
    // stdin is the control pipe; a script that read it would swallow the commands after it.
    session.proc.stdin.write(
      `source '${file}' 2>&1 </dev/null; __ora_rc=$?; printf '\\n%s %s\\n' '${session.marker}' "$__ora_rc"\n`,
    );
    session.proc.stdin.flush();
  }

  const result = await waitForMarker(session, timeoutS * 1000);
  const seconds = ((Date.now() - started) / 1000).toFixed(1);
  let trailer: string;
  if (result.done === true) trailer = `[exit ${result.code} · ${seconds}s]`;
  else if (result.done === "exited") {
    sessions.delete(name);
    trailer = `[the shell exited with code ${result.code}; the next call starts a fresh one]`;
  } else
    trailer = `[still running after ${seconds}s; call again with an empty script to wait for it]`;

  const output = clean(result.output).trimEnd();
  return output ? `${output}\n${trailer}` : trailer;
}

setInterval(() => {
  const now = Date.now();
  for (const [name, session] of sessions) {
    if (!session.marker && now - session.lastUsed > IDLE_MS) {
      kill(session);
      sessions.delete(name);
    }
  }
}, 60_000).unref();

// Detached shells outlive this process unless it takes them down on the way out.
process.on("exit", () => {
  for (const session of sessions.values()) kill(session);
});
for (const signal of ["SIGTERM", "SIGINT", "SIGHUP"] as const) {
  process.on(signal, () => process.exit(0));
}
// The shells' pipes keep the event loop alive after the client hangs up.
process.stdin.on("end", () => process.exit(0));

serveStdio(() => {
  const server = new McpServer({ name: "ora", version: "4.0.0" }, { capabilities: { tools: {} } });
  server.registerTool(
    "run",
    {
      description: DESCRIPTION,
      inputSchema: z.object({
        script: z.string().describe("Bash to run. Empty waits for a script that is still running."),
        session: z
          .string()
          .default("main")
          .describe(
            "Shell to run in. Each name has its own working directory, variables and queue.",
          ),
        timeout_s: z
          .number()
          .int()
          .min(1)
          .max(600)
          .default(120)
          .describe("Seconds to wait before returning; the script keeps running past it."),
        reset: z
          .boolean()
          .default(false)
          .describe("Kill this session's shell and start a fresh one first."),
      }),
      _meta: { "anthropic/alwaysLoad": true },
    },
    async ({ script, session, timeout_s, reset }) => {
      const job = (locks.get(session) ?? Promise.resolve()).then(() =>
        run(session, script, timeout_s, reset),
      );
      locks.set(
        session,
        job.catch(() => undefined),
      );
      const text = await job;
      return { content: [{ type: "text", text }] };
    },
  );
  return server;
});
