# Agent Skills Repo

This repo holds the ora Claude Code plugin: two research agents and the toolbox they work in. Everything here targets Claude 5-class models, so assume the model is already smart and knows tool mechanics.

## Editing agents and the `run` tool

<prompting_style>
Subtraction over addition. Keep only what the model can't know: the outcome, the bar an answer has to clear, environment facts (what is on `PATH`, where caches live), empirical gotchas. No step-by-step workflows, no verification/self-check instructions, no anti-laziness pressure, no ALL-CAPS trigger language ("Use X when…", never "CRITICAL: you MUST use X").

One default + escape hatch per intent, not menus of undifferentiated options.

Opus 5.5 plans and picks its own tools: state the outcome and the bar, never the method. When a
rule has a scope, state it rather than implying it through examples. Naming the excluded case
explicitly cut failures from 11/12 to 5/12 (p = 0.027, Fisher, 12 reps/arm, measured on Sonnet 5).

An agent's `description` is the main session's only signal for when to delegate, and every description sits in its context all the time, so keep it to a few sentences. Say which questions the agent answers, what it hands back, and the case it is not for. Opus 5-class callers already delegate readily, so leave out pressure phrases ("use proactively", "MUST BE USED") and comparisons with the built-in agents. Claude Code prints the tool list next to each agent itself, so the description does not repeat it. The frontmatter is YAML, so a colon followed by a space inside an unquoted description breaks it. Bodies never repeat the description.
</prompting_style>

<plugin_versioning>
When modifying plugin components (agents, hooks, the MCP server, manifest), bump `version` in that plugin's `plugin.json`, once per commit. The marketplace listing at `.claude-plugin/marketplace.json` points at plugins by path and carries no versions.
</plugin_versioning>

## Repo structure

- `plugins/ora/` holds two research agents (`explore` for the codebase, `research` for external sources) and one MCP tool, `run` (`mcp/server.ts`, Bun). `run` is a persistent bash session, and its description is the model's only map of the toolbox, so keep that description under Claude Code's 2,048-character cut. `bin/` links, which Claude Code also puts on the Bash tool's `PATH`, all point at `libexec/toolbox`, which runs an existing copy of the tool or installs it through mise on first use. `Brewfile` lists the native copies the links prefer. `tests/` holds the end-to-end suites for both.

## Reference

- Claude 5 prompting best practices: https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/claude-prompting-best-practices
- Per-model pages. Read the one for the model that will read your text, not just the general page:
  - Opus 5.5 (the calling session **and** both agents, via `model: opus`): https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/prompting-claude-opus-5-5
  - Fable 5.1 (the advisor model): https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/prompting-claude-fable-5-1
- Subagents in Claude Code (how `description` drives delegation): https://code.claude.com/docs/en/sub-agents
- Writing tools for agents (tool descriptions): https://www.anthropic.com/engineering/writing-tools-for-agents
