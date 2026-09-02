---
name: code-search
description: "Routes local codebase search to the right tool — keyword, semantic, or symbol. Use when searching a codebase for identifiers, tracing features across files, answering conceptual questions about local code, or exploring unfamiliar projects. Do not use for external docs, GitHub repos, or package lookups."
---

# code-search

Pick the tool by the shape of the question:

- Exact identifier known → `mcp__plugin_ora_fff__grep`. Naming variants of one identifier (e.g. `['ActorAuth', 'actor_auth']`) → `mcp__plugin_ora_fff__multi_grep`.
- File by name → `mcp__plugin_ora_fff__find_files` (frecency-ranked, git-dirty boosted).
- Concept / "how does X work" / "where is Y handled" (no identifier yet) → `mcp__plugin_ora_morph__codebase_search`. One call replaces the grep→Read→grep loop; fall back to fff + Read only when it misses.
- Symbol definition / all references → LSP.
- Match by code structure rather than text → `mcp__plugin_ora_ora__ast_search`.
- Map a file's or directory's symbols/exports/imports before reading → `mcp__plugin_ora_ora__outline`.
- Git history / blame → git via Bash.

Morph returns a synthesized answer with `file:line` citations. The citations are real; the synthesis can misread what the code does — Read the cited locations before relaying load-bearing conclusions.

Don't grep a feature by OR-ing guessed synonyms; the real name gets missed and output drowns in noise. Learn the feature's actual name from README or directory structure, grep that one term, and follow references.
