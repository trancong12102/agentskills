# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.12.1] - 2025-12-26

### Fixed

- Fix bash pattern parsing error in commit command breaking changes section

## [0.12.0] - 2025-12-26

### Changed

- Update code-review agent to sonnet model with structured prompts
- Update commit command to sonnet model with structured prompts
- Add critical rules, scope boundaries, and output format to both agents

## [0.11.0] - 2025-12-26

### Changed

- Rewrite librarian agent following Anthropic Claude 4.5 and multi-agent best practices
- Add critical rules section with mandatory `date` command and `mcp-cli info` checks
- Add scope boundaries (DO/DO NOT) for clear agent responsibilities
- Add success criteria for research completion/failure
- Add context budget management with safe limits table
- Add structured output format for agent-to-agent communication
- Add fallback chain and error templates
- Improve agent description following official Claude Code subagent patterns

## [0.10.0] - 2025-12-25

### Added

- Complexity assessment system for librarian agent (SIMPLE, MODERATE, COMPLEX)
- Adaptive tool call count based on question complexity (1-2, 3-4, 5-8)
- Adaptive search depth parameters (`numResults`, `tokensNum`) per complexity level
- Three output templates scaled to complexity level
- New few-shot examples demonstrating complexity-aware behavior

## [0.9.0] - 2025-12-25

### Changed

- Rename `search` agent to `librarian` for broader research capabilities
- Rewrite librarian agent following Google Gemini 3 prompting best practices
- Add Plan-Execute-Validate-Format workflow for structured research
- Add few-shot examples for IMPLEMENTATION, TROUBLESHOOTING, COMPARISON, CONCEPTUAL queries
- Improve agent description for more proactive invocation triggers

## [0.8.1] - 2025-12-25

### Fixed

- Remove unsupported `category` field from plugin.json and marketplace.json schemas

## [0.8.0] - 2025-12-25

### Changed

- Reorganize plugin structure into categories (`plugins/development`, `plugins/research`, `plugins/git-workflows`)
- Each plugin now has its own `plugin.json` with category metadata
- Version now defined per plugin entry in marketplace.json
- Rename marketplace from `ccc-marketplace` to `ccc`

### Removed

- `test-driven-development` skill
- `writing-skills` skill and `skill-authoring` plugin
- Root-level `agents/`, `commands/`, `skills/` directories (moved to category folders)
- `superpowers` submodule

## [0.7.0] - 2025-12-24

### Removed

- `using-search-agent` skill - agents prefer directly picking subagents

## [0.6.0] - 2025-12-24

### Added

- `using-search-agent` skill for guidance on when to dispatch the search subagent

## [0.5.0] - 2025-12-24

### Added

- `test-driven-development` skill for implementing features and bugfixes using TDD methodology

## [0.4.3] - 2025-12-24

### Changed

- Sync writing-skills skill with superpowers submodule

## [0.4.2] - 2025-12-24

### Changed

- Move MCP server configurations from plugin.json to README for user-level setup
- Remove version field from marketplace.json

## [0.4.0] - 2025-12-24

### Added

- MCP server configurations (context7, exa, deepwiki) for enhanced search and documentation capabilities
- Environment variables documentation in README

## [0.3.0] - 2025-12-24

### Added

- `commit` command for generating conventional commit messages

## [0.2.0] - 2025-12-24

### Added

- `code-review` agent for automated code review
- `search` agent for codebase exploration and search

## [0.1.0] - 2025-12-24

### Added

- Initial plugin release
- `writing-skills` skill for enhanced writing capabilities
