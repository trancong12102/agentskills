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

| Plugin                                 | Description                                                                                        |
| -------------------------------------- | -------------------------------------------------------------------------------------------------- |
| [ctx](./plugins/ctx)                   | Context-gathering agents — `finder` for codebase exploration, `librarian` for documentation lookup |
| [sound-notify](./plugins/sound-notify) | Play macOS notification sounds when Claude stops or asks a question                                |

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

## License

[MIT](./LICENSE) — Cong Tran
