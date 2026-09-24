# Ora

Two research agents, one for the local codebase and one for external sources. Each searches in its own context, so the main conversation gets the answer without the pages of search output behind it.

## Agents

| Agent      | Model | Role                                                   |
| ---------- | ----- | ------------------------------------------------------ |
| `explore`  | Opus  | Answers questions about the local codebase. Read-only. |
| `research` | Opus  | Answers from docs, public repos and the web.           |

Both get the outcome and the bar an answer has to clear, not a method. The model
picks its own tools.

## CLIs

The agents work through Claude Code's Bash tool, so they use whatever CLIs are on
your `PATH`, under your permission rules. The plugin installs none. These are the
ones the agents' prompts name:

- Code: `ast-grep` · Universal Ctags · `scc` · `difft` · `duckdb` · `syft` · `scip` indexers
- Web and documents: `tinyfish` · `lightpanda` · `markitdown` · `pdftotext` · `yt-dlp` · `hf` · `gh` · `glab`

## Answer log

A `SubagentStop` hook appends each `explore` and `research` answer, with the question
the agent was given, to `~/.claude/ora/answers.jsonl`. The file lives outside the
plugin directory so a plugin update does not erase it. Nothing else is recorded,
nothing leaves the machine, and the hook always exits 0 so it cannot stall a turn.
Set `ORA_ANSWER_LOG=0` to turn it off; the log stops growing past 32 MB.

`scripts/audit-answers.py` reads that log and reports answers whose conclusion
states a version or date that none of its own sources carry. A token the question
itself named is the asker's premise, not a claim, so the audit skips it. An auditor
that can't see the question ends up grading the caller instead of the answer.

```bash
python3 plugins/ora/scripts/audit-answers.py            # + Jev when JEV_API_KEY is set
python3 plugins/ora/scripts/audit-answers.py --no-jev   # deterministic, offline, free
```

With `JEV_API_KEY` set (key from [console.typesafe.ai/keys](https://console.typesafe.ai/keys);
`TYPESAFE_API_KEY` also works), TypeSafe's decision model answers two questions per row in
a single request (~$0.00002, under a second): does the evidence carry every version
and date, and does the conclusion contradict its own evidence. On a labelled
fixture the first question ties the regex at 8/10. The second catches what the regex
cannot see at all, a wrong claim that carries no version or date. Both checks only
report. The regex is the enforcer, and the model is a second pair of eyes.

## Installation

```bash
/plugin marketplace add trancong12102/agentskills
/plugin install ora@agentskills
brew bundle --file ~/.claude/plugins/marketplaces/agentskills/plugins/ora/Brewfile
```

The `Brewfile` covers the CLIs Homebrew carries. The rest install separately:

```bash
npm install -g @tiny-fish/cli @sourcegraph/scip-typescript @sourcegraph/scip-python
uv tool install 'markitdown[pdf,docx,xlsx,pptx]'
```

`lightpanda`, `scip` and `scip-go` ship as GitHub release binaries.

`research` uses [TinyFish](https://tinyfish.ai) for parallel page fetches and
browser agents. It reads the key from `TINYFISH_API_KEY`; get one at
[agent.tinyfish.ai/api-keys](https://agent.tinyfish.ai/api-keys).

`explore` also uses the built-in `Grep` and `Glob`, and on some machines
Claude Code's embedded ripgrep fails its first-use test. Those tools then
return "No files found" for files that exist, with no error. Check with
`claude --debug-file /tmp/cc.log -p hi && grep -i ripgrep /tmp/cc.log`; if it
says `FAILED (mode=embedded)`, install `ripgrep` and set
`"USE_BUILTIN_RIPGREP": "0"` under `env` in `~/.claude/settings.json`.

## Testing

`tests/log-answer.test.ts` pipes real `SubagentStop` payloads into the hook script,
with no mocks. Among other cases, it checks that a report sent through
`SubagentHandback` is logged instead of the closing line the agent writes after
sending it.

```bash
cd plugins/ora && bun test tests/
```

## License

[MIT](../../LICENSE). Copyright Cong Tran.
