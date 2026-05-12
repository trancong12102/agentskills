---
name: godgrep
description: "Unified local codebase search across keyword, structural, and semantic modes. Use when searching a codebase for code patterns, tracing how features work across files, finding structural code issues, answering conceptual questions about a codebase, or exploring unfamiliar projects. Do not use for external library docs or GitHub searches."
---

# godgrep

Routes codebase search tasks to the right tool based on intent.

## Tool Routing

| Intent                                               | Primary tool                                      | Also consider                                       |
| ---------------------------------------------------- | ------------------------------------------------- | --------------------------------------------------- |
| Default — concept / feature / "how does X work"      | `ccc search`                                      | follow `[summary]`/`[guide]` hints in output        |
| File / directory summary (path known)                | `ccc describe <path>`                             | Read raw source when no summary exists              |
| Concept guide (cross-cutting topic)                  | `ccc guide <slug>` after `ccc search` surfaces it | —                                                   |
| Trace a flow when no single identifier covers it     | `ccc search` to find entry points                 | `mcp__plugin_ora_fff__grep` to follow refs          |
| Architecture / broad explore                         | `ccc describe .` then drill via `ccc search`      | `mcp__plugin_ora_fff__find_files` for dir structure |
| Keyword / symbol search (exact identifier known)     | `mcp__plugin_ora_fff__grep`                       | LSP for definitions                                 |
| Multi-pattern OR — naming variants of one identifier | `mcp__plugin_ora_fff__multi_grep`                 | sequential `mcp__plugin_ora_fff__grep` calls        |
| Multi-pattern OR — enumerating a feature's keywords  | `ccc search` first                                | fall back to `mcp__plugin_ora_fff__multi_grep`      |
| File discovery (by name)                             | `mcp__plugin_ora_fff__find_files`                 | `mcp__plugin_ora_fff__grep` for content matches     |
| Find all usages of X                                 | LSP find-references                               | `mcp__plugin_ora_fff__grep`                         |
| Find a specific symbol                               | LSP go-to-definition                              | `mcp__plugin_ora_fff__grep`                         |
| Structural code patterns                             | `ast-grep`                                        | `mcp__plugin_ora_fff__grep` as fallback             |
| Outside git index / fallback                         | shell `grep` / `find`                             | last resort, after `fff`                            |
| Git history / blame                                  | Bash (git log/blame)                              | —                                                   |

For broad questions, break into 2-3 search angles and launch in parallel.

## ccc — semantic search and synthesised summaries

`ccc` is the default starting point for codebase exploration. Three commands cover the common reads:

- `ccc search <prose query>` — semantic code search ranked by meaning. Use for concepts, features, or any "how does X work" / "where do we handle Y" question. Returns mixed hits: code chunks, file/directory summaries tagged `[summary]`, and curated concept guides tagged `[guide]`.
- `ccc describe <path>` — pre-synthesised summary of one file or directory (condenses public API, contracts, and role). Use when you already know the path; the summary is typically a faster read than the source. `ccc describe .` gives a project-root overview.
- `ccc guide <slug>` — curated guide for a cross-cutting topic (named subsystems, lifecycles, end-to-end data paths). Discovery is search-driven: `[guide]` hits in `ccc search` carry the slug. Do not run `ccc guide` (no args) as a routine first step — let search surface what is relevant.

```bash
ccc search database connection pooling
ccc search --lang python --lang markdown user authentication
ccc search --path 'src/api/*' request validation
ccc search --offset 5 --limit 5 error handling retry logic
ccc describe src/auth/session.py
ccc guide memoization
```

Follow tagged hints in search output: `[summary]` and `[guide]` results carry the exact follow-up command. The synthesised text is usually a faster read than chasing the underlying files.

Do not use `ccc` when:

- You already have a specific identifier — `mcp__plugin_ora_fff__grep` is faster and exhaustive.
- You need every match of a token (ccc returns top-K by score, not all hits).
- You need a file by name — `mcp__plugin_ora_fff__find_files`.

Typical flow: `ccc search` surfaces relevant files → switch to `fff__grep` + Read on those paths for precise follow-up.

### Anti-pattern: shotgun OR-grep for features

If you find yourself writing a long OR-pattern enumerating guesses for one feature, switch to ccc:

```text
# Shotgun-grep — fragile; misses synonyms, drowns in noise
grep -r "FreeGift|ProgressBar|GiftModal|BuyXGetY|percentOff|fixedOff|salepify"
grep -r "appUpdate|checkAppUpdate|versionCheck|setAppUpdateHandler|needUpdate"

# ccc — meaning-ranked, one query
ccc search free gift and discount rules
ccc search force update / version check flow
```

`multi_grep` is for naming variants of **one identifier** (e.g. `['ActorAuth', 'PopulatedActorAuth', 'actor_auth']`), not for guessing a feature's vocabulary.

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

Do not reach for ast-grep when `mcp__plugin_ora_fff__grep`/`mcp__plugin_ora_fff__find_files`/LSP suffice — it is slower and consumes more resources. Escalate to ast-grep only when the search requires understanding code structure, not just text.
