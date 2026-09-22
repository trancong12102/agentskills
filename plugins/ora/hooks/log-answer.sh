#!/usr/bin/env bash
# Records ora:explore and ora:research answers for offline auditing. ORA_ANSWER_LOG=0 turns it off.
# A hook that fails stalls a turn, so every path here exits 0.
[ "${ORA_ANSWER_LOG:-1}" = "0" ] && exit 0
command -v python3 >/dev/null 2>&1 || exit 0
python3 "$(dirname "$0")/log-answer.py" 2>/dev/null
exit 0
