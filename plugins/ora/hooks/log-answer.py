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


def question_of(transcript):
    """The prompt the subagent was given: the first user message of its own transcript.

    Returns None under --no-session-persistence, where no transcript is written to disk.
    """
    try:
        with open(transcript, encoding="utf-8") as fh:
            for line in fh:
                record = json.loads(line)
                if record.get("type") != "user":
                    continue
                content = record.get("message", {}).get("content")
                if isinstance(content, str):
                    return content
                if isinstance(content, list):
                    return "".join(
                        block.get("text", "")
                        for block in content
                        if isinstance(block, dict)
                    )
    except (OSError, ValueError):
        pass
    return None


def main():
    event = json.load(sys.stdin)
    if event.get("agent_type") not in AGENTS:
        return

    log = pathlib.Path.home() / ".claude" / "ora" / "answers.jsonl"
    if log.exists() and log.stat().st_size > MAX_BYTES:
        return
    log.parent.mkdir(parents=True, exist_ok=True)

    row = {
        "ts": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"),
        "agent": event.get("agent_type"),
        "agent_id": event.get("agent_id"),
        "session_id": event.get("session_id"),
        "stop_hook_active": event.get("stop_hook_active"),
        "cwd": event.get("cwd"),
        "question": question_of(event.get("agent_transcript_path") or ""),
        "answer": event.get("last_assistant_message"),
    }
    with log.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(row, ensure_ascii=False) + "\n")


try:
    main()
except Exception:
    pass
