---
name: godgrep
description: "Unified local codebase search across keyword, structural, and semantic modes. Use when searching a codebase for code patterns, tracing how features work across files, finding structural code issues, answering conceptual questions about a codebase, or exploring unfamiliar projects. Do not use for external library docs or GitHub searches."
---

# godgrep

Routes codebase search tasks to the right tool based on intent.

## Tool Routing

| Intent                                               | Primary tool                                        | Also consider                                       |
| ---------------------------------------------------- | --------------------------------------------------- | --------------------------------------------------- |
| Semantic / "how does X work" — exact keyword unknown | `mcp__plugin_ora_ccc__search`                       | `mcp__plugin_ora_fff__grep` once a keyword surfaces |
| Architecture / broad explore                         | `mcp__plugin_ora_fff__find_files` for dir structure | `mcp__plugin_ora_ccc__search` for entry points      |
| Trace a flow / feature                               | `mcp__plugin_ora_fff__grep` → Read                  | LSP for call chains                                 |
| Find all usages of X                                 | LSP find-references                                 | `mcp__plugin_ora_fff__grep`                         |
| Find a specific symbol                               | LSP go-to-definition                                | `mcp__plugin_ora_fff__grep`                         |
| Structural code patterns                             | `ast-grep`                                          | `mcp__plugin_ora_fff__grep` as fallback             |
| Keyword / symbol search                              | `mcp__plugin_ora_fff__grep`                         | LSP for definitions                                 |
| Multi-pattern / OR search                            | `mcp__plugin_ora_fff__multi_grep`                   | sequential `mcp__plugin_ora_fff__grep` calls        |
| File discovery                                       | `mcp__plugin_ora_fff__find_files`                   | `mcp__plugin_ora_fff__grep` for content matches     |
| Outside git index / fallback                         | shell `grep` / `find`                               | last resort, after `fff`                            |
| Git history / blame                                  | Bash (git log/blame)                                | —                                                   |

For broad questions, break into 2-3 search angles and launch in parallel.

## ccc — cocoindex-code (semantic search)

Vector-based code search — finds chunks by meaning, not text match. Returns top-K hits ranked by relevance score with file path + line range.

```python
mcp__plugin_ora_ccc__search(query="natural language or code snippet", limit=5)
```

Optional filters: `paths` (glob), `languages` (e.g. `["python", "typescript"]`), `offset` (pagination).

Use when:

- Question is conceptual ("how does auth work", "where do we handle retries") and you do not know the identifier to grep for.
- Exploring an unfamiliar codebase and want entry points, not exhaustive enumeration.
- Looking for code similar to a snippet (paste snippet as query).

Do not use when:

- You already have a specific keyword/identifier — `mcp__plugin_ora_fff__grep` is faster and exhaustive.
- You need every match (ccc returns top-K by score, not all hits).
- You need a file by name — `mcp__plugin_ora_fff__find_files`.

Typical flow: ccc surfaces relevant files → switch to fff grep + Read on those paths for precise follow-up.

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
