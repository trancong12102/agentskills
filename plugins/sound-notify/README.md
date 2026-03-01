# Sound Notify

Play notification sounds in Claude Code so you know when Claude is done or needs your attention — no need to keep watching the terminal.

## How It Works

| Event               | Sound     | When                                  |
| ------------------- | --------- | ------------------------------------- |
| **Stop**            | Submarine | Claude finishes its response          |
| **AskUserQuestion** | Glass     | Claude is about to ask you a question |

Both sounds play asynchronously so they never block Claude's response.

## Prerequisites

- **macOS** — uses `afplay` and built-in system sounds (`/System/Library/Sounds/`)

## Installation

In Claude Code, add the marketplace and install:

```bash
/plugin marketplace add trancong12102/agentskills
/plugin install sound-notify@agentskills
```

No configuration needed.

## License

[MIT](../../LICENSE) — Cong Tran
