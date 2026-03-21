# tmux Notify

Send desktop notifications from Claude Code inside tmux to the outer terminal — no need to keep watching the terminal.

tmux intercepts notification escape sequences, so Claude Code's built-in notifications don't work inside tmux. This plugin bypasses tmux by writing directly to the outer terminal's TTY.

## How It Works

| Event               | When                                  |
| ------------------- | ------------------------------------- |
| **Stop**            | Claude finishes its response          |
| **Notification**    | Claude sends a notification           |
| **AskUserQuestion** | Claude is about to ask you a question |
| **ExitPlanMode**    | Claude exits plan mode                |

Each notification sends:
- **Bell** — dock bounce + badge (macOS)
- **OSC 9** — desktop notification banner with the last response (truncated to 100 chars)

All hooks run asynchronously so they never block Claude's response.

## Prerequisites

- **tmux** — the plugin is a no-op outside tmux
- A terminal that supports **OSC 9** notifications (Ghostty, iTerm2, Windows Terminal, etc.)
- **jq** — for parsing hook event JSON

## Installation

In Claude Code, add the marketplace and install:

```bash
/plugin marketplace add trancong12102/agentskills
/plugin install tmux-notify@agentskills
```

No configuration needed.

## License

[MIT](../../LICENSE) — Cong Tran
