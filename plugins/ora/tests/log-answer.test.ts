// End-to-end: pipes a SubagentStop payload into the real hook script, as Claude Code does,
// with HOME pointed at a scratch directory so the log it writes can be read back.
//   bun test plugins/ora/tests/log-answer.test.ts
import { beforeEach, expect, test } from "bun:test";
import { existsSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const hook = join(import.meta.dir, "..", "hooks/log-answer.sh");
let home: string;

beforeEach(() => {
  home = mkdtempSync(join(tmpdir(), "ora-log-"));
});

// Records shaped as Claude Code writes them to a subagent's transcript.
const prompt = (text: string) => ({ type: "user", message: { role: "user", content: text } });
const handback = (id: string, message: string) => ({
  type: "assistant",
  message: { role: "assistant", content: [{ type: "tool_use", id, name: "SubagentHandback", input: { message } }] },
});
const handbackResult = (id: string, success: boolean, message: string) => ({
  type: "user",
  message: {
    role: "user",
    content: [{ type: "tool_result", tool_use_id: id, content: [{ type: "text", text: JSON.stringify({ success, message }) }] }],
  },
});
const reply = (text: string) => ({ type: "assistant", message: { role: "assistant", content: [{ type: "text", text }] } });

function transcript(...records: object[]): string {
  const path = join(home, "agent.jsonl");
  writeFileSync(path, records.map((r) => JSON.stringify(r)).join("\n") + "\n");
  return path;
}

function stop(payload: object) {
  const proc = Bun.spawnSync(["bash", hook], {
    stdin: new TextEncoder().encode(JSON.stringify(payload)),
    env: { ...process.env, HOME: home, ORA_ANSWER_LOG: "1" },
  });
  const log = join(home, ".claude/ora/answers.jsonl");
  const rows = existsSync(log) ? readFileSync(log, "utf8").trim().split("\n").map((l) => JSON.parse(l)) : [];
  return { exitCode: proc.exitCode, rows };
}

const event = (agent_transcript_path: string, last_assistant_message: string) => ({
  hook_event_name: "SubagentStop",
  agent_type: "ora:research",
  agent_id: "a1",
  session_id: "s1",
  stop_hook_active: false,
  cwd: "/tmp",
  agent_transcript_path,
  last_assistant_message,
});

test("a delivered handback is logged as the answer, not the line the agent writes after it", () => {
  const path = transcript(
    prompt("What is X?"),
    handback("h1", "<conclusion>X is 4.2.</conclusion>"),
    handbackResult("h1", true, "Report delivered to your caller."),
    reply("I've sent the report to the calling agent."),
  );
  const { exitCode, rows } = stop(event(path, "I've sent the report to the calling agent."));
  expect(exitCode).toBe(0);
  expect(rows).toHaveLength(1);
  expect(rows[0].question).toBe("What is X?");
  expect(rows[0].answer).toBe("<conclusion>X is 4.2.</conclusion>");
});

test("a refused handback sent nothing, so the final text is the answer", () => {
  const path = transcript(
    prompt("What is X?"),
    handback("h1", "draft report"),
    handbackResult("h1", false, "Nothing was sent: SubagentHandback is not active for this agent."),
    reply("<conclusion>X is 4.2.</conclusion>"),
  );
  expect(stop(event(path, "<conclusion>X is 4.2.</conclusion>")).rows[0].answer).toBe(
    "<conclusion>X is 4.2.</conclusion>",
  );
});

test("an agent continued after its first report is logged with the latest one", () => {
  const path = transcript(
    prompt("What is X?"),
    handback("h1", "first report"),
    handbackResult("h1", true, "Report delivered to your caller."),
    prompt("And Y?"),
    handback("h2", "second report"),
    handbackResult("h2", true, "Report delivered to your caller."),
    reply("Sent."),
  );
  const row = stop(event(path, "Sent.")).rows[0];
  expect(row.question).toBe("What is X?");
  expect(row.answer).toBe("second report");
});

test("without a transcript on disk the payload's last message is still logged", () => {
  const { exitCode, rows } = stop(event(join(home, "missing.jsonl"), "the answer"));
  expect(exitCode).toBe(0);
  expect(rows[0].question).toBeNull();
  expect(rows[0].answer).toBe("the answer");
});

test("a corrupt transcript line neither drops the row nor fails the hook", () => {
  const path = join(home, "agent.jsonl");
  writeFileSync(path, `${JSON.stringify(prompt("What is X?"))}\n{not json\n`);
  const { exitCode, rows } = stop(event(path, "the answer"));
  expect(exitCode).toBe(0);
  expect(rows[0].answer).toBe("the answer");
});

test("other agents are not logged", () => {
  const { exitCode, rows } = stop({ ...event("", "x"), agent_type: "general-purpose" });
  expect(exitCode).toBe(0);
  expect(rows).toHaveLength(0);
});

test("a payload that is not JSON exits 0 and writes nothing", () => {
  const proc = Bun.spawnSync(["bash", hook], {
    stdin: new TextEncoder().encode("not json"),
    env: { ...process.env, HOME: home },
  });
  expect(proc.exitCode).toBe(0);
  expect(existsSync(join(home, ".claude/ora/answers.jsonl"))).toBe(false);
});
