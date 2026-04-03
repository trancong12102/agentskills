#!/usr/bin/env bash
# Send desktop notification via OSC 9
# Works directly in terminals (Ghostty, iTerm2, etc.) and inside tmux

read -r input

message=$(echo "$input" | jq -r '
  (.last_assistant_message // .message // "Done") | gsub("\n"; " ") | .[:100]
')

# Detect tmux even when $TMUX is unset (e.g. cleared for 256-color fix)
TMUX_TTY=$(tmux display-message -p '#{client_tty}' 2>/dev/null)
if [ -n "$TMUX_TTY" ]; then
  TTY="$TMUX_TTY"
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

# Kitty: skip notification if Kitty OS window has focus
_kitty_pid="${KITTY_PID}"
[ -z "$_kitty_pid" ] && _kitty_pid=$(tmux show-environment KITTY_PID 2>/dev/null | sed 's/^[^=]*=//')
if [ -n "$_kitty_pid" ]; then
  _os_focused=$(kitten @ --to "unix:/tmp/kitty-$_kitty_pid" ls 2>/dev/null | jq '.[0].is_focused')
  [ "$_os_focused" = "true" ] && exit 0
fi

# Bell (dock bounce + badge)
printf '\a' > "$TTY"

# Notification banner
if [ -n "$KITTY_PID" ]; then
  kitten notify --only-print-escape-code "Claude Code" "$message" > "$TTY"
else
  printf '\033]9;Claude Code: %s\007' "$message" > "$TTY"
fi
