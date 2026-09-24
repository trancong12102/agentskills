#!/usr/bin/env python3
"""Append one ora:explore or ora:research answer to ~/.claude/ora/answers.jsonl.

The log lives outside the plugin because a plugin update replaces its directory.
SubagentStop can fire more than once for one subagent, so every row carries
`agent_id` and `stop_hook_active` for the reader to deduplicate on.
"""

import datetime
import json
import pathlib
import sys

AGENTS = {"ora:explore", "ora:research"}
MAX_BYTES = 32_000_000


def text_of(content):
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        return "".join(block.get("text", "") for block in content if isinstance(block, dict))
    return ""


def delivered(result):
    try:
        return json.loads(text_of(result.get("content"))).get("success") is True
    except (ValueError, AttributeError):
        return False


def read_transcript(transcript):
    """The prompt the subagent was given and the report it handed back, from its own transcript.

    A subagent that delivers its report through the SubagentHandback tool ends on a line like
    "I've sent the report to the caller", which is all `last_assistant_message` carries. The
    report is the `message` of the last handback the tool accepted; a refused one ("not active
    for this agent") sends nothing, and the agent then writes the report as its final text.

    Both are None under --no-session-persistence, where no transcript is written to disk.
    """
    question, handbacks, report = None, {}, None
    try:
        with open(transcript, encoding="utf-8") as fh:
            for line in fh:
                record = json.loads(line)
                content = record.get("message", {}).get("content")
                if record.get("type") == "user" and question is None:
                    question = text_of(content)
                if not isinstance(content, list):
                    continue
                for block in content:
                    if not isinstance(block, dict):
                        continue
                    if block.get("type") == "tool_use" and block.get("name") == "SubagentHandback":
                        handbacks[block.get("id")] = (block.get("input") or {}).get("message")
                    elif block.get("type") == "tool_result" and block.get("tool_use_id") in handbacks:
                        if delivered(block):
                            report = handbacks[block["tool_use_id"]]
    except (OSError, ValueError):
        pass
    return question, report


def main():
    event = json.load(sys.stdin)
    if event.get("agent_type") not in AGENTS:
        return

    log = pathlib.Path.home() / ".claude" / "ora" / "answers.jsonl"
    if log.exists() and log.stat().st_size > MAX_BYTES:
        return
    log.parent.mkdir(parents=True, exist_ok=True)

    question, report = read_transcript(event.get("agent_transcript_path") or "")
    row = {
        "ts": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"),
        "agent": event.get("agent_type"),
        "agent_id": event.get("agent_id"),
        "session_id": event.get("session_id"),
        "stop_hook_active": event.get("stop_hook_active"),
        "cwd": event.get("cwd"),
        "question": question,
        "answer": report or event.get("last_assistant_message"),
    }
    with log.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(row, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    try:
        main()
    except Exception:
        pass
