use std::path::PathBuf;
use std::process::Stdio;

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use tokio::process::Command;

use super::{failed, invalid, text};

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub(crate) struct RepoFetchArgs {
    /// `owner/repo`, an HTTPS URL, or an SSH URL. Any git host works for cloning;
    /// reading a single file without cloning is GitHub-only.
    pub(crate) repo: String,
    /// Path of one file to read. Supply this or `clone`; one of the two is required.
    #[serde(default)]
    pub(crate) path: Option<String>,
    /// Branch, tag, or commit. Defaults to the repo's default branch.
    #[serde(default, rename = "ref")]
    pub(crate) git_ref: Option<String>,
    /// Shallow-clone into a local cache and return the path. Use for a deep dive
    /// across 3+ files, branching follow-ups, or running ast-grep over the source.
    #[serde(default)]
    pub(crate) clone: Option<bool>,
    /// Re-fetch a cached clone instead of reusing the state on disk.
    #[serde(default)]
    pub(crate) refresh: Option<bool>,
}

/// GitHub shortcut, HTTPS, and SSH forms all normalise to a clone URL.
fn clone_url(repo: &str) -> Result<String, McpError> {
    if repo.contains("://") || (repo.contains('@') && repo.contains(':')) {
        return Ok(repo.to_string());
    }
    if repo.contains('/') && !repo.contains(' ') {
        return Ok(format!("https://github.com/{repo}.git"));
    }
    Err(invalid(format!(
        "cannot parse repo `{repo}` — use owner/repo, https://…, or git@host:path"
    )))
}

/// `owner/repo` for GitHub URLs in any form, else None.
fn github_slug(repo: &str) -> Option<String> {
    let rest = if repo.contains("://") || repo.contains('@') {
        let after_host = repo
            .split_once("github.com")
            .map(|(_, rest)| rest.trim_start_matches([':', '/']))?;
        after_host.to_string()
    } else if repo.contains('/') {
        repo.to_string()
    } else {
        return None;
    };

    let cleaned = rest.trim_end_matches('/').trim_end_matches(".git");
    let mut parts = cleaned.split('/');
    let (owner, name) = (parts.next()?, parts.next()?);
    (!owner.is_empty() && !name.is_empty()).then(|| format!("{owner}/{name}"))
}

/// Mirrors the cache layout the previous shell script used, so existing clones
/// under ~/.cache/clio-repos are reused rather than duplicated.
fn cache_key(url: &str, git_ref: Option<&str>) -> String {
    let mut key = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("ssh://")
        .to_string();
    // A clone URL may carry credentials (`https://oauth2:TOKEN@host/…`, or just
    // `git@host`). Dropping the userinfo keeps a token from being written into a
    // directory name that then persists on disk, and lands an authenticated
    // clone in the same cache slot as an anonymous one.
    let authority_end = key.find('/').unwrap_or(key.len());
    if let Some(at) = key.get(..authority_end).and_then(|a| a.rfind('@')) {
        key.drain(..=at);
    }
    if let Some(pos) = key.find(':') {
        key.replace_range(pos..=pos, "/");
    }
    let key = key.trim_end_matches('/').trim_end_matches(".git");
    let mut out = key.replace('/', "--");
    if let Some(r) = git_ref {
        out.push_str("--");
        out.push_str(&r.replace('/', "_"));
    }
    out
}

/// git echoes the remote URL in most of its failures, and that URL may carry a
/// token. Every git error therefore passes through here before it can reach the
/// model, the transcript, or a log.
fn redact(msg: &str) -> String {
    let mut out = String::with_capacity(msg.len());
    let mut rest = msg;
    while let Some(pos) = rest.find("://") {
        let (before, after) = rest.split_at(pos + 3);
        out.push_str(before);
        // The authority ends at the path separator, or wherever the surrounding
        // prose resumes if git wrapped the URL in quotes.
        let end = after
            .find(['/', ' ', '\t', '\n', '\'', '"'])
            .unwrap_or(after.len());
        let (authority, tail) = after.split_at(end);
        match authority.rfind('@') {
            Some(at) => {
                out.push_str("***@");
                out.push_str(authority.get(at + 1..).unwrap_or_default());
            }
            None => out.push_str(authority),
        }
        rest = tail;
    }
    out.push_str(rest);
    out
}

fn git_failed(action: &str, output: &std::process::Output) -> McpError {
    failed(format!(
        "git {action} failed: {}",
        redact(String::from_utf8_lossy(&output.stderr).trim())
    ))
}

async fn run_cmd(bin: &str, args: &[String]) -> Result<std::process::Output, McpError> {
    Command::new(bin)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                failed(format!(
                    "`{bin}` not found on PATH; install it (`brew install {bin}`)"
                ))
            } else {
                failed(format!("failed to run {bin}: {e}"))
            }
        })
}

async fn do_clone(args: &RepoFetchArgs, url: String) -> Result<CallToolResult, McpError> {
    let base = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| failed("HOME is not set; cannot locate the repo cache"))?
        .join(".cache/clio-repos");
    let dest = base.join(cache_key(&url, args.git_ref.as_deref()));
    let dest_str = dest.display().to_string();

    if dest.join(".git").is_dir() {
        if args.refresh.unwrap_or(false) {
            let target = args.git_ref.clone().unwrap_or_else(|| "HEAD".into());
            let fetch = run_cmd(
                "git",
                &[
                    "-C".into(),
                    dest_str.clone(),
                    "fetch".into(),
                    "--depth=1".into(),
                    "--quiet".into(),
                    "origin".into(),
                    target,
                ],
            )
            .await?;
            if !fetch.status.success() {
                return Err(git_failed("fetch", &fetch));
            }
            let reset = run_cmd(
                "git",
                &[
                    "-C".into(),
                    dest_str.clone(),
                    "reset".into(),
                    "--hard".into(),
                    "--quiet".into(),
                    "FETCH_HEAD".into(),
                ],
            )
            .await?;
            if !reset.status.success() {
                return Err(git_failed("reset", &reset));
            }
        }
        return Ok(text(dest_str));
    }

    std::fs::create_dir_all(&base)
        .map_err(|e| failed(format!("cannot create {}: {e}", base.display())))?;

    let mut cmd: Vec<String> = vec!["clone".into(), "--depth=1".into(), "--quiet".into()];
    if let Some(r) = &args.git_ref {
        cmd.push("--branch".into());
        cmd.push(r.clone());
        cmd.push("--single-branch".into());
    }
    cmd.push(url);
    cmd.push(dest_str.clone());

    let out = run_cmd("git", &cmd).await?;
    if !out.status.success() {
        return Err(git_failed("clone", &out));
    }
    Ok(text(dest_str))
}

async fn read_file(args: &RepoFetchArgs, path: &str) -> Result<CallToolResult, McpError> {
    let Some(slug) = github_slug(&args.repo) else {
        return Err(invalid(format!(
            "reading one file without cloning is GitHub-only; `{}` is elsewhere — \
             call again with `clone: true` and read from the returned path",
            args.repo
        )));
    };

    let mut endpoint = format!("repos/{slug}/contents/{path}");
    if let Some(r) = &args.git_ref {
        endpoint.push_str("?ref=");
        endpoint.push_str(r);
    }

    let out = run_cmd(
        "gh",
        &[
            "api".into(),
            endpoint,
            "-H".into(),
            "Accept: application/vnd.github.raw".into(),
        ],
    )
    .await?;

    if !out.status.success() {
        return Err(failed(format!(
            "gh api failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(text(String::from_utf8_lossy(&out.stdout).into_owned()))
}

pub(crate) async fn run(args: RepoFetchArgs) -> Result<CallToolResult, McpError> {
    // Parsed up front, before any mode is chosen: a repo string no mode could
    // use must fail as one, rather than reaching the GitHub-only branch below
    // and being told to retry with a clone that would fail too.
    let url = clone_url(&args.repo)?;

    if args.clone.unwrap_or(false) {
        return do_clone(&args, url).await;
    }
    if let Some(path) = args.path.clone() {
        return read_file(&args, &path).await;
    }
    Err(invalid(
        "supply `path` to read one file, e.g. {\"repo\": \"tokio-rs/tokio\", \"path\": \"README.md\"}, \
         or `clone: true` to get a local checkout to explore",
    ))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{cache_key, clone_url, github_slug, redact};

    #[test]
    fn credentials_never_reach_the_cache_path() {
        // The token would otherwise become a directory name under ~/.cache.
        assert_eq!(
            cache_key(
                "https://oauth2:glpat-SECRET@gitlab.example.com/g/p.git",
                None
            ),
            "gitlab.example.com--g--p"
        );
        // An authenticated clone and an anonymous one share a cache slot.
        assert_eq!(
            cache_key("https://x-token:SECRET@github.com/a/b.git", None),
            cache_key("https://github.com/a/b.git", None)
        );
    }

    #[test]
    fn credentials_never_reach_an_error_message() {
        assert_eq!(
            redact(
                "fatal: could not read from 'https://oauth2:glpat-SECRET@gitlab.example.com/g/p.git'"
            ),
            "fatal: could not read from 'https://***@gitlab.example.com/g/p.git'"
        );
        // A URL without userinfo, and prose without any URL, pass through intact.
        assert_eq!(
            redact("remote: https://github.com/a/b not found"),
            "remote: https://github.com/a/b not found"
        );
        assert_eq!(
            redact("fatal: repository not found"),
            "fatal: repository not found"
        );
    }

    #[test]
    fn repo_shorthand_and_urls_normalise_to_clone_urls() {
        assert_eq!(
            clone_url("vercel/next.js").unwrap(),
            "https://github.com/vercel/next.js.git"
        );
        assert_eq!(
            clone_url("https://gitlab.com/g/p").unwrap(),
            "https://gitlab.com/g/p"
        );
        assert_eq!(
            clone_url("git@github.com:a/b.git").unwrap(),
            "git@github.com:a/b.git"
        );
        assert!(clone_url("justaword").is_err());
    }

    #[test]
    fn github_is_recognised_across_url_forms_and_others_are_not() {
        assert_eq!(
            github_slug("vercel/next.js").as_deref(),
            Some("vercel/next.js")
        );
        assert_eq!(
            github_slug("https://github.com/vercel/next.js").as_deref(),
            Some("vercel/next.js")
        );
        assert_eq!(
            github_slug("git@github.com:vercel/next.js.git").as_deref(),
            Some("vercel/next.js")
        );
        // A non-GitHub host must not be mistaken for `owner/repo`.
        assert_eq!(github_slug("https://gitlab.com/group/proj"), None);
        assert_eq!(github_slug("nopath"), None);
    }

    #[test]
    fn cache_keys_are_stable_and_collision_free() {
        assert_eq!(
            cache_key("https://github.com/vercel/next.js.git", None),
            "github.com--vercel--next.js"
        );
        // SSH and HTTPS forms of one repo must land in the same directory.
        assert_eq!(
            cache_key("git@github.com:vercel/next.js.git", None),
            cache_key("https://github.com/vercel/next.js.git", None)
        );
        assert_eq!(
            cache_key("https://github.com/a/b", Some("release/2.0")),
            "github.com--a--b--release_2.0"
        );
    }
}
