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

str_enum! {
    /// Languages this ast-grep build parses. Verified against the binary rather
    /// than the docs page, which omits `markdown` and lists alias spellings.
    Lang {
        Bash => "bash",
        C => "c",
        Cpp => "cpp",
        CSharp => "csharp",
        Css => "css",
        Dart => "dart",
        Elixir => "elixir",
        Go => "go",
        Haskell => "haskell",
        Hcl => "hcl",
        Html => "html",
        Java => "java",
        JavaScript => "javascript",
        Json => "json",
        Jsx => "jsx",
        Kotlin => "kotlin",
        Lua => "lua",
        Markdown => "markdown",
        Nix => "nix",
        Php => "php",
        Python => "python",
        Ruby => "ruby",
        Rust => "rust",
        Scala => "scala",
        Solidity => "solidity",
        Swift => "swift",
        Tsx => "tsx",
        TypeScript => "typescript",
        Yaml => "yaml",
    }
}

str_enum! {
    Items {
        Auto => "auto",
        Structure => "structure",
        Exports => "exports",
        Imports => "imports",
        All => "all",
    }
}

str_enum! {
    View {
        Auto => "auto",
        Names => "names",
        Signatures => "signatures",
        Digest => "digest",
        Expanded => "expanded",
    }
}

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
    /// Language of the pattern. Required with `pattern`; a `rule` carries its own
    /// `language` field instead.
    #[serde(default)]
    pub(crate) lang: Option<Lang>,
    /// Cap on reported matches (default 60).
    #[serde(default)]
    pub(crate) max_results: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub(crate) struct OutlineArgs {
    /// File or directory to outline.
    pub(crate) path: String,
    /// Which top-level items to include. `auto` (the default) means `structure`
    /// for a file and `exports` for a directory.
    #[serde(default)]
    pub(crate) items: Option<Items>,
    /// How much text per item, in increasing detail. `auto` (the default) means
    /// `digest` for a file and `names` for a directory.
    #[serde(default)]
    pub(crate) view: Option<View>,
    /// Regex filter over top-level item names. Never filters members.
    #[serde(default)]
    pub(crate) r#match: Option<String>,
    /// Comma-separated top-level kinds to keep, e.g. `class,enum`. Kind names are
    /// language-specific.
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

/// What ast-grep wrote to stderr, split into the part that means the run failed
/// and the part that is merely advice.
///
/// The exit code cannot carry this: `run` exits 1 on zero matches (grep
/// convention) while `scan` exits 0, and `outline` exits 0 even for a path that
/// does not exist. Reading the status would turn every empty result into an
/// error and every mistyped path into a silent "nothing found". stderr holds the
/// truth, but it also holds warnings — a pattern that parses into an ERROR node
/// still runs and still matches — so only the error blocks are fatal.
fn diagnostics(output: &std::process::Output) -> (Option<String>, Option<String>) {
    let text = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if text.is_empty() {
        return (None, None);
    }
    // A fatal block carries its own `Help:` and `Caused by:` continuation lines,
    // so one error line makes the whole of stderr worth surfacing.
    let fatal = text
        .lines()
        .any(|l| l.trim_start().to_ascii_lowercase().starts_with("error"));
    if fatal {
        (Some(text), None)
    } else {
        (None, Some(text))
    }
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
            return Err(invalid(
                "supply `pattern` or `rule`, not both. A `rule` can express anything a \
                 `pattern` can, so drop the `pattern` if you need relational matching.",
            ));
        }
        (None, None) => {
            return Err(invalid(
                "supply either `pattern` or `rule`. Pattern: \
                 {\"path\": \"src\", \"lang\": \"typescript\", \"pattern\": \"useEffect($A, $B)\"}. \
                 Rule: {\"path\": \"src\", \"rule\": \"id: r\\nlanguage: typescript\\nrule:\\n  \
                 pattern: await $E\\n  inside:\\n    kind: function_declaration\\n    stopBy: end\"}.",
            ));
        }
        (Some(pattern), None) => {
            let Some(lang) = args.lang else {
                return Err(invalid(
                    "`lang` is required when using `pattern` — ast-grep cannot parse a \
                     pattern without knowing its grammar.",
                ));
            };
            vec![
                "run".into(),
                "--pattern".into(),
                pattern.clone(),
                "--lang".into(),
                lang.as_str().into(),
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
    let (fatal, warning) = diagnostics(&output);
    if let Some(stderr) = fatal {
        let mut msg = format!("ast-grep failed: {stderr}");
        if args.rule.is_some() {
            let _ = write!(msg, "\n\n---\n{RULE_REFERENCE}");
        }
        return Err(failed(msg));
    }

    let (rendered, total) = render_matches(&String::from_utf8_lossy(&output.stdout), max)?;

    if total == 0 {
        let mut msg = String::from("No matches.\n");
        // Usually the reason: the pattern did not parse into the node the model
        // meant, which the warning says outright.
        if let Some(w) = &warning {
            let _ = writeln!(msg, "{w}");
        }
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
                 Use `--debug-query=cst` on a known-matching file to find the right node kind.\n\
                 A fragment that is ambiguous on its own parses as the wrong kind — Go's \
                 `fmt.Println($A)` becomes a type conversion, not a call — and no rewriting of \
                 the pattern fixes that. Give it context through a rule instead:\n  \
                 rule:\n    pattern:\n      context: \"func f() { fmt.Println($A) }\"\n      \
                 selector: call_expression\n",
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
    // Matches found despite a warning still deserve the caveat: an ERROR node in
    // the pattern means some of them may be wrong.
    if let Some(w) = warning {
        let _ = write!(body, "\n{w}");
    }
    Ok(text(body))
}

pub(crate) async fn outline(args: OutlineArgs) -> Result<CallToolResult, McpError> {
    let mut cmd: Vec<String> = vec!["outline".into(), args.path.clone()];
    if let Some(items) = args.items {
        cmd.push("--items".into());
        cmd.push(items.as_str().into());
    }
    if let Some(view) = args.view {
        cmd.push("--view".into());
        cmd.push(view.as_str().into());
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
    if let (Some(stderr), _) = diagnostics(&output) {
        return Err(failed(format!("ast-grep outline failed: {stderr}")));
    }

    let body = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !has_items(&body) {
        return Ok(text(
            "No items found. Outlining is syntax-only and covers a fixed set of languages, \
             so a file in an unsupported language yields nothing even when it parses elsewhere. \
             Any `match`/`kind` filter applies to top-level names only.",
        ));
    }
    Ok(text(body))
}

/// A file that yields nothing still prints its path plus a literal "nothing
/// found", so an empty outline is not an empty string. Every item line is
/// labelled — by line number in most views, by symbol type in `names` — and that
/// label is what separates one from the bare path headers around it.
fn has_items(stdout: &str) -> bool {
    stdout
        .lines()
        .any(|line| line.contains(": ") && line.trim() != "nothing found")
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::{Items, Lang, View, diagnostics, has_items, missing_stop_by, render_matches};

    fn stderr(text: &str) -> std::process::Output {
        std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: Vec::new(),
            stderr: text.as_bytes().to_vec(),
        }
    }

    #[test]
    fn warnings_are_advice_and_errors_are_fatal() {
        // Every wording ast-grep 0.45 uses for a real failure: clap's, the
        // walker's, and the rule parser's.
        for fatal in [
            "error: invalid value 'notalang' for '--lang <LANG>'",
            "ERROR: /tmp/x: No such file or directory (os error 2)",
            "Error: Cannot parse rule INLINE_RULES\nHelp: refer to doc",
        ] {
            let (err, warn) = diagnostics(&stderr(fatal));
            assert_eq!(err.as_deref(), Some(fatal.trim()), "should be fatal");
            assert!(warn.is_none());
        }

        // A pattern that parses into an ERROR node still runs, so this must not
        // sink the call — but it is the explanation for an empty result.
        let (err, warn) = diagnostics(&stderr(
            "Warning: Pattern contains an ERROR node and may cause unexpected results.\n\
             Help: ast-grep parsed the pattern but it matched nothing in this run.",
        ));
        assert!(err.is_none(), "a warning must not be fatal");
        assert!(warn.is_some_and(|w| w.contains("ERROR node")));

        let (err, warn) = diagnostics(&stderr("   \n "));
        assert!(err.is_none() && warn.is_none(), "blank stderr says nothing");
    }

    #[test]
    fn outline_items_are_told_apart_from_path_headers() {
        assert!(has_items("/tmp/a.rs\n1: fn main()\n"));
        assert!(has_items("  12: class Foo"));
        // The `names` view labels by symbol type instead of line number.
        assert!(has_items("/tmp/app.ts\nfunction: greet\nconstant: VERSION"));
        // A path header plus ast-grep's literal miss marker is an empty outline,
        // even though the body is not an empty string.
        assert!(!has_items("/tmp/a.rs\nnothing found"));
        assert!(!has_items("nothing found"));
        assert!(!has_items(""));
        // One file yielding nothing does not empty a whole directory outline.
        assert!(has_items(
            "/tmp/a.rs\nnothing found\n/tmp/b.rs\nfunction: run"
        ));
    }

    /// The wire literal is what reaches the ast-grep CLI *and* what the schema
    /// advertises, so a typo in one would silently offer the model a value the
    /// binary rejects.
    #[test]
    fn every_enum_value_round_trips_through_its_wire_name() {
        for v in Lang::ALL {
            assert_eq!(
                serde_json::from_value::<Lang>(serde_json::json!(v.as_str())).unwrap(),
                *v
            );
        }
        for v in Items::ALL {
            assert_eq!(
                serde_json::from_value::<Items>(serde_json::json!(v.as_str())).unwrap(),
                *v
            );
        }
        for v in View::ALL {
            assert_eq!(
                serde_json::from_value::<View>(serde_json::json!(v.as_str())).unwrap(),
                *v
            );
        }
    }

    #[test]
    fn unsupported_languages_are_rejected_at_the_schema_boundary() {
        // ast-grep 0.45 parses none of these; the enum is what keeps them out.
        for bad in ["sql", "zig", "toml", "terraform", "vue"] {
            assert!(serde_json::from_value::<Lang>(serde_json::json!(bad)).is_err());
        }
    }

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
