---
name: lib-docs
description: "Fetches library documentation from author-published llms.txt, with context7 as fallback. Use when looking up docs, API references, changelogs, or breaking changes for an external library or framework. Do not use for local code or repository source exploration."
---

# lib-docs

llms.txt is author-published — no curation lag, no enrichment layer that can hallucinate — so prefer it over context7.

1. Probe: `bash scripts/llms-probe.sh <docs-domain>` — outputs TSV `kind \t url \t size` (`index` = llms.txt page list, `full` = llms-full.txt whole corpus). Paths vary per site; probe instead of hardcoding URLs.
2. Fetch: `full` ≤ ~500 KB → WebFetch it directly. Larger or size `?` → WebFetch the index, then fetch only the matching page links, in parallel. Cloudflare's llms-full.txt is ~46 MB — a blind full fetch blows the context window.
3. Probe failed → context7 via the `ctx7` CLI (one-time `bunx ctx7@latest login`):

   ```bash
   bunx ctx7@latest library <name> [query]      # resolve library ID first — IDs aren't guessable
   bunx ctx7@latest docs <libraryId> "<query>"  # specific queries rank far better than broad ones
   ```

   Retry with `--research` only when the default answer is shallow — it spawns sandboxed agents and costs more.

Most Mintlify/GitBook-hosted docs publish llms.txt; Tailwind and most pre-1.0 or community libraries don't — those go straight to context7.
