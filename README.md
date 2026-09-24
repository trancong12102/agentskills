# Agent Skills

The **ora** Claude Code plugin: two research agents and the toolbox they work in, written for Claude 5-class models.

## Why ora

Agent frameworks built for weaker models compensate with many agents, hooks, and prescriptive step-by-step instructions. Modern models don't need that. ora ships only what the model can't already know: the outcome it is after, the bar an answer has to clear, and the environment it works in.

Each agent searches in its own context and hands back only the answer, so a broad search doesn't fill up your main conversation.

- `explore` answers questions about the local codebase. It is read-only.
- `research` answers questions from docs, public repos, package registries and the web.

Neither agent is taught a method. Both agents and the main session get one MCP tool, `run`. It is a persistent shell with search, code-intelligence, data and web tools on its `PATH`, and a tool that isn't installed yet installs itself on first use. See the [plugin README](plugins/ora/README.md#the-run-tool).

## Getting started

### Prerequisites

[Bun](https://bun.com) to run the MCP server, native tools from the plugin's `Brewfile` (optional, includes Bun), and [mise](https://mise.jdx.dev) on `PATH` to fetch whatever is still missing on first use:

```shell
brew bundle --file ~/.claude/plugins/marketplaces/agentskills/plugins/ora/Brewfile
```

### Install

```shell
/plugin marketplace add trancong12102/agentskills
/plugin install ora@agentskills
```

### Credentials

| Component               | Credential         | How to get                                                                                                                                                                                                |
| ----------------------- | ------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `research`              | `TINYFISH_API_KEY` | Create a key at [agent.tinyfish.ai/api-keys](https://agent.tinyfish.ai/api-keys), then `export TINYFISH_API_KEY=…`. Parallel page fetches, search and browser agents through the `tinyfish` CLI.          |
| answer audit (optional) | `JEV_API_KEY`      | Create a key at [console.typesafe.ai/keys](https://console.typesafe.ai/keys), then `export JEV_API_KEY=…` (`TYPESAFE_API_KEY` also works). Turns on Jev's two extra checks in `scripts/audit-answers.py`. |

Neither is read at startup. Without `TINYFISH_API_KEY`, `research` falls back to the built-in web tools; without `JEV_API_KEY`, the audit runs its offline check alone.

## License

[MIT](./LICENSE). Copyright Cong Tran.
