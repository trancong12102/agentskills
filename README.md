# Agent Skills

The **ora** Claude Code plugin: two research agents and five focused skills, written for Claude 5-class models.

## Why ora

Agent frameworks built for weaker models compensate with many agents, hooks, and prescriptive step-by-step instructions. Modern models don't need that. ora ships only what the model can't already know — tool routing, script paths, and empirical gotchas — in two research agents and five small skills:

- **Ariadne** — codebase exploration (semantic, keyword, and structural search across local files)
- **Clio** — external research (docs, repos, package registries)

Both isolate search context from the main conversation — broad queries never pollute your main window.

| Skill           | Purpose                                                                        |
| --------------- | ------------------------------------------------------------------------------ |
| `code-search`   | Routes local search — fff keyword, morph semantic, LSP symbols                 |
| `ast-grep`      | Structural code search by AST pattern                                          |
| `lib-docs`      | Library documentation via llms.txt, context7 fallback                          |
| `repo-research` | External repos — morph GitHub search, Sourcegraph, `gh`, cached shallow clones |
| `pkg-versions`  | Latest versions and deprecation status via deps.dev                            |

Skills live inside the plugin, so they update automatically with it.

## Getting Started

### Prerequisites

ora ships three MCP servers:

**[`fff-mcp`](https://github.com/dmtrKovalenko/fff.nvim)** — stdio MCP, fast file finder (frecency-ranked). Install the binary:

```shell
# Install the prebuilt binary to ~/.local/bin/fff-mcp
curl -L https://dmtrkovalenko.dev/install-fff-mcp.sh | bash

# Ensure ~/.local/bin is on PATH (zsh shown — adjust for bash/fish)
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc && source ~/.zshrc

# Verify
which fff-mcp   # should print ~/.local/bin/fff-mcp
```

Statically linked (musl on Linux) — no Node, Rust toolchain, or runtime dependency.

**[`sourcegraph`](https://sourcegraph.com/docs/api/mcp)** — HTTP MCP, cross-repo code search across 2M+ OSS repos. See [Credentials](#credentials) for token setup.

**[`morph`](https://docs.morphllm.com/mcpquickstart)** — stdio MCP via `bunx @morphllm/morphmcp@latest`, semantic codebase search (`codebase_search` for local, `github_codebase_search` for GitHub deps). See [Credentials](#credentials) for API key setup.

### Install

```shell
/plugin marketplace add trancong12102/agentskills
/plugin install ora@agentskills

# Notification plugins (optional)
/plugin install sound-notify@agentskills
/plugin install terminal-notify@agentskills
```

### Credentials

| Component   | Credential          | How to get                                                                                                                                                                          |
| ----------- | ------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `lib-docs`  | `ctx7 login`        | One-time login via `bunx ctx7@latest login` (library docs from [context7.com](https://context7.com))                                                                                |
| `ora` (MCP) | `SOURCEGRAPH_TOKEN` | Generate a PAT at [sourcegraph.com/user/settings/tokens/new](https://sourcegraph.com/user/settings/tokens/new) (scope `mcp`, no expiration), then `export SOURCEGRAPH_TOKEN=sgp_…`. |
| `ora` (MCP) | `MORPH_API_KEY`     | Sign up at [morphllm.com](https://morphllm.com), generate an API key, then `export MORPH_API_KEY=…`.                                                                                |

> Both tokens are read from your shell at startup — if either is unset, Claude Code fails to parse `.mcp.json`. Export both, or remove the corresponding server block. (OAuth via `/mcp` is an alternative for Sourcegraph but expires quickly.)

## License

[MIT](./LICENSE) — Cong Tran
