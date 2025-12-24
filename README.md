# CCC - Claude Code Core Skills

Core skills library for Claude Code: TDD, debugging, collaboration patterns, and proven techniques.

## Installation

### Quick Install

1. Open Claude Code in your terminal
2. Add this repository as a marketplace:

   ```bash
   /plugin marketplace add trancong12102/ccc
   ```

3. Install the plugin:

   ```bash
   /plugin install ccc@ccc-marketplace
   ```

### Verify Installation

```bash
/plugin list
```

You should see `ccc` in the list of installed plugins.

## Environment Variables

Add these to your shell configuration (e.g., `.zshrc` or `.bashrc`):

```bash
# Enable dynamic loading/unloading of MCP servers during active sessions
export ENABLE_EXPERIMENTAL_MCP_CLI=true

# API keys for MCP servers
export CONTEXT7_API_KEY=xxx
export EXA_API_KEY=xxx
```

## MCP Servers

The search agent requires MCP servers for web search, library documentation, and GitHub repository analysis. Add these to your `~/.claude.json`:

```json
{
  "mcpServers": {
    "context7": {
      "type": "http",
      "url": "https://mcp.context7.com/mcp",
      "headers": {
        "CONTEXT7_API_KEY": "${CONTEXT7_API_KEY}"
      }
    },
    "exa": {
      "type": "http",
      "url": "https://mcp.exa.ai/mcp?exaApiKey=${EXA_API_KEY}"
    },
    "deepwiki": {
      "type": "http",
      "url": "https://mcp.deepwiki.com/mcp"
    }
  }
}
```

After adding, restart Claude Code to load the MCP servers.

## Included Components

### Skills

#### test-driven-development

Use when implementing any feature or bugfix, before writing implementation code. Enforces the Red-Green-Refactor cycle: write a failing test first, write minimal code to pass, then refactor.

#### writing-skills

Use when creating new skills, editing existing skills, or verifying skills work before deployment.

This skill applies Test-Driven Development principles to process documentation - write test cases (pressure scenarios), watch them fail (baseline behavior), write the skill, and verify agents comply.

### Agents

#### code-review

Senior code reviewer for security, performance, and best practices. Use proactively after completing significant code changes, before commits, or when reviewing PRs. Analyzes uncommitted changes, branch diffs, or specific commits.

#### search

Research specialist for web searches, documentation lookups, and GitHub repository analysis. Use when asking about libraries, frameworks, APIs, or any technical topics requiring up-to-date information.

### Commands

#### commit

Generate conventional commit messages following the [Conventional Commits 1.0.0](https://www.conventionalcommits.org/) specification. Invoke with `/commit`.

## Usage

Once installed, Claude Code will automatically discover and use the skills when relevant. Skills are loaded based on context matching - when Claude encounters a situation matching a skill's description, it will apply that skill's techniques.

## License

MIT
