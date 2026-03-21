#!/usr/bin/env bash
# Send notification to outer terminal, bypassing tmux
# Works with any terminal that supports OSC 9 (Ghostty, iTerm2, Windows Terminal, etc.)
[ -z "$TMUX" ] && exit 0

read -r input

message=$(echo "$input" | jq -r '
  (.last_assistant_message // .message // "Done") | gsub("\n"; " ") | .[:100]
')

CLIENT_TTY=$(tmux display-message -p '#{client_tty}')
[ -z "$CLIENT_TTY" ] && exit 0

# Bell (dock bounce + badge) + OSC 9 (notification banner)
printf '\a' > "$CLIENT_TTY"
printf '\033]9;Claude Code: %s\007' "$message" > "$CLIENT_TTY"
