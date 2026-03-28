# Terminal Notify

Desktop notifications for Claude Code — works in any terminal that supports OSC 9 (Ghostty, iTerm2, Windows Terminal, etc.) and inside tmux.

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

**Direct terminal** — writes OSC 9 to the current TTY.
**Inside tmux** — bypasses tmux by writing directly to the outer terminal's TTY.

## Prerequisites

- A terminal that supports **OSC 9** notifications (Ghostty, iTerm2, Windows Terminal, etc.)
- **jq** — for parsing hook event JSON
- **tmux** (optional) — automatically detected and handled

## Installation

In Claude Code, add the marketplace and install:

```bash
/plugin marketplace add trancong12102/agentskills
/plugin install terminal-notify@agentskills
```

No configuration needed.

## License

[MIT](../../LICENSE) — Cong Tran
