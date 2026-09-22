# Ora

Research agents and focused skills for codebase exploration and external research — isolate search context from the main conversation.

## Agents

| Agent          | Model  | Role                                            |
| -------------- | ------ | ----------------------------------------------- |
| **`explore`**  | Sonnet | Codebase exploration — enhanced contextual grep |
| **`research`** | Sonnet | External research — docs, repos, registries     |

## MCP tools

The `ora` MCP server ([`mcp/`](mcp)) exposes the operations directly, so they are
called rather than recalled.

| Tool           | Purpose                                                              |
| -------------- | -------------------------------------------------------------------- |
| `ast_search`   | Search by AST pattern or inline YAML rule                            |
| `outline`      | Symbols, exports and imports of a file or directory                  |
| `lib_docs`     | Author-published llms.txt, with a page index when the corpus is huge |
| `pkg_versions` | Latest version and deprecation status via deps.dev                   |
| `repo_fetch`   | Read one file from a public repo, or shallow-clone it                |

## Answer log

A `SubagentStop` hook appends every `explore` and `research` answer — the question it was
given and the answer it returned — to `~/.claude/ora/answers.jsonl`, outside the
plugin directory so a plugin update does not erase it. Nothing else is recorded,
nothing leaves the machine, and the hook always exits 0 so it cannot stall a turn.
Set `ORA_ANSWER_LOG=0` to turn it off; the log stops growing past 32 MB.

`scripts/audit-answers.py` reads that log and reports answers whose conclusion
states a version or date that none of its own sources carry. A token the question
itself named is the asker's premise, not a claim, and is excluded — an auditor
blind to the question measures the caller instead of the answer.

```bash
python3 plugins/ora/scripts/audit-answers.py            # + Jev when JEV_API_KEY is set
python3 plugins/ora/scripts/audit-answers.py --no-jev   # deterministic, offline, free
```

With `JEV_API_KEY` set (key from [console.typesafe.ai/keys](https://console.typesafe.ai/keys);
`TYPESAFE_API_KEY` also works), TypeSafe's decision model answers two questions per row in
a single request (~$0.00002, under a second): does the evidence carry every version
and date, and does the conclusion contradict its own evidence. On a labelled
fixture the first question ties the regex 8/10; the second catches what the regex
structurally cannot -- a wrong claim carrying no version or date at all, which no
pattern over version tokens can see. Both report and overrule nothing: the regex is
the enforcer, the model is a second pair of eyes.

## Skills

Only routing decisions stay as skills — they choose between tools rather than
performing an operation.

| Skill           | Purpose                                                        |
| --------------- | -------------------------------------------------------------- |
| `code-search`   | Routes local search — fff keyword, morph semantic, LSP symbols |
| `repo-research` | Routes external repo research — morph, Sourcegraph, `gh`       |

## Installation

```bash
/plugin marketplace add trancong12102/agentskills
/plugin install ora@agentskills
```

Then install the MCP server binary:

```bash
brew install trancong12102/tap/ora-mcp
```

`ast_search` and `outline` shell out to [`ast-grep`](https://ast-grep.github.io)
(≥ 0.45, for `outline`); `repo_fetch` uses `gh` and `git`. The formula declares
`ast-grep` and `gh` as dependencies; a `cargo install --path plugins/ora/mcp`
build expects them on `PATH` already.

`explore` falls back to the built-in `Grep` and `Glob`, and on some machines
Claude Code's embedded ripgrep fails its first-use test — those tools then
return "No files found" for files that exist, with no error. Check with
`claude --debug-file /tmp/cc.log -p hi && grep -i ripgrep /tmp/cc.log`; if it
says `FAILED (mode=embedded)`, install `ripgrep` and set
`"USE_BUILTIN_RIPGREP": "0"` under `env` in `~/.claude/settings.json`.

## Testing

`cargo test` covers the pure logic and stays offline. The end-to-end suite drives
the built binary over stdio JSON-RPC against real ast-grep, real registries and
real docs hosts, so it is `#[ignore]`d by default:

```bash
cargo test --test e2e -- --ignored           # everything
cargo test --test e2e -- --ignored local_    # no network; this half gates CI
```

One case clones from a private GitLab host to prove a token in a clone URL never
reaches a cache path or an error message. It needs `JMANGO_GITLAB_API_PAT` and
skips itself without it.

## Releasing

Versioning is automatic. [release-plz](https://release-plz.dev) reads the
conventional commits under `plugins/ora/mcp/` since the last tag, opens a release
PR with the bumped `Cargo.toml` and a `CHANGELOG.md`, and on merge pushes
`ora-mcp-v<version>`. That tag triggers [dist](https://github.com/axodotdev/cargo-dist),
which builds the four target archives, cuts the GitHub Release, and pushes the
updated formula to `trancong12102/homebrew-tap`.

`fix:` → patch, `feat:` → minor, `feat!:`/`BREAKING CHANGE:` → major — but while
the version is still `0.x`, Cargo's compatibility rules shift each of those down
one level, so `feat:` bumps the patch and a breaking change bumps the minor.
Commits that touch no file under `plugins/ora/mcp/` produce no release.

Regenerate `.github/workflows/release.yml` with `dist init --yes` after changing
`dist-workspace.toml`; it is generated, so do not hand-edit it.

## License

[MIT](../../LICENSE) — Cong Tran
