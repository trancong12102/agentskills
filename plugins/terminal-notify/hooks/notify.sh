#!/usr/bin/env bash
# Send desktop notification via OSC 9
# Works directly in terminals (Ghostty, iTerm2, etc.) and inside tmux

read -r input

message=$(echo "$input" | jq -r '
  (.last_assistant_message // .message // "Done") | gsub("\n"; " ") | .[:100]
')

if [ -n "$TMUX" ]; then
  # Inside tmux: write directly to outer terminal TTY to bypass tmux interception
  TTY=$(tmux display-message -p '#{client_tty}')
else
  # Direct terminal: find TTY from parent process tree (hook subprocesses lack a controlling TTY)
  pid=$$
  while [ "$pid" -gt 1 ] 2>/dev/null; do
    t=$(ps -o tty= -p "$pid" 2>/dev/null | tr -d ' ')
    if [ -n "$t" ] && [ "$t" != "??" ]; then
      TTY="/dev/$t"
      break
    fi
    pid=$(ps -o ppid= -p "$pid" 2>/dev/null | tr -d ' ')
  done
fi

[ -z "$TTY" ] && exit 0

# Bell (dock bounce + badge) + OSC 9 (notification banner)
printf '\a' > "$TTY"
printf '\033]9;Claude Code: %s\007' "$message" > "$TTY"
