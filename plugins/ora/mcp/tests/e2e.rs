//! End-to-end coverage: every tool driven over stdio JSON-RPC against the real
//! binary, real ast-grep, real registries and real docs hosts.
//!
//! All of it is `#[ignore]`d, because it needs `ast-grep`, `git`, `gh` and the
//! network. Run it with:
//!
//! ```text
//! cargo test --test e2e -- --ignored              # everything
//! cargo test --test e2e -- --ignored local_       # no network
//! ```
//!
//! `net_repo_fetch_carries_credentials_without_leaking_them` additionally needs
//! `JMANGO_GITLAB_API_PAT`, and skips itself when it is absent.
// A panic is how a test reports, and indexing a JSON reply that does not have
// the expected shape is exactly the failure worth panicking on.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

mod harness;

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use harness::{Server, assert_err_contains, assert_ok_contains};

/// Every language the `Lang` enum offers. ast-grep dropping or renaming one is
/// invisible until a call fails in production, so the sweep below asks the
/// binary itself.
const LANGS: [&str; 29] = [
    "bash",
    "c",
    "cpp",
    "csharp",
    "css",
    "dart",
    "elixir",
    "go",
    "haskell",
    "hcl",
    "html",
    "java",
    "javascript",
    "json",
    "jsx",
    "kotlin",
    "lua",
    "markdown",
    "nix",
    "php",
    "python",
    "ruby",
    "rust",
    "scala",
    "solidity",
    "swift",
    "tsx",
    "typescript",
    "yaml",
];

const SYSTEMS: [&str; 7] = ["npm", "pypi", "go", "cargo", "maven", "nuget", "rubygems"];

/// Written fresh once per run — so a stale fixture cannot explain a pass — and
/// then shared, because the test harness runs these in parallel and a second
/// wipe would delete the tree out from under a test already reading it.
fn fixtures() -> &'static Path {
    static ROOT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    ROOT.get_or_init(build_fixtures)
}

fn build_fixtures() -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("fixtures");
    let _ = std::fs::remove_dir_all(&root);

    let write = |rel: &str, body: &str| {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    };

    write(
        "ts/app.ts",
        "export function greet(name: string): string {\n  \
         console.log(name);\n  console.log(\"hello\");\n  console.log(1);\n  return name;\n}\n\
         export const VERSION = \"1\";\n",
    );
    write(
        "tsx/app.tsx",
        "export const App = () => {\n  console.log(\"render\");\n  return <div />;\n};\n",
    );
    write("py/mod.py", "def greet(name):\n    print(name)\n");
    write(
        "go/main.go",
        "package main\n\nimport \"fmt\"\n\nfunc Greet(name string) {\n\tfmt.Println(name)\n}\n",
    );
    write(
        "java/Main.java",
        "public class Main {\n  public static void greet(String name) {\n    \
         System.out.println(name);\n  }\n}\n",
    );
    write("rb/a.rb", "def greet(name)\n  puts name\nend\n");
    write("yaml/conf.yaml", "service:\n  name: ora\n  port: 8080\n");
    write(
        "rs/lib.rs",
        "async fn work() -> u32 {\n    1\n}\n\n\
         pub async fn run() -> u32 {\n    let a = work().await;\n    a\n}\n\n\
         pub struct Config {\n    pub name: String,\n    secret: String,\n}\n",
    );
    write("txt/notes.txt", "plain prose, no grammar to parse\n");

    // A language sweep needs a path that parses in all 29 grammars; only an
    // empty directory qualifies.
    std::fs::create_dir_all(root.join("empty")).unwrap();

    root
}

fn p(root: &Path, rel: &str) -> String {
    root.join(rel).display().to_string()
}

// ---------------------------------------------------------------- schema ----

#[test]
#[ignore = "spawns the server binary"]
fn local_schemas_are_self_describing() {
    let mut server = Server::start();
    let tools = server.list_tools();

    let names: Vec<&str> = tools
        .iter()
        .filter_map(|t| t.get("name").and_then(Value::as_str))
        .collect();
    assert_eq!(
        names,
        // rmcp sorts the list; the order is the client's to present, not ours.
        [
            "ast_search",
            "lib_docs",
            "outline",
            "pkg_versions",
            "repo_fetch"
        ],
        "tool set changed"
    );

    for tool in &tools {
        let name = tool["name"].as_str().unwrap();
        let description = tool["description"].as_str().unwrap_or_default();
        assert!(
            description.len() > 120,
            "{name}: description is too thin to route on ({} chars)",
            description.len()
        );
        let annotations = tool
            .get("annotations")
            .unwrap_or_else(|| panic!("{name}: no annotations"));
        assert!(annotations.get("title").is_some(), "{name}: no title");
        assert!(
            annotations.get("readOnlyHint").is_some(),
            "{name}: no readOnlyHint"
        );

        // A `$ref` costs the model a hop to learn what it may pass, and some
        // clients do not resolve one at all.
        let schema = tool["inputSchema"].to_string();
        assert!(
            !schema.contains("$ref") && !schema.contains("$defs"),
            "{name}: schema is not inlined: {schema}"
        );
    }

    let by_name = |n: &str| {
        tools
            .iter()
            .find(|t| t["name"] == n)
            .unwrap_or_else(|| panic!("no {n}"))
            .clone()
    };

    let lang = by_name("ast_search")["inputSchema"]["properties"]["lang"].to_string();
    for value in LANGS {
        assert!(
            lang.contains(value),
            "ast_search.lang omits {value}: {lang}"
        );
    }

    let pkg = by_name("pkg_versions");
    let system = pkg["inputSchema"]["properties"]["system"].to_string();
    for value in SYSTEMS {
        assert!(system.contains(value), "pkg_versions.system omits {value}");
    }
    let required = pkg["inputSchema"]["required"].to_string();
    assert!(
        required.contains("system") && required.contains("packages"),
        "pkg_versions requires the wrong fields: {required}"
    );
}

// ------------------------------------------------------------ ast_search ----

#[test]
#[ignore = "needs ast-grep"]
fn local_ast_search_finds_and_reports_matches() {
    let root = fixtures();
    let mut server = Server::start();

    let hit = server.call(
        "ast_search",
        json!({"path": p(root, "ts"), "lang": "typescript", "pattern": "console.log($A)"}),
    );
    assert_ok_contains(
        &hit,
        "pattern match",
        &["app.ts:2:3", "console.log(name)", "$A=name"],
    );
    assert_eq!(
        hit.as_ref().unwrap().lines().count(),
        3,
        "expected one line per match"
    );

    // The cap has to be visible, otherwise a truncated result reads as the
    // complete picture.
    let capped = server.call(
        "ast_search",
        json!({"path": p(root, "ts"), "lang": "typescript", "pattern": "console.log($A)", "max_results": 1}),
    );
    assert_ok_contains(&capped, "max_results", &["Showing 1 of 3 matches"]);

    // Structure, not text: a relational rule that a bare pattern cannot express.
    let relational = server.call(
        "ast_search",
        json!({"path": p(root, "rs"), "rule": "id: awaited\nlanguage: rust\nrule:\n  pattern: $E.await\n  inside:\n    kind: function_item\n    stopBy: end"}),
    );
    assert_ok_contains(
        &relational,
        "relational rule",
        &["lib.rs:6:13", "work().await"],
    );
}

#[test]
#[ignore = "needs ast-grep"]
fn local_ast_search_reports_an_empty_result_as_a_result() {
    let root = fixtures();
    let mut server = Server::start();

    // `ast-grep run` exits 1 on zero matches, so this used to surface as a tool
    // error with an empty message — on the single most common outcome of a
    // first search attempt.
    let empty = server.call(
        "ast_search",
        json!({"path": p(root, "ts"), "lang": "typescript", "pattern": "notCalledAnywhere($A)"}),
    );
    assert_ok_contains(&empty, "no matches", &["No matches.", "--debug-query"]);

    // The one hint that turns a silently empty relational rule into a fix.
    let no_stop_by = server.call(
        "ast_search",
        json!({"path": p(root, "rs"), "rule": "id: awaited\nlanguage: rust\nrule:\n  pattern: $E.await\n  inside:\n    kind: function_item"}),
    );
    assert_ok_contains(
        &no_stop_by,
        "missing stopBy",
        &["No matches.", "stopBy: end"],
    );
    assert!(
        !no_stop_by.as_ref().unwrap().contains("Rule Reference"),
        "an empty result must not cost a full rule reference"
    );
}

#[test]
#[ignore = "needs ast-grep"]
fn local_ast_search_rejects_bad_input_with_a_way_forward() {
    let root = fixtures();
    let mut server = Server::start();

    let cases: [(&str, Value, &[&str]); 6] = [
        (
            "no pattern and no rule",
            json!({"path": p(root, "ts")}),
            &["pattern", "rule", "\"lang\": \"typescript\""],
        ),
        (
            "both pattern and rule",
            json!({"path": p(root, "ts"), "lang": "typescript", "pattern": "a()", "rule": "id: r"}),
            &["not both"],
        ),
        (
            "pattern without lang",
            json!({"path": p(root, "ts"), "pattern": "console.log($A)"}),
            &["`lang` is required"],
        ),
        (
            "unsupported language",
            json!({"path": p(root, "ts"), "lang": "sql", "pattern": "SELECT $A"}),
            // The rejected value and the whole legal set, so a retry needs no
            // second lookup.
            &["unknown variant `sql`", "`typescript`", "`rust`"],
        ),
        (
            // Exits 1 with an empty match list, exactly like a genuine miss;
            // only stderr tells them apart.
            "path that does not exist",
            json!({"path": p(root, "no/such/dir"), "lang": "rust", "pattern": "fn $N()"}),
            &["No such file"],
        ),
        (
            "rule that does not compile",
            json!({"path": p(root, "rs"), "rule": "id: r\nlanguage: rust\nrule:\n  bogus_key: 1"}),
            &["Cannot parse rule", "ast-grep Rule Reference"],
        ),
    ];

    for (case, args, needles) in cases {
        let result = server.call("ast_search", args);
        assert_err_contains(&result, case, needles);
    }
}

#[test]
#[ignore = "needs ast-grep"]
fn local_every_advertised_language_is_one_ast_grep_accepts() {
    let root = fixtures();
    let mut server = Server::start();

    for lang in LANGS {
        let result = server.call(
            "ast_search",
            json!({"path": p(root, "empty"), "lang": lang, "pattern": "x"}),
        );
        assert_ok_contains(&result, lang, &["No matches."]);
    }
}

#[test]
#[ignore = "needs ast-grep"]
fn local_ast_search_works_across_grammars() {
    let root = fixtures();
    let mut server = Server::start();

    let cases: [(&str, &str, &str, &str); 6] = [
        ("py", "python", "print($A)", "print(name)"),
        ("go", "go", "func $N($$$P)", "func Greet(name string)"),
        (
            "java",
            "java",
            "System.out.println($A)",
            "System.out.println(name)",
        ),
        ("rb", "ruby", "puts $A", "puts name"),
        ("tsx", "tsx", "console.log($A)", "console.log(\"render\")"),
        ("rs", "rust", "pub struct $N { $$$F }", "pub struct Config"),
    ];

    for (dir, lang, pattern, expected) in cases {
        let result = server.call(
            "ast_search",
            json!({"path": p(root, dir), "lang": lang, "pattern": pattern}),
        );
        assert_ok_contains(&result, lang, &[expected]);
    }
}

// ---------------------------------------------------------------- outline ----

#[test]
#[ignore = "needs ast-grep"]
fn local_outline_renders_every_items_and_view_combination() {
    let root = fixtures();
    let mut server = Server::start();

    let file = p(root, "rs/lib.rs");
    for items in ["auto", "structure", "exports", "imports", "all"] {
        for view in ["auto", "names", "signatures", "digest", "expanded"] {
            let result = server.call(
                "outline",
                json!({"path": file, "items": items, "view": view}),
            );
            // `imports` legitimately finds nothing in this fixture; what matters
            // is that no combination errors out.
            assert!(
                result.is_ok(),
                "outline items={items} view={view} failed: {}",
                result.unwrap_err()
            );
        }
    }

    assert_ok_contains(
        &server.call("outline", json!({"path": file})),
        "default view",
        &["pub async fn run", "struct Config"],
    );

    // A directory outlines its public surface rather than its internals.
    assert_ok_contains(
        &server.call(
            "outline",
            json!({"path": p(root, "ts"), "items": "exports", "view": "names"}),
        ),
        "directory exports",
        &["greet"],
    );
}

#[test]
#[ignore = "needs ast-grep"]
fn local_outline_filters_narrow_the_result() {
    let root = fixtures();
    let mut server = Server::start();
    let file = p(root, "rs/lib.rs");

    let matched = server.call("outline", json!({"path": file, "match": "^run$"}));
    assert_ok_contains(&matched, "match filter", &["run"]);
    assert!(
        !matched.as_ref().unwrap().contains("struct Config"),
        "match filter did not exclude other items"
    );

    let kinds = server.call("outline", json!({"path": file, "kind": "function"}));
    assert_ok_contains(&kinds, "kind filter", &["fn run"]);
    assert!(
        !kinds.as_ref().unwrap().contains("struct Config"),
        "kind filter did not exclude the struct"
    );

    assert_ok_contains(
        &server.call(
            "outline",
            json!({"path": file, "view": "expanded", "pub_members": true}),
        ),
        "pub members",
        &["name"],
    );
}

#[test]
#[ignore = "needs ast-grep"]
fn local_outline_distinguishes_nothing_found_from_a_bad_path() {
    let root = fixtures();
    let mut server = Server::start();

    // ast-grep prints "nothing found" and exits 0 in both cases; only stderr
    // separates an unparseable file from a path that is not there.
    assert_ok_contains(
        &server.call("outline", json!({"path": p(root, "txt/notes.txt")})),
        "unsupported language",
        &["No items found", "syntax-only"],
    );
    assert_err_contains(
        &server.call("outline", json!({"path": p(root, "no/such/file.rs")})),
        "missing path",
        &["No such file"],
    );
    assert_err_contains(
        &server.call(
            "outline",
            json!({"path": p(root, "rs/lib.rs"), "kind": "nosuchkind"}),
        ),
        "unknown kind",
        &["Unknown outline symbol type"],
    );
}

// ---------------------------------------------------------- pkg_versions ----

#[test]
#[ignore = "hits deps.dev"]
fn net_pkg_versions_resolves_every_registry() {
    let mut server = Server::start();

    let cases: [(&str, &[&str], &[&str]); 7] = [
        ("npm", &["react", "@types/node"], &["react", "@types/node"]),
        ("pypi", &["requests", "django"], &["requests", "django"]),
        ("go", &["github.com/gin-gonic/gin"], &["gin-gonic/gin"]),
        ("cargo", &["serde", "tokio"], &["serde", "tokio"]),
        ("maven", &["com.google.guava:guava"], &["guava"]),
        ("nuget", &["Newtonsoft.Json"], &["Newtonsoft.Json"]),
        ("rubygems", &["rails"], &["rails"]),
    ];

    for (system, packages, needles) in cases {
        let result = server.call(
            "pkg_versions",
            json!({"system": system, "packages": packages}),
        );
        assert_ok_contains(&result, system, needles);
        let body = result.unwrap();
        assert!(
            body.starts_with("package\tversion\tpublished\tstatus"),
            "{system}: header missing"
        );
        for line in body.lines().skip(1).filter(|l| !l.is_empty()) {
            assert!(
                line.ends_with("\tok") || line.ends_with("\tdeprecated"),
                "{system}: unresolved row: {line}"
            );
        }
    }
}

#[test]
#[ignore = "hits deps.dev"]
fn net_pkg_versions_flags_deprecation_and_misses() {
    let mut server = Server::start();

    // A silently-deprecated dependency is the failure this tool exists to
    // prevent, so the callout has to survive refactors.
    assert_ok_contains(
        &server.call(
            "pkg_versions",
            json!({"system": "npm", "packages": ["request"]}),
        ),
        "deprecated",
        &[
            "deprecated",
            "Deprecated: request",
            "Surface this to the user",
        ],
    );

    // One bad name must not sink the rest of the batch.
    let mixed = server.call(
        "pkg_versions",
        json!({"system": "npm", "packages": ["react", "ora-mcp-package-that-does-not-exist"]}),
    );
    assert_ok_contains(&mixed, "partial miss", &["not found"]);
    let body = mixed.unwrap();
    assert!(
        body.lines()
            .any(|l| l.starts_with("react\t") && l.ends_with("\tok")),
        "the valid package was lost with the invalid one:\n{body}"
    );
    // Order is the request order, so a batched answer can be read against the
    // list that was sent.
    let react = body.find("react\t").unwrap();
    let missing = body.find("ora-mcp-package").unwrap();
    assert!(react < missing, "rows came back out of order:\n{body}");
}

#[test]
#[ignore = "spawns the server binary"]
fn net_pkg_versions_rejects_bad_input() {
    let mut server = Server::start();

    assert_err_contains(
        &server.call("pkg_versions", json!({"system": "npm", "packages": []})),
        "empty batch",
        &["must not be empty", "\"packages\": [\"react\", \"zod\"]"],
    );
    assert_err_contains(
        &server.call(
            "pkg_versions",
            json!({"system": "crates.io", "packages": ["serde"]}),
        ),
        "unknown registry",
        // The near-miss is the common one: the registry is `cargo`, not the site.
        &["unknown variant `crates.io`", "`cargo`"],
    );
}

// -------------------------------------------------------------- lib_docs ----

#[test]
#[ignore = "hits live docs hosts"]
fn net_lib_docs_returns_a_corpus_that_fits() {
    let mut server = Server::start();

    // Content-length known and under budget.
    let measured = server.call("lib_docs", json!({"domain": "zod.dev"}));
    assert_ok_contains(&measured, "zod.dev", &["Zod"]);
    assert!(
        !measured.as_ref().unwrap().contains("Page index below"),
        "a corpus that fits was replaced by an index"
    );

    // No content-length at all. Treating that as "too large" used to send back
    // an index for a corpus that fits comfortably.
    let unmeasured = server.call("lib_docs", json!({"domain": "hono.dev"}));
    assert_ok_contains(&unmeasured, "hono.dev", &["Hono"]);
    assert!(
        !unmeasured.as_ref().unwrap().contains("Page index below"),
        "an unmeasurable corpus was refused instead of fetched"
    );
}

#[test]
#[ignore = "hits live docs hosts"]
fn net_lib_docs_falls_back_to_an_index_when_the_corpus_is_too_large() {
    let mut server = Server::start();

    // Cloudflare publishes ~46 MB. The reply has to stay small, and it has to
    // name the next step rather than just refusing.
    let huge = server.call("lib_docs", json!({"domain": "docs.cloudflare.com"}));
    assert_ok_contains(
        &huge,
        "cloudflare",
        &["Page index below", "`url` set to the page you need"],
    );
    let body = huge.unwrap();
    assert!(
        body.len() < 600_000,
        "index reply grew to {} bytes",
        body.len()
    );

    // The index lists real pages, and fetching one by URL is the documented
    // follow-up, so it has to work.
    let page = body
        .lines()
        .find_map(|line| {
            let start = line.find("](https://")? + 2;
            let rest = line.get(start..)?;
            let end = rest.find(')')?;
            rest.get(..end)
        })
        .expect("no page URL in the index")
        .to_string();
    assert_ok_contains(
        &server.call(
            "lib_docs",
            json!({"domain": "docs.cloudflare.com", "url": page}),
        ),
        "index follow-up",
        &[""],
    );
}

#[test]
#[ignore = "hits live docs hosts"]
fn net_lib_docs_caps_and_falls_back() {
    let mut server = Server::start();

    // A cap below the corpus size must cut the reply, not the connection.
    let capped = server.call(
        "lib_docs",
        json!({"domain": "zod.dev", "url": "https://zod.dev/llms-full.txt", "max_bytes": 2000}),
    );
    assert_ok_contains(&capped, "explicit cap", &["[cut at 2000 bytes"]);
    assert!(
        capped.unwrap().len() < 2400,
        "the cap was applied after the fact, not to the download"
    );

    // No llms.txt anywhere: say so, and name the tool that does cover it.
    assert_ok_contains(
        &server.call("lib_docs", json!({"domain": "example.com"})),
        "no llms.txt",
        &["No llms.txt", "ctx7"],
    );

    // A domain that does not resolve is an error, not an empty answer.
    assert_ok_contains(
        &server.call(
            "lib_docs",
            json!({"domain": "ora-mcp-nonexistent-host.invalid"}),
        ),
        "dead host",
        &["No llms.txt"],
    );
}

// ------------------------------------------------------------ repo_fetch ----

#[test]
#[ignore = "needs gh auth and network"]
fn net_repo_fetch_reads_single_files() {
    let mut server = Server::start();

    assert_ok_contains(
        &server.call(
            "repo_fetch",
            json!({"repo": "tokio-rs/tokio", "path": "README.md"}),
        ),
        "github file",
        &["Tokio"],
    );

    // A pinned ref must actually pin: this is what makes a quoted line
    // reproducible.
    assert_ok_contains(
        &server.call(
            "repo_fetch",
            json!({
                "repo": "tokio-rs/tokio",
                "path": "tokio/Cargo.toml",
                "ref": "tokio-1.0.0",
            }),
        ),
        "pinned ref",
        &["version = \"1.0.0\""],
    );

    assert_err_contains(
        &server.call(
            "repo_fetch",
            json!({"repo": "tokio-rs/tokio", "path": "no/such/file.rs"}),
        ),
        "missing file",
        &["gh api failed"],
    );
}

#[test]
#[ignore = "spawns the server binary"]
fn net_repo_fetch_rejects_bad_input() {
    let mut server = Server::start();

    let cases: [(&str, Value, &[&str]); 3] = [
        (
            "neither path nor clone",
            json!({"repo": "tokio-rs/tokio"}),
            &["\"path\": \"README.md\"", "clone"],
        ),
        (
            "unparseable repo",
            json!({"repo": "justaword", "path": "README.md"}),
            &["cannot parse repo"],
        ),
        (
            // Reading one file is GitHub-only, so the reply has to hand over the
            // route that does work rather than just refusing.
            "single file from a non-GitHub host",
            json!({"repo": "https://gitlab.com/gitlab-org/gitlab-foss", "path": "README.md"}),
            &["GitHub-only", "clone"],
        ),
    ];

    for (case, args, needles) in cases {
        assert_err_contains(&server.call("repo_fetch", args), case, needles);
    }
}

#[test]
#[ignore = "clones over the network"]
fn net_repo_fetch_clones_from_any_host_and_reuses_the_cache() {
    let mut server = Server::start();

    let first = server.call(
        "repo_fetch",
        json!({"repo": "https://gitlab.com/gitlab-org/gitlab-runner-docker-cleanup", "clone": true}),
    );
    let path = first.expect("gitlab clone failed");
    assert!(
        Path::new(&path).join(".git").is_dir(),
        "clone returned {path}, which is not a checkout"
    );

    // The second call must land on the same checkout instead of cloning again.
    let second = server.call(
        "repo_fetch",
        json!({"repo": "https://gitlab.com/gitlab-org/gitlab-runner-docker-cleanup", "clone": true}),
    );
    assert_eq!(second.unwrap(), path, "cache slot moved between calls");

    // Refreshing an existing clone keeps the same slot.
    let refreshed = server.call(
        "repo_fetch",
        json!({
            "repo": "https://gitlab.com/gitlab-org/gitlab-runner-docker-cleanup",
            "clone": true,
            "refresh": true,
        }),
    );
    assert_eq!(refreshed.unwrap(), path, "refresh relocated the clone");

    // A pinned clone is a separate slot, so the two cannot overwrite each other.
    let pinned = server
        .call(
            "repo_fetch",
            json!({"repo": "tokio-rs/tokio", "clone": true, "ref": "tokio-1.0.0"}),
        )
        .expect("pinned clone failed");
    assert!(
        pinned.contains("tokio-1.0.0"),
        "pinned clone shares a slot: {pinned}"
    );

    // A cloned checkout is what the other tools then run over.
    assert_ok_contains(
        &server.call(
            "outline",
            json!({"path": format!("{pinned}/tokio/src/lib.rs")}),
        ),
        "outline over a clone",
        &["pub"],
    );
}

#[test]
#[ignore = "needs JMANGO_GITLAB_API_PAT"]
fn net_repo_fetch_carries_credentials_without_leaking_them() {
    let Ok(token) = std::env::var("JMANGO_GITLAB_API_PAT") else {
        eprintln!("skipped: JMANGO_GITLAB_API_PAT is not set");
        return;
    };
    assert!(!token.is_empty(), "JMANGO_GITLAB_API_PAT is empty");

    let mut server = Server::start();

    // The only way to reach a private host through git is credentials in the
    // URL, which is exactly how a token ends up in a cache path or an error
    // message if nothing strips it.
    let url = format!("https://oauth2:{token}@gitlab.jmango360.com/cong.tran/jclaude.git");
    let path = server
        .call("repo_fetch", json!({"repo": url, "clone": true}))
        .expect("authenticated clone failed");

    assert!(
        !path.contains(&token),
        "the token was written into the clone path"
    );
    assert!(
        path.contains("gitlab.jmango360.com--cong.tran--jclaude"),
        "unexpected cache slot: {path}"
    );
    assert!(
        Path::new(&path).join(".git").is_dir(),
        "authenticated clone produced no checkout"
    );

    // And the failure path, where git echoes the remote URL back.
    let bad = format!("https://oauth2:{token}@gitlab.jmango360.com/cong.tran/no-such-repo.git");
    let error = server
        .call("repo_fetch", json!({"repo": bad, "clone": true}))
        .expect_err("cloning a missing repo should fail");
    assert!(
        !error.contains(&token),
        "the token was echoed back in an error message"
    );
}
