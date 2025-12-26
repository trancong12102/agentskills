# CCC - Claude Code Core Plugins Marketplace

**3 focused plugins** organized by category for minimal token usage.

## Quick Start

```bash
# Add marketplace
/plugin marketplace add trancong12102/ccc

# Install individual plugins by category
/plugin install development@ccc
/plugin install research@ccc
/plugin install git-workflows@ccc
```

## Plugin Categories

| Category | Plugin | Description |
|----------|--------|-------------|
| **Development** (1) | `development` | Code review agent |
| **Research** (1) | `research` | Web search and documentation lookup |
| **Workflows** (1) | `git-workflows` | Conventional commit command |

## Plugins

### Development

Code review and development best practices.

| Type | Name | Description |
|------|------|-------------|
| Agent | `code-review` | Senior code reviewer for security, performance, and best practices |

### Research

Web search, documentation lookup, and repository analysis.

| Type | Name | Description |
|------|------|-------------|
| Agent | `librarian` | Research specialist for external libraries, frameworks, APIs using MCP tools |

### Git Workflows

Git operations and version control workflows.

| Type | Name | Description |
|------|------|-------------|
| Command | `/commit` | Generate conventional commit messages |

## Environment Variables

Required for the `research` plugin:

```bash
export ENABLE_EXPERIMENTAL_MCP_CLI=true
export CONTEXT7_API_KEY=xxx
export EXA_API_KEY=xxx
```

## MCP Servers

Add to `~/.claude.json` for the research agent:

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

## License

MIT
