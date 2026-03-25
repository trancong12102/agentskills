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

| Plugin                                 | Description                                                                                    |
| -------------------------------------- | ---------------------------------------------------------------------------------------------- |
| [ora](./plugins/ora)                   | 7 specialized subagents for exploration, planning, and execution (see [Agents](#agents) below) |
| [sound-notify](./plugins/sound-notify) | Play macOS notification sounds when Claude stops or asks a question                            |

Install plugins in Claude Code:

```shell
/plugin marketplace add trancong12102/agentskills
/plugin install ora@agentskills
/plugin install sound-notify@agentskills
```

Enable auto-update to get the latest plugin versions on startup:

```shell
/plugin marketplace update agentskills
```

Then select **Enable auto-update** when prompted.

## Agents

The `ora` plugin ships 7 specialized subagents organized across three phases:

### Research

| Agent         | Model  | Description                                                                       |
| ------------- | ------ | --------------------------------------------------------------------------------- |
| `ora:Ariadne` | Sonnet | Codebase exploration — traces flows, finds implementations, maps architecture.    |
| `ora:Clio`    | Sonnet | External research — fetches docs, searches GitHub repos, looks up best practices. |

### Planning

| Agent            | Model  | Description                                                                                                                                                                                                                     |
| ---------------- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ora:Prometheus` | Opus   | Interview-style planner — gathers codebase context, then asks targeted questions to clarify scope before producing a structured plan. Two-phase invocation: Phase 1 returns questions, Phase 2 synthesizes a plan from answers. |
| `ora:Metis`      | Opus   | Pre-plan risk analysis — classifies intent, surfaces hidden requirements, flags AI-slop risks, and generates directives for the planner.                                                                                        |
| `ora:Momus`      | Sonnet | Plan validation — checks reference validity, task executability, and critical blockers. Strong approval bias — rejects only for true blockers (max 3 issues). Auto-triggered by ExitPlanMode hook on plans with 4+ steps.       |

### Execution

| Agent            | Model | Description                                                                                                                                                                                                |
| ---------------- | ----- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ora:Atlas`      | Opus  | Plan conductor — organizes tasks into parallel waves, assigns agents per task, and accumulates learnings between waves so each task benefits from what came before. Re-invoked between waves with results. |
| `ora:Hephaestus` | Opus  | Autonomous deep worker — receives a goal, works independently in a worktree, returns finished code with a structured summary. First ora agent with write access.                                           |

### Workflow

Each step is optional — simple tasks skip straight to execution.

```text
User request
  │
  ├─ Unclear scope? ──▶ ora:Prometheus (interview)
  │                          │
  ├─ Complex task? ────▶ ora:Metis (risk analysis)
  │                          │
  ├─ Plan mode ────────▶ ora:Momus (validation, auto-triggered)
  │                          │
  ├─ Multi-task plan? ─▶ ora:Atlas (wave dispatch)
  │                          │
  └─ Execute ──────────▶ ora:Hephaestus (worktree)
                         ora:Ariadne (research)
                         Direct (simple tasks)
```

## License

[MIT](./LICENSE) — Cong Tran
