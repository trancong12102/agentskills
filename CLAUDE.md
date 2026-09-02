# Agent Skills Repo

Repo containing the ora Claude Code plugin (research agents + skills). Content targets Claude 5-class models — assume the model is already smart and knows tool mechanics.

## Editing skills and agents

<prompting_style>
Subtraction over addition: keep only what the model can't know — MCP tool names, script paths, empirical gotchas. No step-by-step workflows, no verification/self-check instructions, no anti-laziness pressure, no ALL-CAPS trigger language ("Use X when…", never "CRITICAL: you MUST use X").

One default + escape hatch per intent, not menus of undifferentiated options.

Skill descriptions: third person, `[capability]. Use when [triggers]. Do not use for [anti-triggers].` Max 1024 chars — name + description are the only routing signal. Bodies never repeat the description's use-when.

SKILL.md stays small (target ≤ ~60 lines). Reference material goes to `references/` (load-on-demand, one level deep from SKILL.md), fragile operations to `scripts/` (execute-without-reading).
</prompting_style>

<plugin_versioning>
When modifying plugin components (agents, skills, hooks, manifest), bump `version` in that plugin's `plugin.json` — once per commit. Marketplace listing at `.claude-plugin/marketplace.json` references plugins by path and carries no versions.
</plugin_versioning>

## Repo structure

- `plugins/ora/` — research agents (Ariadne: codebase, Clio: external), the `ora-mcp` server (`plugins/ora/mcp/`, Rust), and two routing skills (`code-search`, `repo-research`). Operations live in MCP tools; skills only choose between tools, and guidance the model needs per call belongs in the tool's response, not in a skill body.

## Reference

- Claude 5 prompting best practices: https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/claude-prompting-best-practices (plus per-model pages for Opus 5, Sonnet 5, Fable 5)
- Skill authoring best practices: https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices
