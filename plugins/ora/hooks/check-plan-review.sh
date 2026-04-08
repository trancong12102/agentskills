#!/bin/bash
# PreToolUse:ExitPlanMode — remind about Momus/Atlas if not yet called.
TRANSCRIPT=$(jq -r '.transcript_path')

if [ -z "$TRANSCRIPT" ] || [ ! -f "$TRANSCRIPT" ]; then
  exit 0
fi

missing=()
grep -q '"ora:Momus"' "$TRANSCRIPT" || missing+=("ora:Momus")
grep -q '"ora:Atlas"' "$TRANSCRIPT" || missing+=("ora:Atlas")

if [ ${#missing[@]} -eq 0 ]; then
  exit 0
fi

# Non-blocking reminder via additionalContext
cat <<EOF
{
  "hookSpecificOutput": {
    "hookEventName": "PreToolUse",
    "permissionDecision": "allow"
  },
  "additionalContext": "Pipeline check: ${missing[*]} not yet called. Momus validates multi-step plans (skip for 1-step). Atlas produces wave dispatch (skip for pure research). If applicable, spawn them before exiting plan mode."
}
EOF
exit 0
