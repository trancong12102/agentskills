---
name: godgrep
description: "Unified local codebase search across keyword, structural, and semantic modes. Use when searching a codebase for code patterns, tracing how features work across files, finding structural code issues, answering conceptual questions about a codebase, or exploring unfamiliar projects. Do not use for external library docs or GitHub searches."
---

# godgrep

Routes codebase search tasks to the right tool based on intent.

## Tool Routing

| Intent                       | Primary tool                                         | Also consider                                   |
| ---------------------------- | ---------------------------------------------------- | ----------------------------------------------- |
| Conceptual / semantic Q      | `mcp__plugin_ora_fff__find_files` + grep → Read loop | LSP for call chains                             |
| Architecture / broad explore | `mcp__plugin_ora_fff__find_files` for dir structure  | `mcp__plugin_ora_fff__grep` for keyword anchors |
| Trace a flow / feature       | `mcp__plugin_ora_fff__grep` → Read                   | LSP for call chains                             |
| Find all usages of X         | LSP find-references                                  | `mcp__plugin_ora_fff__grep`                     |
| Find a specific symbol       | LSP go-to-definition                                 | `mcp__plugin_ora_fff__grep`                     |
| Structural code patterns     | `ast-grep`                                           | `mcp__plugin_ora_fff__grep` as fallback         |
| Keyword / symbol search      | `mcp__plugin_ora_fff__grep`                          | LSP for definitions                             |
| Multi-pattern / OR search    | `mcp__plugin_ora_fff__multi_grep`                    | sequential `mcp__plugin_ora_fff__grep` calls    |
| File discovery               | `mcp__plugin_ora_fff__find_files`                    | `mcp__plugin_ora_fff__grep` for content matches |
| Outside git index / fallback | shell tools (`rg`/`grep`/`ugrep`, `fd`/`find`/`bfs`) | last resort, after `fff`                        |
| Git history / blame          | Bash (git log/blame)                                 | —                                               |

For broad questions, break into 2-3 search angles and launch in parallel.

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
