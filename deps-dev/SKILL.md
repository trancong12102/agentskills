---
name: deps-dev
description: Look up the latest version of any package using the deps.dev API. This skill should be used proactively whenever adding, installing, or recommending packages — always verify the latest version before writing it to a manifest file. Also use when checking package versions, updating dependencies, or adding new packages to a project.
---

# Latest Package Version Lookup

Query the deps.dev API to get the latest stable version of open source packages.

## Supported Ecosystems

| Ecosystem | System ID | Example Package |
| --------- | --------- | --------------- |
| npm | `npm` | `express`, `@types/node` |
| PyPI | `pypi` | `requests`, `django` |
| Go | `go` | `github.com/gin-gonic/gin` |
| Cargo | `cargo` | `serde`, `tokio` |
| Maven | `maven` | `org.springframework:spring-core` |
| NuGet | `nuget` | `Newtonsoft.Json` |

## Workflow

1. **Identify the ecosystem** from context:
   - `package.json` or `node_modules` → npm
   - `requirements.txt`, `pyproject.toml`, `setup.py` → pypi
   - `go.mod`, `go.sum` → go
   - `Cargo.toml` → cargo
   - `pom.xml`, `build.gradle` → maven
   - `*.csproj`, `packages.config` → nuget

2. **Run the script** — run `scripts/get-versions.py --help` first if unsure about usage:

```bash
python3 scripts/get-versions.py <system> <pkg1> [pkg2] ...
```

## Examples

```bash
python3 scripts/get-versions.py npm express lodash @types/node
python3 scripts/get-versions.py pypi requests django flask
python3 scripts/get-versions.py go github.com/gin-gonic/gin
```

## Output Format

TSV with header. One line per package, ready to read directly:

```
package	version	published	status
express	5.0.0	2024-09-10	ok
lodash	4.17.21	2021-02-20	ok
```

Status values: `ok`, `deprecated`, `not found`, `error: <detail>`.

## Rules

- Always use the script instead of manual curl commands
- Flag packages with `deprecated` status
- The script handles URL encoding and parallel fetching automatically
