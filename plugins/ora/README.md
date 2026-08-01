# Ora

Research agents and focused skills for codebase exploration and external research — isolate search context from the main conversation.

## Agents

| Agent       | Model  | Role                                            |
| ----------- | ------ | ----------------------------------------------- |
| **Ariadne** | Sonnet | Codebase exploration — enhanced contextual grep |
| **Clio**    | Sonnet | External research — docs, repos, registries     |

## Skills

| Skill           | Purpose                                                        |
| --------------- | -------------------------------------------------------------- |
| `code-search`   | Routes local search — fff keyword, morph semantic, LSP symbols |
| `ast-grep`      | Structural code search by AST pattern                          |
| `lib-docs`      | Library documentation via llms.txt, context7 fallback          |
| `repo-research` | External repos — morph GitHub search, Sourcegraph, `gh`        |
| `pkg-versions`  | Latest versions and deprecation status via deps.dev            |

## Installation

```bash
/plugin marketplace add trancong12102/agentskills
/plugin install ora@agentskills
```

## License

[MIT](../../LICENSE) — Cong Tran
