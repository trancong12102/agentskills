---
name: deps-dev
description: "Use this skill to look up the latest stable version of any open-source package across npm, PyPI, Go, Cargo, Maven, and NuGet. Trigger whenever the user asks 'what's the latest version of X', 'what version should I use', 'is X deprecated', 'how outdated is my package.json/requirements.txt/Cargo.toml', or needs version numbers for adding or updating dependencies. Also trigger when they mention pinning versions, checking if packages are maintained, or comparing current vs installed versions. Do NOT use for private/internal packages or for looking up documentation."
---

# Latest Package Version Lookup

Query the deps.dev API to get the latest stable version of open-source packages. This is faster and more reliable than searching the web or guessing version numbers, and it catches deprecated packages before you install them.

## Supported Ecosystems

| Ecosystem | System ID | Example Package |
| --------- | --------- | --------------- |
| npm | `npm` | `express`, `@types/node` |
| PyPI | `pypi` | `requests`, `django` |
| Go | `go` | `github.com/gin-gonic/gin` |
| Cargo | `cargo` | `serde`, `tokio` |
| Maven | `maven` | `org.springframework:spring-core` |
| NuGet | `nuget` | `Newtonsoft.Json` |

## When to Use

- Adding a new dependency and need the current version
- Updating `package.json`, `requirements.txt`, `Cargo.toml`, etc. to latest
- Checking whether a package has been deprecated
- Comparing versions across multiple packages at once

## When NOT to Use

- Private or internal packages (deps.dev only indexes public registries)
- Looking up documentation or usage examples (use `context7` instead)

## Workflow

**DO NOT read script source code.** Run scripts directly and use `--help` for usage.

1. **Identify the ecosystem** from project files:
   - `package.json` or `node_modules` → npm
   - `requirements.txt`, `pyproject.toml`, `setup.py` → pypi
   - `go.mod`, `go.sum` → go
   - `Cargo.toml` → cargo
   - `pom.xml`, `build.gradle` → maven
   - `*.csproj`, `packages.config` → nuget

2. **Run the script:**

```bash
python3 scripts/get-versions.py <system> <pkg1> [pkg2] ...
```

Run `python3 scripts/get-versions.py --help` if unsure about usage.

## Examples

```bash
python3 scripts/get-versions.py npm express lodash @types/node
python3 scripts/get-versions.py pypi requests django flask
python3 scripts/get-versions.py go github.com/gin-gonic/gin
```

## Output Format

TSV with header. One line per package:

```
package	version	published	status
express	5.0.0	2024-09-10	ok
lodash	4.17.21	2021-02-20	ok
```

Status values: `ok`, `deprecated`, `not found`, `error: <detail>`.

## Rules

- **Use the script instead of manual curl** — it handles URL encoding (especially for scoped npm packages like `@types/node`) and fetches multiple packages in parallel, so it's both easier and faster.
- **Flag deprecated packages** — if the status column says `deprecated`, tell the user and suggest an alternative if you know one.
- **Batch lookups when possible** — the script accepts multiple package names in one call, which is faster than running it once per package.
