# Ora

Agents for parallel execution, independent verification, and research isolation.

## Agents

| Agent          | Model  | Role                                                      |
| -------------- | ------ | --------------------------------------------------------- |
| **Ariadne**    | Sonnet | Codebase exploration — enhanced contextual grep           |
| **Clio**       | Sonnet | External research — docs, web, GitHub repos               |
| **Hephaestus** | Opus   | Autonomous deep work in isolated worktrees                |
| **Aletheia**   | Sonnet | Goal-backward verification — checks delivery against plan |

## Hooks

| Event       | Matcher                             | What it does                                                      |
| ----------- | ----------------------------------- | ----------------------------------------------------------------- |
| PostToolUse | `EnterPlanMode`                     | Injects reminder to load `/ora` workflow skill                    |
| PostToolUse | `Agent` (filtered `ora:Hephaestus`) | Injects reminder to run Aletheia verification before squash-merge |

Hooks use `hookSpecificOutput.additionalContext` to inject text into the model's conversation context.

## Skill

`/ora` — Workflow orchestration skill covering planning → execution → verification → review lifecycle. Loaded on demand when entering plan mode.

## Installation

```bash
/plugin marketplace add trancong12102/agentskills
/plugin install ora@agentskills
```

## Prerequisites

- **jq** — for parsing hook event JSON

## License

[MIT](../../LICENSE) — Cong Tran
