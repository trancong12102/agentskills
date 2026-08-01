---
name: pkg-versions
description: "Checks latest stable versions and deprecation status of public packages via deps.dev. Use when adding or updating dependencies, verifying whether a package is deprecated, or comparing installed versions against latest. Do not use for private registries or package documentation."
---

# pkg-versions

```bash
python3 scripts/get-versions.py <system> <pkg1> [pkg2] ...
```

Systems: `npm`, `pypi`, `go`, `cargo`, `maven` (`group:artifact`), `nuget`, `rubygems`. Batch multiple packages in one call.

Output: TSV `package  version  published  status` with status `ok` | `deprecated` | `not found` | `error`. Surface deprecations and suggest an alternative when one exists.
