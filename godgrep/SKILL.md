---
name: godgrep
description: "Unified local codebase search across semantic, structural, and keyword modes. Use when searching a codebase for code patterns, tracing how features work across files, finding structural code issues, or exploring unfamiliar projects. Do not use for external library docs or GitHub searches."
---

# godgrep

Unified codebase search skill. Routes search tasks to the right tool based on intent.

## Tool Routing

Prefer `fff` MCP tools (`mcp__plugin_ora_fff__grep`, `mcp__plugin_ora_fff__find_files`, `mcp__plugin_ora_fff__multi_grep`) for keyword and file search in git-indexed directories. Frecency-ranked, faster, dirty-file boost. Fall back to `rg`/`fd` only when fff unavailable or target is outside git index.

| Intent                       | Primary tool                      | Also consider                                       |
| ---------------------------- | --------------------------------- | --------------------------------------------------- |
| Architecture / broad explore | `codebase-search`                 | `mcp__plugin_ora_fff__find_files` for dir structure |
| Trace a flow / feature       | `codebase-search` → Read          | LSP for call chains                                 |
| Find all usages of X         | `codebase-search`                 | LSP find-references                                 |
| Explore risks / dependencies | `codebase-search` → Read          | `mcp__plugin_ora_fff__grep` for specific checks     |
| Find a specific symbol       | LSP go-to-definition              | `mcp__plugin_ora_fff__grep`                         |
| Structural code patterns     | `ast-grep`                        | `mcp__plugin_ora_fff__grep` as fallback             |
| Keyword / symbol search      | `mcp__plugin_ora_fff__grep`       | codebase-search if conceptual                       |
| Multi-pattern / OR search    | `mcp__plugin_ora_fff__multi_grep` | sequential `mcp__plugin_ora_fff__grep` calls        |
| File discovery               | `mcp__plugin_ora_fff__find_files` | `mcp__plugin_ora_fff__grep` for content matches     |
| Outside git index            | `rg` / `fd`                       | —                                                   |
| Git history / blame          | Bash (git log/blame)              | —                                                   |

**Decision rule**: Can you write the grep pattern? Use `mcp__plugin_ora_fff__grep`. Need multiple patterns at once? Use `mcp__plugin_ora_fff__multi_grep`. Need a symbol definition or references? Use LSP. Need AST structure? Use ast-grep. Conceptual question? Use codebase-search.

Start broad with `codebase-search`, then drill down with `mcp__plugin_ora_fff__grep`/Read/LSP. Do not start with 20+ grep calls when 1-2 `codebase-search` calls can map the landscape first.

For broad questions, break into 2-3 search angles and launch in parallel. Read files surfaced by search to get full context before answering.

---

## codebase-search

Semantic search powered by MorphLLM -- an RL-trained subagent that runs ~15-30 internal grep+read operations to answer natural language questions about project files (code, config, docs, markdown, YAML, JSON).

Requires `MORPH_API_KEY` environment variable.

### Usage

```bash
python3 scripts/codebase-search.py search "<natural language query>" [repo_path]
```

Key flags:

- `--search-type node_modules` -- include node_modules in search
- `--timeout N` -- timeout in seconds (default 120)
- `--dry-run` -- print command without executing

```bash
# Search current repo
python3 scripts/codebase-search.py search "how does the authentication flow work"

# Search a specific repo
python3 scripts/codebase-search.py search "how are database migrations handled" /path/to/repo

# Include node_modules
python3 scripts/codebase-search.py search "how does the router resolve paths" --search-type node_modules

# Large codebase
python3 scripts/codebase-search.py search "trace the payment pipeline" --timeout 180
```

### Rules

- Write natural language questions, not keywords -- `"How does auth middleware validate tokens?"` not `"auth JWT"`
- Be specific about what you want to know -- the subagent plans its own search strategy from your question
- Default timeout is 120s -- increase with `--timeout` for large codebases
- Do not read the script source code -- run directly and use `--help` for usage

Reference: `references/codebase-search.md` for full documentation.

---

## fff (preferred keyword/file search)

Fast file finder MCP. Frecency-ranked results — frequent/recent files first, git-dirty files boosted. Use as primary tool for keyword and file lookup in git-indexed dirs.

### Tool selection

- **`mcp__plugin_ora_fff__grep`** — DEFAULT. Search file CONTENTS for definitions, usages, patterns. Use when you have specific name or pattern.
- **`mcp__plugin_ora_fff__find_files`** — Explore which files/modules exist for topic. Use when you DO NOT have specific identifier or are LOOKING FOR FILE.
- **`mcp__plugin_ora_fff__multi_grep`** — OR logic across multiple patterns. Use for case variants (e.g. `['PrepareUpload', 'prepare_upload']`) or 2+ different identifiers in one call.

### Search rules

- Search BARE IDENTIFIERS only — grep matches single lines. One identifier per query.
  - Good: `'InProgressQuote'`, `'ActorAuth'`
  - Bad: `'load.*metadata.*InProgressQuote'` (regex spanning tokens), `'struct ActorAuth'` (keyword narrows results)
- Plain text > regex. Patterns like `.*`, `\d+`, `\s+` rarely match within single lines. Use regex only for true alternation.
- Stop after 2 grep calls — READ code instead. More greps != better understanding.
- Use `mcp__plugin_ora_fff__multi_grep` for multiple identifiers in single call:
  - Good: `multi_grep(['ActorAuth', 'PopulatedActorAuth', 'actor_auth'])`
  - Bad: sequential grep calls with variants

### Fallback to rg/fd

Use `rg` / `fd` only when:

- Target outside git index (untracked dirs, system paths)
- fff MCP unavailable in session

---

## ast-grep

Structural code search using Abstract Syntax Tree patterns. Matches code by structure, not text.

### Pattern Search

```bash
ast-grep run --pattern '<pattern>' --lang <lang> <path>
```

```bash
# Find all console.log calls
ast-grep run --pattern 'console.log($ARG)' --lang javascript .

# Find class declarations
ast-grep run --pattern 'class $NAME' --lang python /path/to/project
```

### Complex Rules

**Inline YAML** (quick iterations, no temp files):

```bash
ast-grep scan --inline-rules "<yaml>" <path>
```

```bash
# Find async functions containing await
ast-grep scan --inline-rules "id: async-await
language: javascript
rule:
  kind: function_declaration
  has:
    pattern: await \$EXPR
    stopBy: end" /path/to/project
```

**Rule file** (recommended for complex rules):

```bash
# Write rule to a temp file, then scan
ast-grep scan --rule /tmp/my_rule.yml /path/to/project
```

### AST Inspection

```bash
ast-grep run --pattern '<code>' --lang <lang> --debug-query=cst
```

Use `--debug-query=cst` to dump the concrete syntax tree and find correct `kind` values when rules do not match. Available formats: `cst` (all nodes), `ast` (named nodes only), `pattern` (how ast-grep interprets your pattern).

### Critical Rules

- **ALWAYS use `stopBy: end`** for relational rules (`inside`, `has`) -- without it, search stops at the first non-matching node instead of traversing the full subtree:

```yaml
has:
  pattern: await $EXPR
  stopBy: end # required for deep traversal
```

- **Escape metavariables in shell**: use `\$VAR` in double-quoted strings, or `'$VAR'` in single-quoted strings
- Start simple (pattern first), add `kind` + relational rules only when needed
- Use `all`/`any`/`not` to compose complex structural queries

Reference: `references/ast-grep/ast-grep.md` for full guide, `references/ast-grep/rule_reference.md` for YAML rule syntax.

---

Do not reach for codebase-search or ast-grep when `mcp__plugin_ora_fff__grep`/`mcp__plugin_ora_fff__find_files`/LSP suffice — they are slower and consume more resources. Escalate only when:

- The question is conceptual and you cannot write a grep pattern (use codebase-search)
- The search requires understanding code structure, not just text (use ast-grep)
