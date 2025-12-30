# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.4.3] - 2025-12-30

### Changed

- exploring-codebase: restructured following best practices (role, quick reference, tiered examples, fallback table)
- exploring-codebase: improved description with purpose and trigger hints
- searching-web: simplified description

## [1.4.2] - 2025-12-30

### Changed

- exploring-codebase: conditional subagent delegation (≤3 searches → direct calls, >3 → subagent)
- searching-web: complexity-based approach (1-2 calls → direct, ≥3 → subagent)
- Both skills reduced in size and simplified

## [1.4.1] - 2025-12-30

### Changed

- exploring-codebase: use subagent delegation for cleaner context
- exploring-codebase: improved trigger description for better skill activation
- exploring-codebase: added error recovery section

## [1.4.0] - 2025-12-30

### Added

- exploring-codebase skill for semantic code search using WarpGrep MCP

## [1.3.1] - 2025-12-30

### Changed

- searching-web: clarify core principle to explicitly forbid direct tool calls

## [1.3.0] - 2025-12-29

### Added

- searching-web skill for web searches, library documentation, GitHub repos, and URL content extraction

## [1.2.0] - 2025-12-29

### Changed

- Flatten skills directory structure - skills now live at root level instead of nested `skills/` folder

## [1.1.0] - 2025-12-29

### Added

- openspec-proposal skill for scaffolding OpenSpec changes
- openspec-apply skill for implementing approved changes
- openspec-archive skill for archiving deployed changes

### Changed

- Improved skill descriptions with trigger context following best practices

## [1.0.0] - 2025-12-29

### Changed

- Rebrand from Claude Code plugin marketplace to generic agent skills collection
- Rewrite README for agent skills focus

### Removed

- Claude Code plugin system (plugins/, marketplace.json, commands/, agents/)
- MCP server configurations
- Environment variables requirements

### Added

- conventional-commit skill following Conventional Commits 1.0.0 specification
