# Ora

Two research agents, one for the local codebase and one for external sources. Each searches in its own context, so the main conversation gets the answer without the pages of search output behind it.

## Agents

| Agent      | Model | Role                                                   |
| ---------- | ----- | ------------------------------------------------------ |
| `explore`  | Opus  | Answers questions about the local codebase. Read-only. |
| `research` | Opus  | Answers from docs, public repos and the web.           |

Both get the outcome and the bar an answer has to clear, not a method. The model
picks its own tools.

## The `run` tool

One MCP tool, `mcp__plugin_ora_ora__run`, runs a bash script in a persistent shell
and returns its combined output. It is available to the main session and to both
agents. Its description lists the toolbox below, so the model knows what it can
call without a skill being loaded, and it is marked always-loaded, so it costs no
tool-search round trip.

- **State.** The working directory, variables and functions persist between calls.
  Each `session` name is its own shell. Calls within one session run in order;
  different sessions run in parallel.
- **Timeouts.** A script still running at `timeout_s` (default 120 s) keeps
  running. A call with an empty script waits for it, and `reset: true` kills it.
- **Scope.** It runs outside Claude Code's Bash sandbox and permission rules, so
  its description scopes it to reading, analysis and fetching. Changes to files
  or git state stay with the Bash tool.
- **Cleanup.** Each session is its own process group. A reset, 30 idle minutes,
  or the server exiting (the client hanging up, or a signal) kills the whole
  group, including anything a script left running in the background. Each
  script's file is deleted once it finishes, and the server's directory under
  `$TMPDIR` goes when the server exits.
- **Large output.** Claude Code saves any result over its limit to a file itself.
- **Line endings.** CRLF lines (HTTP headers, Windows files) come back intact; a
  line redrawn with `\r`, such as a progress bar, comes back in its final state.

The server is `mcp/server.ts`, run by Bun (56 ms to start, 0.4 ms of overhead per
call). Claude Code runs `bun install` against the plugin's `bun.lock` when it
installs the plugin.

## Toolbox

The plugin's `bin/` directory is on the `PATH` of the `run` shell and of every
Bash command Claude Code runs. Each entry runs the tool it is named after: a copy
already on `PATH` if there is one, otherwise [mise](https://mise.jdx.dev) installs
it on first use (a few seconds; `bfs` builds from source, so it needs a C compiler)
and caches it (~50 ms after that). The directory comes last on `PATH`, so any
native copy wins without the link being involved.

- Search and read code: `ast-grep` · `fd` · `bfs` · `rg` · `ugrep` · `uctags` · `scc` · `difft` · `sd`
- Code intelligence (JS/TS, Python, Go; needs installed dependencies): `scip` · `scip-typescript` · `scip-python` · `scip-go` · `depcruise` · `knip`
- Data: `jq` · `yq` · `duckdb` · `syft`
- Web and documents: `tinyfish` · `lightpanda` · `markitdown` · `pdftotext` · `pdfinfo` · `yt-dlp` · `hf`
- Hosts and runtimes: `gh` · `glab` · `uv` / `uvx` · `bun` / `bunx`

`uctags` is Universal Ctags under its own name, because macOS ships a BSD `ctags`
in `/usr/bin`, and that directory comes before this one on `PATH`.

A version manager's shim (mise, asdf) that has no version set for a tool falls
back to the next copy on `PATH`, which is this link. The link skips any candidate
that has already led back to it and installs through mise instead, so the two
cannot bounce off each other.

The agents install anything else with `mise x <tool>@latest -- <cmd>`, or
`mise x github:<owner>/<repo>@latest -- <cmd>` for a GitHub release binary. mise
keeps it cached for later sessions.

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

The `Brewfile` installs native copies of the toolbox, Bun included, which the
`bin/` entries then prefer. It is optional: with only [mise](https://mise.jdx.dev)
on `PATH`, each tool is fetched on first use instead. `tinyfish` is not in
Homebrew; `npm install -g @tiny-fish/cli` gives a native copy, and mise's
fallback needs `node` ≥ 24.

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

All suites run real processes, with no mocks.

- `tests/server.test.ts` drives the MCP server over stdio JSON-RPC with real bash.
  It covers state, timeouts, reset, a script exiting the shell, stdin isolation,
  and parallel versus serial sessions.
- `tests/log-answer.test.ts` pipes `SubagentStop` payloads into the hook script.
  Among other cases, it checks that a report sent through `SubagentHandback` is
  logged instead of the closing line the agent writes after sending it.
- `tests/toolbox.sh` drives the `bin/` entries against real mise and real GitHub
  releases, installing into a throwaway mise data directory.

```bash
cd plugins/ora && bun install && bun test tests/
bash plugins/ora/tests/toolbox.sh
```

## License

[MIT](../../LICENSE). Copyright Cong Tran.
