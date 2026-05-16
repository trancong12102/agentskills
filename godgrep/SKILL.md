---
name: godgrep
description: "Unified local codebase search across keyword, structural, and semantic modes. Use when searching a codebase for code patterns, tracing how features work across files, finding structural code issues, answering conceptual questions about a codebase, or exploring unfamiliar projects. Do not use for external library docs or GitHub searches."
---

# godgrep

Routes codebase search tasks to the right tool based on intent.

## Tool Routing

| Intent                                                                 | Primary tool                                                 | Also consider                                                       |
| ---------------------------------------------------------------------- | ------------------------------------------------------------ | ------------------------------------------------------------------- |
| Keyword / symbol search (exact identifier known)                       | `mcp__plugin_ora_fff__grep`                                  | LSP for definitions                                                 |
| Multi-pattern OR — naming variants of one identifier                   | `mcp__plugin_ora_fff__multi_grep`                            | sequential `mcp__plugin_ora_fff__grep` calls                        |
| File discovery (by name)                                               | `mcp__plugin_ora_fff__find_files`                            | `mcp__plugin_ora_fff__grep` for content matches                     |
| Concept / "how does X work" / "where is Y handled" (no identifier yet) | `mcp__plugin_ora_morph__codebase_search`                     | fall back to `find_files` + `fff__grep` + Read only if morph misses |
| Trace a feature end-to-end                                             | `mcp__plugin_ora_morph__codebase_search` for the initial map | `mcp__plugin_ora_fff__grep` to verify specific call sites           |
| Find all usages of X                                                   | LSP find-references                                          | `mcp__plugin_ora_fff__grep`                                         |
| Find a specific symbol                                                 | LSP go-to-definition                                         | `mcp__plugin_ora_fff__grep`                                         |
| Structural code patterns                                               | `ast-grep`                                                   | `mcp__plugin_ora_fff__grep` as fallback                             |
| Outside git index / fallback                                           | shell `grep` / `find`                                        | last resort, after `fff`                                            |
| Git history / blame                                                    | Bash (git log/blame)                                         | —                                                                   |

When the question is a single broad concept, one `mcp__plugin_ora_morph__codebase_search` call replaces multi-angle parallelization. Split into 2-3 parallel angles only when the work is multiple independent identifier-based searches (e.g. "find callers of X and definitions of Y in one pass").

## Anti-pattern: shotgun OR-grep for features

Do not enumerate guesses for one feature with a long OR-pattern — synonyms get missed and the output drowns in noise:

```text
# Shotgun-grep — fragile, no signal
grep -r "FreeGift|ProgressBar|GiftModal|BuyXGetY|percentOff|fixedOff|salepify"
grep -r "appUpdate|checkAppUpdate|versionCheck|setAppUpdateHandler|needUpdate"
```

Instead: skim README/dir structure to learn the feature's actual name, pick **one specific term**, grep for it, Read the top hit, and follow references. `multi_grep` is for naming variants of **one identifier** (e.g. `['ActorAuth', 'PopulatedActorAuth', 'actor_auth']`), not for guessing a feature's vocabulary.

## Morph `codebase_search` — default for semantic/concept queries

`mcp__plugin_ora_morph__codebase_search` runs a Morph WarpGrep subagent that parallel-greps and reads files, returning a synthesized answer. **Reach for it first** when the question is shaped like "how does X work", "where is Y handled", "what wires Z together", "find all the places that do W". One morph call beats 4-8 `fff__grep` + Read turns on these shapes.

- **Use when** the question is conceptual / semantic / "find X-related code" and you don't have a single exact identifier to grep. Why: morph parallelizes the grep + read + summarize loop the calling agent would otherwise do serially; cuts latency and turn count substantially.
- **Do not use when** you have a concrete identifier — `fff__grep` is faster, exhaustive, and returns exact `file:line` hits in one call.
- **Trust citations, verify conclusions** — morph's `file:line` references point to real code locations, but its synthesis (what that code _does_, how it fits the question) can be wrong. When the answer is load-bearing, `Read` the cited locations and confirm morph's interpretation before relaying.

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
