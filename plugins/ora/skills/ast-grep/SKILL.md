---
name: ast-grep
description: "Structural code search with ast-grep — matches code by AST shape instead of text. Use when a search needs code structure that grep can't express, e.g. async functions containing await, calls with specific argument shapes, or nodes inside particular scopes. Do not use when plain grep or LSP suffices — it is slower."
---

# ast-grep

```bash
ast-grep run --pattern 'console.log($ARG)' --lang javascript .    # simple pattern
ast-grep scan --inline-rules "<yaml>" <path>                      # relational/composite rule, no temp file
ast-grep scan --rule /path/rule.yml <path>                        # complex rule from file
ast-grep run --pattern '<code>' --lang <lang> --debug-query=cst   # dump CST to find correct `kind` values when a rule doesn't match
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
