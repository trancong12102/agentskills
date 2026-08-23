# Agent Skills

The **ora** Claude Code plugin: two research agents, an MCP server, and two routing skills, written for Claude 5-class models.

## Why ora

Agent frameworks built for weaker models compensate with many agents, hooks, and prescriptive step-by-step instructions. Modern models don't need that. ora ships only what the model can't already know — tool routing and empirical gotchas.

Operations are MCP tools rather than skills, because a tool gets called where a skill gets skipped, and its guidance travels in the response instead of a document read once at load time. What is left as a skill is only the routing between tools.

Two research agents isolate search from the main conversation:

- **Ariadne** — codebase exploration (semantic, keyword, and structural search across local files)
- **Clio** — external research (docs, repos, package registries)

Both isolate search context from the main conversation — broad queries never pollute your main window.

| ora MCP tool   | Purpose                                                              |
| -------------- | -------------------------------------------------------------------- |
| `ast_search`   | Search by AST pattern or inline YAML rule                            |
| `outline`      | Symbols, exports and imports of a file or directory                  |
| `lib_docs`     | Author-published llms.txt, with a page index when the corpus is huge |
| `pkg_versions` | Latest version and deprecation status via deps.dev                   |
| `repo_fetch`   | Read one file from a public repo, or shallow-clone it                |

| Skill           | Purpose                                                        |
| --------------- | -------------------------------------------------------------- |
| `code-search`   | Routes local search — fff keyword, morph semantic, LSP symbols |
| `repo-research` | Routes external repo research — morph, Sourcegraph, `gh`       |

Both live inside the plugin, so they update automatically with it.

## Getting Started

### Prerequisites

ora ships four MCP servers:

**`ora`** — stdio MCP, source in [`plugins/ora/mcp`](plugins/ora/mcp):

```shell
brew install trancong12102/tap/ora-mcp
```

The formula pulls in `ast-grep` and `gh`, which the server shells out to. Building from source instead needs [`ast-grep`](https://ast-grep.github.io) ≥ 0.45 (for `outline`) on `PATH`:

```shell
cargo install --path plugins/ora/mcp
```

**[`fff-mcp`](https://github.com/dmtrKovalenko/fff.nvim)** — stdio MCP, fast file finder (frecency-ranked). Install the binary and ensure it is on `PATH`.

**[`sourcegraph`](https://sourcegraph.com/docs/api/mcp)** — HTTP MCP, cross-repo code search across 2M+ OSS repos. See [Credentials](#credentials) for token setup.

**[`morph`](https://docs.morphllm.com/mcpquickstart)** — stdio MCP via `bunx @morphllm/morphmcp@latest`, semantic codebase search (`codebase_search` for local, `github_codebase_search` for GitHub deps). See [Credentials](#credentials) for API key setup.

### Install

```shell
/plugin marketplace add trancong12102/agentskills
/plugin install ora@agentskills
```

### Credentials

| Component   | Credential          | How to get                                                                                                                                                                          |
| ----------- | ------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `lib_docs`  | `ctx7 login`        | One-time login via `bunx ctx7@latest login` — only needed as a fallback when a docs site publishes no llms.txt ([context7.com](https://context7.com))                               |
| `ora` (MCP) | `SOURCEGRAPH_TOKEN` | Generate a PAT at [sourcegraph.com/user/settings/tokens/new](https://sourcegraph.com/user/settings/tokens/new) (scope `mcp`, no expiration), then `export SOURCEGRAPH_TOKEN=sgp_…`. |
| `ora` (MCP) | `MORPH_API_KEY`     | Sign up at [morphllm.com](https://morphllm.com), generate an API key, then `export MORPH_API_KEY=…`.                                                                                |

> Both tokens are read from your shell at startup — if either is unset, Claude Code fails to parse `.mcp.json`. Export both, or remove the corresponding server block. (OAuth via `/mcp` is an alternative for Sourcegraph but expires quickly.)

## Releasing (maintainers)

Versions are derived from conventional commits — see [plugins/ora/README.md](plugins/ora/README.md#releasing). Two repository secrets drive it:

| Secret               | Used by              | Why a PAT is required                                                                                                                                         |
| -------------------- | -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `RELEASE_PLZ_TOKEN`  | `release-plz.yml`    | A tag pushed with the default `GITHUB_TOKEN` does not trigger other workflows, so `release.yml` would never fire. Needs Contents + Pull requests: read/write. |
| `HOMEBREW_TAP_TOKEN` | `release.yml` (dist) | `GITHUB_TOKEN` is scoped to this repo only and cannot push to `trancong12102/homebrew-tap`. Needs Contents: read/write on the tap.                            |

One fine-grained PAT scoped to both `agentskills` and `homebrew-tap` can back both secrets.

## License

[MIT](./LICENSE) — Cong Tran
