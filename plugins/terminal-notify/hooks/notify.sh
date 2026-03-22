#!/usr/bin/env bash
# Send desktop notification via OSC 9
# Works directly in terminals (Ghostty, iTerm2, etc.) and inside tmux

read -r input

message=$(echo "$input" | jq -r '
  (.last_assistant_message // .message // "Done") | gsub("\n"; " ") | .[:100]
')

if [ -n "$TMUX" ]; then
  # Inside tmux: write directly to outer terminal TTY to bypass tmux interception
  CLIENT_TTY=$(tmux display-message -p '#{client_tty}')
  [ -z "$CLIENT_TTY" ] && exit 0
  printf '\a' > "$CLIENT_TTY"
  printf '\033]9;Claude Code: %s\007' "$message" > "$CLIENT_TTY"
else
  # Direct terminal: write to current TTY
  TTY=$(tty 2>/dev/null || echo /dev/tty)
  printf '\a' > "$TTY"
  printf '\033]9;Claude Code: %s\007' "$message" > "$TTY"
fi
