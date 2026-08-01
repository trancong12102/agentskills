---
name: ast-grep
description: "Structural code tools via ast-grep — search by AST pattern, and outline file/module structure. Use when a search needs code structure that grep can't express (e.g. async functions containing await, calls with specific argument shapes), or when mapping a file's or directory's symbols, exports, and imports before reading it. Do not use for text search when plain grep or LSP suffices."
---

# ast-grep

## Search

```bash
ast-grep run --pattern 'console.log($ARG)' --lang javascript .        # simple pattern
ast-grep scan --inline-rules "<yaml>" <path>                          # relational/composite rule, no temp file
ast-grep scan --rule /path/rule.yml <path>                            # complex rule from file
echo '<code>' | ast-grep scan --inline-rules "<yaml>" --stdin         # test a rule on a snippet before scanning the codebase
ast-grep run --pattern '<code>' --lang <lang> --debug-query=<style>   # cst = target's full tree (find correct `kind`), ast = named nodes only, pattern = how ast-grep parsed your pattern
```

Gotchas:

- Relational rules (`inside`, `has`) stop at the first non-matching node unless you add `stopBy: end` — always add it for deep traversal:

  ```yaml
  has:
    pattern: await $EXPR
    stopBy: end
  ```

- Escape metavariables in shell: `\$VAR` inside double quotes, or single-quote the pattern.
- Start with a plain pattern; add `kind` and relational rules only when the pattern alone over- or under-matches. Compose with `all`/`any`/`not`.

Full YAML rule syntax: `references/rule-reference.md`.

## Outline (ast-grep ≥ 0.44, alpha)

Map structure before reading full files — top-level items (functions, classes, imports, exports) and their members, syntax-only, no indexing.

```bash
ast-grep outline <file>                                        # local items + member digest
ast-grep outline <dir> --items exports                         # public surface of a directory
ast-grep outline <file> --match Parser --type class --view expanded   # zoom into one symbol, with line numbers
ast-grep outline <dir> --items imports --view signatures       # dependency map
```

- `--items structure|exports|imports|all`; `--view names|signatures|digest|expanded` in increasing detail. Defaults: file → `structure`/`digest`, directory → `exports`/`names`.
- `--match <regex>` and `--type class,enum,...` filter top-level items only, never members; `--pub-members` restricts member views to public members.
- `--json[=stream]` for structured output — `stream` emits one object per file, pipes to `jq`.
