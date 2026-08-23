use std::fmt::Write as _;
use std::process::Stdio;

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use serde_json::Value;
use tokio::process::Command;

use super::{failed, invalid, text};

const DEFAULT_MAX: usize = 60;

/// Full YAML rule syntax. Returned only when a supplied rule fails to compile —
/// that is the one moment the whole reference is worth its size.
const RULE_REFERENCE: &str = include_str!("rule-reference.md");

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub(crate) struct AstSearchArgs {
    /// Directory or file to search.
    pub(crate) path: String,
    /// ast-grep pattern, e.g. `console.log($ARG)`. Supply this or `rule`, not both.
    /// Start here; reach for `rule` only when a bare pattern over- or under-matches.
    #[serde(default)]
    pub(crate) pattern: Option<String>,
    /// Inline ast-grep YAML rule, for relational (`inside`, `has`) or composite
    /// (`all`, `any`, `not`) matching that a bare pattern cannot express.
    #[serde(default)]
    pub(crate) rule: Option<String>,
    /// Language of the pattern, e.g. `typescript`, `rust`, `python`. Required with `pattern`.
    #[serde(default)]
    pub(crate) lang: Option<String>,
    /// Cap on reported matches (default 60).
    #[serde(default)]
    pub(crate) max_results: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub(crate) struct OutlineArgs {
    /// File or directory to outline.
    pub(crate) path: String,
    /// `structure` | `exports` | `imports` | `all`. Defaults: file → structure, directory → exports.
    #[serde(default)]
    pub(crate) items: Option<String>,
    /// `names` | `signatures` | `digest` | `expanded`, in increasing detail.
    #[serde(default)]
    pub(crate) view: Option<String>,
    /// Regex filter over top-level item names. Never filters members.
    #[serde(default)]
    pub(crate) r#match: Option<String>,
    /// Comma-separated top-level kinds to keep, e.g. `class,enum`.
    #[serde(default)]
    pub(crate) kind: Option<String>,
    /// Restrict member views to public members.
    #[serde(default)]
    pub(crate) pub_members: Option<bool>,
}

async fn ast_grep(args: &[String]) -> Result<std::process::Output, McpError> {
    Command::new("ast-grep")
        .args(args)
        .stdin(Stdio::null())
        .output()
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                failed("`ast-grep` not found on PATH; install it (`brew install ast-grep`)")
            } else {
                failed(format!("failed to run ast-grep: {e}"))
            }
        })
}

/// Relational rules stop at the first non-matching node unless `stopBy: end` is
/// set, which is the single most common reason a hand-written rule returns nothing.
fn missing_stop_by(rule: &str) -> bool {
    (rule.contains("inside:") || rule.contains("has:")) && !rule.contains("stopBy")
}

fn render_matches(raw: &str, max: usize) -> Result<(String, usize), McpError> {
    let parsed: Vec<Value> =
        serde_json::from_str(raw).map_err(|e| failed(format!("unparsable ast-grep JSON: {e}")))?;
    let total = parsed.len();

    let mut out = String::new();
    for m in parsed.iter().take(max) {
        let file = m.get("file").and_then(Value::as_str).unwrap_or("?");
        // ast-grep reports zero-based line/column; editors and humans are one-based.
        let line = m
            .pointer("/range/start/line")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            + 1;
        let col = m
            .pointer("/range/start/column")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            + 1;
        let snippet = m
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or("")
            .lines()
            .next()
            .unwrap_or("")
            .trim();
        let _ = write!(out, "{file}:{line}:{col}\t{snippet}");

        if let Some(singles) = m
            .pointer("/metaVariables/single")
            .and_then(Value::as_object)
        {
            let bindings: Vec<String> = singles
                .iter()
                .filter_map(|(name, v)| {
                    let t = v.get("text").and_then(Value::as_str)?.trim();
                    (!t.is_empty()).then(|| format!("${name}={t}"))
                })
                .collect();
            if !bindings.is_empty() {
                let _ = write!(out, "\t[{}]", bindings.join(" "));
            }
        }
        out.push('\n');
    }
    Ok((out, total))
}

pub(crate) async fn search(args: AstSearchArgs) -> Result<CallToolResult, McpError> {
    let max = args.max_results.unwrap_or(DEFAULT_MAX);

    let cmd: Vec<String> = match (&args.pattern, &args.rule) {
        (Some(_), Some(_)) => {
            return Err(invalid("supply `pattern` or `rule`, not both"));
        }
        (None, None) => return Err(invalid("supply either `pattern` or `rule`")),
        (Some(pattern), None) => {
            let Some(lang) = &args.lang else {
                return Err(invalid("`lang` is required when using `pattern`"));
            };
            vec![
                "run".into(),
                "--pattern".into(),
                pattern.clone(),
                "--lang".into(),
                lang.clone(),
                "--json".into(),
                args.path.clone(),
            ]
        }
        (None, Some(rule)) => vec![
            "scan".into(),
            "--inline-rules".into(),
            rule.clone(),
            "--json".into(),
            args.path.clone(),
        ],
    };

    let output = ast_grep(&cmd).await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut msg = format!("ast-grep failed: {}", stderr.trim());
        if args.rule.is_some() {
            let _ = write!(msg, "\n\n---\n{RULE_REFERENCE}");
        }
        return Err(failed(msg));
    }

    let (rendered, total) = render_matches(&String::from_utf8_lossy(&output.stdout), max)?;

    if total == 0 {
        let mut msg = String::from("No matches.\n");
        if let Some(rule) = &args.rule
            && missing_stop_by(rule)
        {
            msg.push_str(
                "This rule uses `inside`/`has` without `stopBy: end`, so traversal halts at the \
                 first non-matching node. Add `stopBy: end` and retry.\n",
            );
        }
        if args.pattern.is_some() {
            msg.push_str(
                "Check how the pattern parsed before rewriting it: \
                 `ast-grep run --pattern '<code>' --lang <lang> --debug-query=pattern`. \
                 Use `--debug-query=cst` on a known-matching file to find the right node kind.\n",
            );
        }
        return Ok(text(msg));
    }

    let mut body = rendered;
    if total > max {
        let _ = write!(
            body,
            "\nShowing {max} of {total} matches; narrow the pattern or raise `max_results`."
        );
    }
    Ok(text(body))
}

pub(crate) async fn outline(args: OutlineArgs) -> Result<CallToolResult, McpError> {
    let mut cmd: Vec<String> = vec!["outline".into(), args.path.clone()];
    if let Some(items) = &args.items {
        cmd.push("--items".into());
        cmd.push(items.clone());
    }
    if let Some(view) = &args.view {
        cmd.push("--view".into());
        cmd.push(view.clone());
    }
    if let Some(m) = &args.r#match {
        cmd.push("--match".into());
        cmd.push(m.clone());
    }
    if let Some(kind) = &args.kind {
        cmd.push("--type".into());
        cmd.push(kind.clone());
    }
    if args.pub_members.unwrap_or(false) {
        cmd.push("--pub-members".into());
    }

    let output = ast_grep(&cmd).await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(failed(format!(
            "ast-grep outline failed: {}",
            stderr.trim()
        )));
    }

    let body = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if body.is_empty() {
        return Ok(text(
            "No items found. ast-grep outline is syntax-only — check the path and that the language is supported.",
        ));
    }
    Ok(text(body))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::{missing_stop_by, render_matches};

    #[test]
    fn stop_by_hint_fires_only_for_relational_rules_without_it() {
        assert!(missing_stop_by("rule:\n  inside:\n    kind: function_item"));
        assert!(missing_stop_by("has:\n  pattern: await $E"));
        assert!(!missing_stop_by(
            "inside:\n  kind: function_item\n  stopBy: end"
        ));
        assert!(!missing_stop_by("pattern: console.log($A)"));
    }

    #[test]
    fn matches_render_one_based_with_metavariables() {
        let raw = r#"[{
            "text": "console.log(\"x\")",
            "file": "src/a.js",
            "range": {"start": {"line": 0, "column": 18}},
            "metaVariables": {"single": {"A": {"text": "\"x\""}}}
        }]"#;
        let (out, total) = render_matches(raw, 10).unwrap();
        assert_eq!(total, 1);
        // ast-grep is zero-based; the rendered location must be one-based.
        assert_eq!(out, "src/a.js:1:19\tconsole.log(\"x\")\t[$A=\"x\"]\n");
    }

    #[test]
    fn multiline_match_collapses_to_its_first_line() {
        let raw = r#"[{
            "text": "fn a() {\n  b();\n}",
            "file": "src/a.rs",
            "range": {"start": {"line": 4, "column": 0}}
        }]"#;
        let (out, _) = render_matches(raw, 10).unwrap();
        assert_eq!(out, "src/a.rs:5:1\tfn a() {\n");
    }

    #[test]
    fn total_counts_all_matches_not_just_rendered_ones() {
        let one = r#"{"text": "a", "file": "f", "range": {"start": {"line": 0, "column": 0}}}"#;
        let raw = format!("[{one},{one},{one}]");
        let (out, total) = render_matches(&raw, 2).unwrap();
        assert_eq!(total, 3, "total must reflect every match, not the cap");
        assert_eq!(out.lines().count(), 2);
    }

    #[test]
    fn empty_result_set_is_not_an_error() {
        let (out, total) = render_matches("[]", 10).unwrap();
        assert_eq!(total, 0);
        assert!(out.is_empty());
    }
}
