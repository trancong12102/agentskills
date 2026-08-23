use std::fmt::Write as _;

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use serde_json::Value;

use super::{failed, invalid, text};

str_enum! {
    /// The registries deps.dev indexes.
    System {
        Npm => "npm",
        Pypi => "pypi",
        Go => "go",
        Cargo => "cargo",
        Maven => "maven",
        Nuget => "nuget",
        RubyGems => "rubygems",
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub(crate) struct PkgVersionsArgs {
    /// Registry the packages come from. One call covers one registry.
    pub(crate) system: System,
    /// Package names. Maven uses `group:artifact`, Go uses the full module path.
    /// Batch every package you care about into one call.
    pub(crate) packages: Vec<String>,
}

struct Row {
    package: String,
    version: String,
    published: String,
    status: String,
}

fn parse(package: &str, body: &Value) -> Row {
    let versions = body.get("versions").and_then(Value::as_array);
    let picked = versions.and_then(|vs| {
        vs.iter()
            .find(|v| v.get("isDefault").and_then(Value::as_bool).unwrap_or(false))
            .or_else(|| vs.last())
    });

    let Some(v) = picked else {
        return Row {
            package: package.to_string(),
            version: "-".into(),
            published: "-".into(),
            status: "not found".into(),
        };
    };

    let deprecated = v
        .get("isDeprecated")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    Row {
        package: package.to_string(),
        version: v
            .pointer("/versionKey/version")
            .and_then(Value::as_str)
            .unwrap_or("-")
            .to_string(),
        // `publishedAt` is RFC 3339; the date prefix is all that is useful here.
        published: v
            .get("publishedAt")
            .and_then(Value::as_str)
            .unwrap_or("")
            .chars()
            .take(10)
            .collect(),
        status: if deprecated { "deprecated" } else { "ok" }.into(),
    }
}

async fn fetch(client: &reqwest::Client, system: &str, package: &str) -> Row {
    let url = format!(
        "https://api.deps.dev/v3/systems/{system}/packages/{}",
        urlencode(package)
    );
    let miss = |status: String| Row {
        package: package.to_string(),
        version: "-".into(),
        published: "-".into(),
        status,
    };

    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => return miss(format!("error: {e}")),
    };
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return miss("not found".into());
    }
    if !resp.status().is_success() {
        return miss(format!("error: HTTP {}", resp.status().as_u16()));
    }
    match resp.json::<Value>().await {
        Ok(body) => parse(package, &body),
        Err(e) => miss(format!("error: {e}")),
    }
}

/// deps.dev wants the package name percent-encoded with nothing left safe,
/// which matters for scoped npm names (`@types/node`) and Go module paths.
fn urlencode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

pub(crate) async fn run(
    client: &reqwest::Client,
    args: PkgVersionsArgs,
) -> Result<CallToolResult, McpError> {
    if args.packages.is_empty() {
        return Err(invalid(
            "`packages` must not be empty, e.g. {\"system\": \"npm\", \"packages\": [\"react\", \"zod\"]}",
        ));
    }

    // deps.dev matches the system segment case-sensitively, in upper case.
    let upper = args.system.as_str().to_uppercase();
    let mut set = tokio::task::JoinSet::new();
    for (idx, pkg) in args.packages.iter().enumerate() {
        let (client, system, pkg) = (client.clone(), upper.clone(), pkg.clone());
        set.spawn(async move { (idx, fetch(&client, &system, &pkg).await) });
    }
    let mut indexed: Vec<(usize, Row)> = Vec::with_capacity(args.packages.len());
    while let Some(joined) = set.join_next().await {
        indexed.push(joined.map_err(|e| failed(format!("fetch task panicked: {e}")))?);
    }
    indexed.sort_by_key(|(idx, _)| *idx);
    let rows: Vec<Row> = indexed.into_iter().map(|(_, row)| row).collect();

    let mut out = String::from("package\tversion\tpublished\tstatus\n");
    for r in &rows {
        let _ = writeln!(
            out,
            "{}\t{}\t{}\t{}",
            r.package, r.version, r.published, r.status
        );
    }

    let deprecated: Vec<&str> = rows
        .iter()
        .filter(|r| r.status == "deprecated")
        .map(|r| r.package.as_str())
        .collect();
    if !deprecated.is_empty() {
        let _ = write!(
            out,
            "\nDeprecated: {}. Surface this to the user and name a maintained alternative.",
            deprecated.join(", ")
        );
    }

    Ok(text(out))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use serde_json::json;

    use super::{System, parse, urlencode};

    #[test]
    fn every_system_round_trips_through_its_wire_name() {
        for v in System::ALL {
            assert_eq!(
                serde_json::from_value::<System>(json!(v.as_str())).unwrap(),
                *v
            );
        }
    }

    #[test]
    fn scoped_and_pathed_names_are_fully_encoded() {
        // deps.dev 404s unless `/` and `@` are escaped inside the path segment.
        assert_eq!(urlencode("@types/node"), "%40types%2Fnode");
        assert_eq!(
            urlencode("github.com/gin-gonic/gin"),
            "github.com%2Fgin-gonic%2Fgin"
        );
        assert_eq!(urlencode("serde"), "serde");
    }

    #[test]
    fn default_version_wins_over_later_entries() {
        let body = json!({"versions": [
            {"versionKey": {"version": "1.0.0"}, "isDefault": true, "publishedAt": "2024-01-02T03:04:05Z"},
            {"versionKey": {"version": "2.0.0-beta"}, "isDefault": false}
        ]});
        let row = parse("x", &body);
        assert_eq!(row.version, "1.0.0");
        assert_eq!(row.published, "2024-01-02");
        assert_eq!(row.status, "ok");
    }

    #[test]
    fn last_entry_is_the_fallback_when_nothing_is_default() {
        let body = json!({"versions": [
            {"versionKey": {"version": "1.0.0"}},
            {"versionKey": {"version": "1.1.0"}}
        ]});
        assert_eq!(parse("x", &body).version, "1.1.0");
    }

    #[test]
    fn deprecation_is_reported() {
        let body = json!({"versions": [
            {"versionKey": {"version": "2.88.2"}, "isDefault": true, "isDeprecated": true}
        ]});
        assert_eq!(parse("request", &body).status, "deprecated");
    }

    #[test]
    fn empty_or_absent_versions_read_as_not_found() {
        assert_eq!(parse("x", &json!({"versions": []})).status, "not found");
        assert_eq!(parse("x", &json!({})).status, "not found");
    }
}
