# Agent Skills Repo

Repo containing Claude Code skills and subagents. When editing content here, apply Opus 4.7 prompting practices.

## Editing skills and agents

<prompting_style>
Prescriptive framing default ("Before X, do Y" / "Route X to Y"). Prohibitive framing ("Never X") reserved for behavior gates with Anthropic-official phrasing (e.g., "Never speculate about code you have not opened") or technical correctness requirements (e.g., "Never hold MutexGuard across .await").

Explain _why_ per rule. Opus 4.7 generalizes from rationale; bare imperatives underperform on edge cases.

ALL-CAPS (`ALWAYS`/`NEVER`/`DO NOT`) reserved for output-template enforcement or Anthropic-official gate phrasing. Not for tool-trigger pressure.

Keep instruction scope explicit — 4.7 does not auto-generalize. State what the rule applies to and what it does not.
</prompting_style>

<skill_descriptions>
Third-person, canonical pattern: `[capability]. Use when [triggers]. Do not use for [anti-triggers].`

Max 1024 chars. Name + description are the only routing signal at startup.
</skill_descriptions>

<skill_bodies>
SKILL.md body stays under 500 lines. Progressive disclosure via `references/` (load-on-demand) and `scripts/` (execute-without-loading).

Do not duplicate `Use when` / `Do not use for` content in body sections when the description already covers it. 4.7 literal parsing re-triggers on redundant gates.
</skill_bodies>

<plugin_versioning>
When modifying plugin components (agents, hooks, commands, manifest), bump `version` in that plugin's `plugin.json`. Marketplace listing at `.claude-plugin/marketplace.json` references plugins by path, does not carry versions.
</plugin_versioning>

## Repo structure

- `plugins/ora/` — research agents (Ariadne for codebase, Clio for external)
- `plugins/sound-notify/`, `plugins/terminal-notify/` — notification hooks
- `oracle/`, `council-review/`, `godgrep/`, `godfetch/` — workflow skills
- `react-advanced/`, `react-web-advanced/`, `react-native-advanced/` — React ecosystem skills
- `rust-advanced/`, `effect-advanced/`, `typescript-advanced/` — language-specific advanced skills

## Reference

Opus 4.7 prompting best practices: https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/claude-prompting-best-practices
