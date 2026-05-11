---
name: godgrep
description: "Unified local codebase search across keyword, structural, and semantic modes. Use when searching a codebase for code patterns, tracing how features work across files, finding structural code issues, answering conceptual questions about a codebase, or exploring unfamiliar projects. Do not use for external library docs or GitHub searches."
---

# godgrep

Routes codebase search tasks to the right tool based on intent.

## Tool Routing

| Intent                                               | Primary tool                                        | Also consider                                |
| ---------------------------------------------------- | --------------------------------------------------- | -------------------------------------------- |
| Keyword / symbol search (exact identifier known)     | `mcp__plugin_ora_fff__grep`                         | LSP for definitions                          |
| Multi-pattern OR — naming variants of one identifier | `mcp__plugin_ora_fff__multi_grep`                   | sequential `mcp__plugin_ora_fff__grep` calls |
| File discovery                                       | `mcp__plugin_ora_fff__find_files`                   | `mcp__plugin_ora_fff__grep` for content      |
| Find all usages of X                                 | LSP find-references                                 | `mcp__plugin_ora_fff__grep`                  |
| Find a specific symbol                               | LSP go-to-definition                                | `mcp__plugin_ora_fff__grep`                  |
| Structural code patterns                             | `ast-grep`                                          | `mcp__plugin_ora_fff__grep` as fallback      |
| Concept / "how does X work" / unfamiliar codebase    | `mcp__plugin_ora_fff__find_files` for dir structure | grep a likely term, Read top hits, follow up |
| Architecture / broad explore                         | `mcp__plugin_ora_fff__find_files` for dir structure | Read README/docs, then grep entry points     |
| Outside git index / fallback                         | shell `grep` / `find`                               | last resort, after `fff`                     |
| Git history / blame                                  | Bash (git log/blame)                                | —                                            |

For broad questions, break into 2-3 search angles and launch in parallel.

## Concept questions without a known identifier

When the question is conceptual ("how does auth work", "where do we handle retries") and no single keyword covers it:

1. Look at directory structure with `find_files` or a quick `ls` of likely roots (`src/auth`, `apps/*/services`).
2. Skim a top-level README or index file to learn the vocabulary used in this repo.
3. Grep one likely term — read the top hit fully, not just the snippet — and let the imports/calls in that file point you to the next term.
4. Repeat. Two reads beat ten greps.

The point is to converge on the real vocabulary of the repo before grepping. Guessed multi-keyword OR-patterns (`grep "FreeGift|ProgressBar|GiftModal|percentOff|salepify"`) are fragile — they miss synonyms and drown in noise. `multi_grep` is for naming variants of **one identifier** (e.g. `['ActorAuth', 'PopulatedActorAuth', 'actor_auth']`), not for guessing a feature's likely names.

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
