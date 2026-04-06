# Agent Skills

A collection of reusable skills for AI coding agents, mainly for Claude Code.

## Prerequisites

Some skills require API keys or external CLI authentication. Set them up before use:

| Skill                    | Credential                 | How to get                                                                      |
| ------------------------ | -------------------------- | ------------------------------------------------------------------------------- |
| `context7`               | `CONTEXT7_API_KEY` env var | Sign up at [context7.com](https://context7.com)                                 |
| `codebase-search`        | `MORPH_API_KEY` env var    | Sign up at [morphllm.com](https://morphllm.com)                                 |
| `github-codebase-search` | `MORPH_API_KEY` env var    | Same as above                                                                   |
| `oracle`                 | Codex CLI auth             | Run `codex login` after installing [Codex CLI](https://github.com/openai/codex) |
| `council-review`         | Codex CLI auth             | Same as above                                                                   |

### Codex CLI setup

The `oracle` and `council-review` skills require [Codex CLI](https://github.com/openai/codex) with an `oracle` profile. Add this to `~/.codex/config.toml`:

```toml
[profiles.oracle]
model = "gpt-5.4"
model_reasoning_effort = "high"
approval_policy = "never"
sandbox_mode = "read-only"
```

## Installation

### Skills

```bash
# All skills
bunx skills add trancong12102/agentskills -g -y -a claude-code

# Or individual skills
bunx skills add trancong12102/agentskills -g -y -a claude-code -s context7
bunx skills add trancong12102/agentskills -g -y -a claude-code -s council-review
bunx skills add trancong12102/agentskills -g -y -a claude-code -s deps-dev
bunx skills add trancong12102/agentskills -g -y -a claude-code -s oracle
```

### Plugins

| Plugin                                 | Description                                                                                    |
| -------------------------------------- | ---------------------------------------------------------------------------------------------- |
| [ora](./plugins/ora)                   | 6 specialized subagents for exploration, planning, and execution (see [Agents](#agents) below) |
| [sound-notify](./plugins/sound-notify) | Play macOS notification sounds when Claude stops or asks a question                            |

```shell
/plugin marketplace add trancong12102/agentskills
/plugin install ora@agentskills
/plugin install sound-notify@agentskills

# Enable auto-update
/plugin marketplace update agentskills
```

## Agents

The `ora` plugin ships 6 specialized subagents. Four are hook-enforced (automatically triggered at the right time), two are spawn-on-demand.

### On-Demand Agents

| Agent         | Model  | Description                                                                       |
| ------------- | ------ | --------------------------------------------------------------------------------- |
| `ora:Ariadne` | Sonnet | Codebase exploration — traces flows, finds implementations, maps architecture.    |
| `ora:Clio`    | Sonnet | External research — fetches docs, searches GitHub repos, looks up best practices. |

### Hook-Enforced Agents

| Agent            | Model  | Hook                     | Description                                                                                                                            |
| ---------------- | ------ | ------------------------ | -------------------------------------------------------------------------------------------------------------------------------------- |
| `ora:Metis`      | Opus   | PreToolUse EnterPlanMode | Intent classification + pre-analysis. Surfaces risks, generates directives, asks clarifying questions via AskUserQuestion.             |
| `ora:Momus`      | Sonnet | PreToolUse ExitPlanMode  | Plan validation for plans with 2+ steps. Checks executability, references, blockers. Approval-biased — rejects only for true blockers. |
| `ora:Atlas`      | Opus   | PostToolUse ExitPlanMode | Wave dispatch for plans with code tasks. Groups tasks into parallel waves, assigns agents, defines learning capture.                   |
| `ora:Hephaestus` | Opus   | Dispatched by Atlas      | Autonomous deep worker — receives a goal, works independently in a worktree, returns finished code with structured summary.            |

### Workflow

Research agents (Ariadne, Clio) are spawned on-demand throughout the workflow. Planning and execution agents are triggered automatically by hooks.

```text
User request
  │
  ▼
EnterPlanMode ────────────────────────────────────────────
  │  PreToolUse hook
  │  ├─ ora:Metis      — intent classification + directives
  │  └─ AskUserQuestion — clarify open questions from Metis
  │
  ▼
Plan mode (model writes plan) ────────────────────────────
  │  ora:Ariadne / ora:Clio spawned as needed for context
  │
  ▼
ExitPlanMode ─────────────────────────────────────────────
  │  PreToolUse hook
  │  └─ ora:Momus      — validate plan (2+ steps)
  │
  │  User approves plan
  │
  │  PostToolUse hook
  │  └─ ora:Atlas      — wave dispatch (code tasks)
  │
  ▼
Execution ────────────────────────────────────────────────
     ora:Hephaestus — dispatched by Atlas per code task
     ora:Ariadne    — dispatched by Atlas for exploration
     ora:Clio       — dispatched by Atlas for research
```

## License

[MIT](./LICENSE) — Cong Tran
