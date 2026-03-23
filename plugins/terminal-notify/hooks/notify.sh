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

# Kitty: skip notification if this pane/tab has focus
if [ -n "$KITTY_LISTEN_ON" ] && [ -n "$KITTY_WINDOW_ID" ]; then
  focused=$(kitten @ --to "$KITTY_LISTEN_ON" ls 2>/dev/null \
    | jq --argjson wid "$KITTY_WINDOW_ID" '[.[].tabs[].windows[] | select(.id == $wid)][0].is_focused')
  [ "$focused" = "true" ] && exit 0
fi

# Bell (dock bounce + badge)
printf '\a' > "$TTY"

# Notification banner
if [ -n "$KITTY_PID" ]; then
  kitten notify --only-print-escape-code "Claude Code" "$message" > "$TTY"
else
  printf '\033]9;Claude Code: %s\007' "$message" > "$TTY"
fi
