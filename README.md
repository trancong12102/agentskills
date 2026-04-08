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

The `ora` plugin ships 6 specialized subagents and 2 hook-based safety nets. Two agents are spawn-on-demand, four are part of the planning pipeline.

### On-Demand Agents

| Agent         | Model  | Description                                                                       |
| ------------- | ------ | --------------------------------------------------------------------------------- |
| `ora:Ariadne` | Sonnet | Codebase exploration — traces flows, finds implementations, maps architecture.    |
| `ora:Clio`    | Sonnet | External research — fetches docs, searches GitHub repos, looks up best practices. |

### Pipeline Agents

| Agent            | Model  | When                 | Description                                                                                                                            |
| ---------------- | ------ | -------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `ora:Metis`      | Opus   | Before EnterPlanMode | Intent classification + pre-analysis. Surfaces risks, generates directives, asks clarifying questions via AskUserQuestion.             |
| `ora:Momus`      | Sonnet | Before ExitPlanMode  | Plan validation for plans with 2+ steps. Checks executability, references, blockers. Approval-biased — rejects only for true blockers. |
| `ora:Atlas`      | Opus   | Before ExitPlanMode  | Wave dispatch for plans with code tasks. Groups tasks into parallel waves, assigns agents, defines learning capture.                   |
| `ora:Hephaestus` | Opus   | Dispatched by Atlas  | Autonomous deep worker — receives a goal, works independently in a worktree, returns finished code with structured summary.            |

### Hooks

Two command hooks act as safety nets — they grep the session transcript to verify agents were called in the right order.

| Hook                       | Script                 | Behavior                                                                   |
| -------------------------- | ---------------------- | -------------------------------------------------------------------------- |
| `PreToolUse:EnterPlanMode` | `check-metis.sh`       | **Blocks** if `ora:Metis` was not spawned in this session.                 |
| `PreToolUse:ExitPlanMode`  | `check-plan-review.sh` | **Reminds** (non-blocking) if `ora:Momus` or `ora:Atlas` were not spawned. |

### Workflow

Research agents (Ariadne, Clio) are spawned on-demand throughout the workflow. Pipeline agents are called by CLAUDE.md instructions and verified by hooks.

```text
User request
  │
  ▼
ora:Metis ────────────────────────────────────────────────
  │  Intent classification + directives
  │  AskUserQuestion for open questions from Metis
  │
  ▼
EnterPlanMode ────────────────────────────────────────────
  │  Hook: check-metis.sh (blocks if Metis not called)
  │
  ▼
Plan mode (model writes plan) ────────────────────────────
  │  ora:Ariadne / ora:Clio spawned as needed for context
  │
  ▼
ora:Momus + ora:Atlas ────────────────────────────────────
  │  Momus validates plan (skip for 1-step plans)
  │  Atlas produces wave dispatch (skip for pure research)
  │
  ▼
ExitPlanMode ─────────────────────────────────────────────
  │  Hook: check-plan-review.sh (reminds if Momus/Atlas not called)
  │  User approves plan
  │
  ▼
Execution ────────────────────────────────────────────────
     ora:Hephaestus — dispatched by Atlas per code task
     ora:Ariadne    — dispatched by Atlas for exploration
     ora:Clio       — dispatched by Atlas for research
```

## CLAUDE.md

Skills and ora agents work best when Claude Code is instructed to use them proactively. Add these overrides to your global `~/.claude/CLAUDE.md`:

```markdown
# User behavioral overrides

<investigate_before_responding>
Do not respond to technical tasks using training data alone — it is from
mid-2025 and likely outdated. Verify with a matching skill or ora:Clio research
first. The cost of loading a skill or spawning a research agent is near zero —
the cost of outdated knowledge is a broken implementation.
</investigate_before_responding>

<use_skills_proactively>
Do not write code or give technical advice without first loading a matching
skill. Skip only when the task is clearly unrelated to any available skill
(e.g., git operations, file renaming, simple config edits).
</use_skills_proactively>

<subagent_routing>
Do not use the built-in Explore agent or general-purpose agent if an ora agent
can handle the task. Reserve Glob and Grep for simple, targeted searches
(specific file, class, or function by name).
</subagent_routing>

<plan_before_implementing>
Do not implement without entering plan mode first. Skip only for truly trivial
tasks — single-file edits, renaming, simple config changes, typo fixes. If a
task touches 2+ files or has any ambiguity, plan first.
</plan_before_implementing>

<workflow>
Do not enter plan mode without running ora:Metis first. Do not exit plan mode
without running ora:Momus and ora:Atlas first (exceptions below). This is
enforced by hooks — skipping steps will block the tool call.

1. Spawn ora:Metis with the user's full request. Wait for its directives before
   proceeding.
2. If Metis returns "Questions for User", use AskUserQuestion to ask them — do
   not present questions as plain text. Write the user's answers into the plan
   as key decisions.
3. Enter plan mode. Incorporate Metis directives (intent, pre-analysis,
   constraints) into the plan.
4. Before exiting plan mode, spawn ora:Momus with the full plan text. Fix any
   issues it rejects. Skip only for 1-step plans.
5. Spawn ora:Atlas with the full plan text. It returns a wave dispatch assigning
   tasks to agents. Skip only for pure research with no code changes.
6. Exit plan mode. Execute waves as parallel Agent calls following Atlas's
   dispatch.
   </workflow>
```

## License

[MIT](./LICENSE) — Cong Tran
