# Agent Skills

Reusable skills and agents for AI coding agents, primarily Claude Code.

## Why ora

Most Claude Code agent frameworks (11–24 agents, 9+ hooks) add complexity to compensate for weaker models. With Opus 4.6, that complexity burns tokens without improving output. ora ships exactly two research agents:

- **Ariadne** — codebase exploration (enhanced contextual grep over local files)
- **Clio** — external research (docs, web, GitHub repos)

Both isolate search context from the main conversation — broad queries never pollute your main window. The plugin has no hooks and no planning/verification/execution agents. Planning and verification happen inline in the main agent, shaped by behavioral rules in your `CLAUDE.md` (see Configuration below).

## Getting Started

### Install

```shell
# Plugin (agents)
/plugin marketplace add trancong12102/agentskills
/plugin install ora@agentskills

# Skills (optional, standalone)
bunx skills add trancong12102/agentskills -g -y -a claude-code

# Other plugins
/plugin install sound-notify@agentskills
```

### Credentials

| Skill      | Credential                 | How to get                                                                      |
| ---------- | -------------------------- | ------------------------------------------------------------------------------- |
| `godgrep`  | `MORPH_API_KEY` env var    | Sign up at [morphllm.com](https://morphllm.com) (codebase-search)               |
| `godfetch` | `CONTEXT7_API_KEY` env var | Sign up at [context7.com](https://context7.com) (library docs)                  |
| `godfetch` | `MORPH_API_KEY` env var    | Sign up at [morphllm.com](https://morphllm.com) (GitHub code search)            |
| `oracle`   | Codex CLI auth             | Run `codex login` after installing [Codex CLI](https://github.com/openai/codex) |

<details>
<summary>Codex CLI setup for oracle / council-review</summary>

Add to `~/.codex/config.toml`:

```toml
[profiles.oracle]
model = "gpt-5.4"
model_reasoning_effort = "xhigh"
approval_policy = "never"
sandbox_mode = "read-only"
```

</details>

## ora Plugin

| Agent         | Model  | Role                                                                           |
| ------------- | ------ | ------------------------------------------------------------------------------ |
| `ora:Ariadne` | Sonnet | Codebase exploration — traces flows, finds implementations, maps architecture. |
| `ora:Clio`    | Sonnet | External research — fetches docs, searches GitHub repos, checks versions.      |

## Configuration

ora is just two agents — workflow behavior lives in your `~/.claude/CLAUDE.md`. Recommended setup:

```markdown
<investigate_before_responding>
Do not respond to technical tasks without loading a matching skill or spawning ora:Clio research first. The cost is near zero — the cost of outdated knowledge is a broken implementation. Skip only when the task is clearly unrelated to any available skill.
</investigate_before_responding>

<subagent_routing>
Do not use the built-in Explore agent or general-purpose agent — use ora:Ariadne (local codebase) and ora:Clio (external sources) instead. General-purpose agent is the escape hatch when no ora agent fits.

**Resume, don't respawn.** When you need follow-up information from an agent you already spawned in this session, use SendMessage with the returned agentId instead of starting a fresh Agent call. A fresh spawn loses cached context and repeats tool calls. Only start a new agent when the topic is genuinely different from the prior run.
</subagent_routing>

<plan_before_implementing>
Do not implement without entering plan mode when the task is ambiguous, touches multiple subsystems, or has unclear acceptance criteria. Skip plan mode for clearly-scoped changes — bug fixes with obvious fix site, mechanical renames/refactors, config/typo fixes, adding a single feature with known shape.
</plan_before_implementing>

<think_before_coding>
Do not start implementing without first stating your assumptions and flagging tradeoffs. If the request has multiple reasonable interpretations, present them instead of picking silently. If a simpler approach exists than what was asked, say so and push back. If something is unclear enough to block correct execution, stop and ask — do not guess.
</think_before_coding>

<goal_driven_execution>
Do not accept vague goals. Translate each task into a verifiable success criterion before implementing ("add validation" → "write tests for invalid inputs, then make them pass"). Do not mark a task complete until the success criterion is met — read the actual code and run the actual check, do not trust your own summary.
</goal_driven_execution>
```

`think_before_coding` and `goal_driven_execution` are adapted from [forrestchang/andrej-karpathy-skills](https://github.com/forrestchang/andrej-karpathy-skills).

## License

[MIT](./LICENSE) — Cong Tran
