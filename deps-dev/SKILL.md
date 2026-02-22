---
name: deps-dev
description: Look up the latest version of any package using deps.dev API. Use this skill when checking package versions, updating dependencies, adding new packages to a project, or when the user asks about the current version of a library.
---

# Latest Package Version Lookup

Query the deps.dev API to get the latest stable version of open source packages.

## Supported Ecosystems

| Ecosystem | System ID | Example Package                    |
| --------- | --------- | ---------------------------------- |
| npm       | `npm`     | `express`, `@types/node`           |
| PyPI      | `pypi`    | `requests`, `django`               |
| Go        | `go`      | `github.com/gin-gonic/gin`         |
| Cargo     | `cargo`   | `serde`, `tokio`                   |
| Maven     | `maven`   | `org.springframework:spring-core`  |
| NuGet     | `nuget`   | `Newtonsoft.Json`                  |

## Workflow

Think step-by-step:

1. **Identify the ecosystem** from context:
   - `package.json` or `node_modules` → npm
   - `requirements.txt`, `pyproject.toml`, `setup.py` → pypi
   - `go.mod`, `go.sum` → go
   - `Cargo.toml` → cargo
   - `pom.xml`, `build.gradle` → maven
   - `*.csproj`, `packages.config` → nuget
   - If unclear, ask the user

2. **Run the get-versions script**:

```bash
SCRIPT=scripts/get-versions.py
python3 $SCRIPT <system> <pkg1> [pkg2] ...
```

3. **Optionally chain CLI tools to transform output** if needed
   - If JSON parsing/filtering is needed, use `jq` first
   - Then optionally use tools like `awk`, `sort`, `cut`, `sed`, `uniq`, `column` for formatting or aggregation
   - Latest versions only: package + version
   - Deprecation audit: only packages with `isDeprecated: true`
   - Error triage: only packages with `error`
   - If no transformation is needed, report directly from script output

## Script Usage

**Single package:**

```bash
python3 scripts/get-versions.py npm express
```

**Multiple packages:**

```bash
python3 scripts/get-versions.py npm express lodash @types/node
```

**Different ecosystems:**

```bash
python3 scripts/get-versions.py pypi requests django flask
python3 scripts/get-versions.py go github.com/gin-gonic/gin
python3 scripts/get-versions.py maven org.springframework:spring-core
```

## Optional CLI Transform Examples (Agent-Focused)

Use `jq` after the script output when you need to return only specific fields.

When needed, chain additional CLI tools after `jq` for sorting, tabular formatting, and summary transforms.

**Version summary (package + version):**

```bash
python3 scripts/get-versions.py npm express lodash @types/node \
  | jq -r '.packages[] | select(has("error") | not) | "\(.package)\t\(.version)"'
```

**Single package version only:**

```bash
python3 scripts/get-versions.py npm express lodash \
  | jq -r '.packages[] | select(.package == "express") | .version'
```

**Deprecated packages only:**

```bash
python3 scripts/get-versions.py npm express lodash \
  | jq '.packages[] | select(.isDeprecated == true) | {package, version, isDeprecated}'
```

**Errors only (for troubleshooting):**

```bash
python3 scripts/get-versions.py npm express nonexistent-pkg \
  | jq '.packages[] | select(has("error")) | {package, error}'
```

**Tabular output with `jq | awk` (agent-friendly reporting):**

```bash
python3 scripts/get-versions.py npm express lodash @types/node \
  | jq -r '.packages[] | select(has("error") | not) | "\(.package)\t\(.version)\t\(.publishedAt)"' \
  | awk -F '\t' '{printf "%-20s %-12s %s\n", $1, $2, $3}'
```

**Stable sorted output (`jq | sort`):**

```bash
python3 scripts/get-versions.py npm express lodash @types/node \
  | jq -r '.packages[] | select(has("error") | not) | "\(.package)\t\(.version)"' \
  | sort
```

## Output Format

The script outputs JSON with the following structure:

```json
{
  "system": "npm",
  "packages": [
    {
      "package": "express",
      "version": "5.0.0",
      "publishedAt": "2024-09-10T04:40:34Z",
      "isDeprecated": false
    },
    {
      "package": "lodash",
      "version": "4.17.21",
      "publishedAt": "2021-02-20T15:42:16Z",
      "isDeprecated": false
    }
  ]
}
```

**Error response:**

```json
{
  "system": "npm",
  "packages": [
    {
      "package": "nonexistent-pkg",
      "error": "HTTP 404: Not Found"
    }
  ]
}
```

## Error Handling

- **HTTP 404**: Package not found - check spelling and ecosystem
- **Network error**: deps.dev API may be temporarily unavailable
- **No default version**: Script returns the latest available version with a note

## Rules

- Always use the script instead of manual curl commands
- CLI chaining (`jq`, `awk`, etc.) is optional; use it only when it improves clarity or efficiency
- If chaining is used, prefer `jq` for JSON parsing; use text tools after `jq` for formatting/aggregation
- The script handles URL encoding automatically
- Multiple packages are fetched in parallel for efficiency
- Use `isDeprecated` field to warn users about deprecated packages
