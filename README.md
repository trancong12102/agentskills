# Agent Skills

A collection of reusable skills for AI coding agents, mainly for Claude Code.

## Prerequisites

The `oracle` and `council-review` skills require [Codex CLI](https://github.com/openai/codex) with an `oracle` profile. Add this to `~/.codex/config.toml`:

```toml
[profiles.oracle]
model = "gpt-5.4"
model_reasoning_effort = "high"
approval_policy = "never"
sandbox_mode = "read-only"
```

## Installation

Install all skills:

```bash
bunx skills add trancong12102/agentskills -g -y -a claude-code
```

Or install individual skills:

```bash
bunx skills add trancong12102/agentskills -g -y -a claude-code -s context7
bunx skills add trancong12102/agentskills -g -y -a claude-code -s council-review
bunx skills add trancong12102/agentskills -g -y -a claude-code -s deps-dev
bunx skills add trancong12102/agentskills -g -y -a claude-code -s oracle
```

## Plugins

| Plugin                                 | Description                                                                   |
| -------------------------------------- | ----------------------------------------------------------------------------- |
| [ctx](./plugins/ctx)                   | Context-gathering agents and plan quality gates (see [Agents](#agents) below) |
| [sound-notify](./plugins/sound-notify) | Play macOS notification sounds when Claude stops or asks a question           |

Install plugins in Claude Code:

```shell
/plugin marketplace add trancong12102/agentskills
/plugin install ctx@agentskills
/plugin install sound-notify@agentskills
```

Enable auto-update to get the latest plugin versions on startup:

```shell
/plugin marketplace update agentskills
```

Then select **Enable auto-update** when prompted.

## Agents

The `ctx` plugin ships four specialized subagents, two for context gathering and two for plan quality gates:

| Agent           | Role         | Model  | Description                                                                                                                                                                                                                    |
| --------------- | ------------ | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `ctx:finder`    | Context      | Sonnet | Codebase exploration — traces flows, finds implementations, maps architecture. Enhanced contextual grep with structured output.                                                                                                |
| `ctx:librarian` | Context      | Sonnet | Documentation & remote code lookup — fetches official docs, searches public GitHub repos, finds best practices.                                                                                                                |
| `ctx:metis`     | Pre-planning | Opus   | Analyzes requests before planning. Classifies intent (Refactoring / Build / Mid-sized / Collaborative / Architecture / Research), surfaces hidden requirements, flags AI-slop risks, and generates directives for the planner. |
| `ctx:momus`     | Plan review  | Sonnet | Reviews plans after planning. Checks reference validity, task executability, critical blockers, and QA criteria. Strong approval bias — rejects only for true blockers (max 3 issues).                                         |

### Plan quality hooks

The plugin includes two hooks that create a plan → review → improve feedback loop:

- **`PreToolUse:EnterPlanMode`** — Before entering plan mode, suggests consulting `ctx:metis` for complex or ambiguous requests.
- **`PreToolUse:ExitPlanMode`** — Before exiting plan mode, triggers parallel review by `ctx:momus` and the `oracle` skill (GPT-5.4 via Codex CLI). If either reviewer rejects, the plan is refined before implementation begins.

```txt
Request → [metis] → Plan mode → Write plan → [momus + oracle] → Review
                                                    ↓
                                          Both OKAY → Implement
                                          Any REJECT → Revise plan → Re-review
```

## License

[MIT](./LICENSE) — Cong Tran
