# Agent Skills

A collection of reusable agent skills for AI coding assistants.

## Skills

| Skill | Description |
| ----- | ----------- |
| `oracle` | Invokes a powerful reasoning model for complex analysis, debugging, and code review |
| `conventional-commit` | Generates git commits following Conventional Commits 1.0.0 specification |
| `brainstorming` | Collaboratively explore ideas through guided dialogue before implementation |
| `test-driven-development` | Guides strict TDD using the Red-Green-Refactor cycle |
| `deps-dev` | Look up the latest version of any package using deps.dev API |

## Setup Notes

### Oracle Skill

The oracle skill requires **Codex MCP** to be configured in Claude Code (or other agents). Add the Codex MCP server to your configuration to enable invoking gpt-5.2 (xhigh) for complex reasoning tasks.

See the [Codex MCP documentation](https://github.com/openai/codex) for setup instructions.

## Recommended External Skills

Other agent skills repositories worth exploring:

| Repository | Focus |
| ---------- | ----- |
| [vercel-labs/agent-skills](https://github.com/vercel-labs/agent-skills) | React and Next.js best practices |
| [boristane/agent-skills](https://github.com/boristane/agent-skills) | Logging best practices |
| [timescale/pg-aiguide](https://github.com/timescale/pg-aiguide) | PostgreSQL AI guide |

## License

MIT
