#!/bin/bash
# PreToolUse:EnterPlanMode — block if ora:Metis was not spawned in this session.
TRANSCRIPT=$(jq -r '.transcript_path')

if [ -z "$TRANSCRIPT" ] || [ ! -f "$TRANSCRIPT" ]; then
  exit 0 # can't verify — allow
fi

if grep -q '"ora:Metis"' "$TRANSCRIPT"; then
  exit 0
fi

echo "ora:Metis has not been called in this session. Spawn Agent(subagent_type='ora:Metis') with the user's request before entering plan mode." >&2
exit 2
