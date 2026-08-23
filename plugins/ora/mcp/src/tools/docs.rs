use std::fmt::Write as _;

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;

use super::{failed, text};

/// Cloudflare's llms-full.txt is ~46 MB. Fetching a corpus blind is the one
/// failure mode of this tool that costs a whole context window, so anything
/// above this returns the page index instead of the body.
const DEFAULT_MAX_BYTES: usize = 500_000;

const PROBE_PATHS: [&str; 6] = [
    "/llms.txt",
    "/llms-full.txt",
    "/docs/llms.txt",
    "/docs/llms-full.txt",
    "/en/llms.txt",
    "/en/llms-full.txt",
];

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub(crate) struct LibDocsArgs {
    /// Documentation host, e.g. `react.dev` or `https://docs.cloudflare.com`.
    /// Paths vary per site, so this is probed rather than guessed.
    pub(crate) domain: String,
    /// Fetch this exact URL instead of probing. Use it to pull one page listed
    /// in an index returned by a previous call.
    #[serde(default)]
    pub(crate) url: Option<String>,
    /// Byte ceiling on returned content (default 500000).
    #[serde(default)]
    pub(crate) max_bytes: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Index,
    Full,
}

#[derive(Debug)]
struct Hit {
    kind: Kind,
    url: String,
    bytes: Option<u64>,
}

fn host_of(input: &str) -> &str {
    let stripped = input
        .strip_prefix("https://")
        .or_else(|| input.strip_prefix("http://"))
        .unwrap_or(input);
    stripped.split('/').next().unwrap_or(stripped)
}

fn human(bytes: Option<u64>) -> String {
    match bytes {
        None => "?".into(),
        Some(n) if n < 1024 => format!("{n}B"),
        Some(n) if n < 1_048_576 => format!("{}KB", n / 1024),
        #[allow(clippy::cast_precision_loss)]
        Some(n) => format!("{:.1}MB", n as f64 / 1_048_576.0),
    }
}

/// A soft-404 that serves HTML with status 200 is common; content-type is the
/// only reliable way to tell it from a real llms.txt.
async fn probe_one(client: &reqwest::Client, url: String) -> Option<Hit> {
    let resp = client.head(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let ctype = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if ctype.contains("text/html") {
        return None;
    }

    let final_url = resp.url().to_string();
    let kind = if final_url.contains("llms-full.txt") {
        Kind::Full
    } else {
        Kind::Index
    };
    Some(Hit {
        kind,
        url: final_url,
        bytes: resp.content_length(),
    })
}

struct Fetched {
    body: String,
    /// The stream was abandoned at the cap, so the total size stayed unknown.
    overflowed: bool,
}

/// Reads the body chunk by chunk and drops the connection once the cap is
/// reached. Buffering the whole response first would pull Cloudflare's 46 MB
/// corpus into memory just to throw it away, and would still cost the download.
async fn fetch_capped(
    client: &reqwest::Client,
    url: &str,
    max_bytes: usize,
) -> Result<Fetched, McpError> {
    let mut resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| failed(format!("GET {url} failed: {e}")))?;
    if !resp.status().is_success() {
        return Err(failed(format!("GET {url} returned {}", resp.status())));
    }

    let mut buf: Vec<u8> = Vec::new();
    let mut overflowed = false;
    while let Some(chunk) = resp
        .chunk()
        .await
        .map_err(|e| failed(format!("reading {url} failed: {e}")))?
    {
        buf.extend_from_slice(&chunk);
        if buf.len() > max_bytes {
            buf.truncate(max_bytes);
            overflowed = true;
            break;
        }
    }

    Ok(Fetched {
        // Cutting mid-stream can split a multi-byte character; lossy decoding
        // turns that trailing fragment into one replacement char.
        body: String::from_utf8_lossy(&buf).into_owned(),
        overflowed,
    })
}

fn render(got: Fetched, max_bytes: usize) -> String {
    if !got.overflowed {
        return got.body;
    }
    format!(
        "{}\n\n[cut at {max_bytes} bytes; request a narrower page URL rather than raising max_bytes]",
        got.body
    )
}

pub(crate) async fn run(
    client: &reqwest::Client,
    args: LibDocsArgs,
) -> Result<CallToolResult, McpError> {
    let max_bytes = args.max_bytes.unwrap_or(DEFAULT_MAX_BYTES);

    if let Some(url) = args.url {
        return Ok(text(render(
            fetch_capped(client, &url, max_bytes).await?,
            max_bytes,
        )));
    }

    let host = host_of(&args.domain).to_string();
    let mut set = tokio::task::JoinSet::new();
    for path in PROBE_PATHS {
        let (client, url) = (client.clone(), format!("https://{host}{path}"));
        set.spawn(async move { probe_one(&client, url).await });
    }

    let mut hits: Vec<Hit> = Vec::new();
    while let Some(joined) = set.join_next().await {
        if let Ok(Some(hit)) = joined
            && !hits.iter().any(|h| h.url == hit.url)
        {
            hits.push(hit);
        }
    }

    if hits.is_empty() {
        return Ok(text(format!(
            "No llms.txt under https://{host}. Fall back to context7:\n  \
             bunx ctx7@latest library <name> [query]      # resolve the library ID first — IDs aren't guessable\n  \
             bunx ctx7@latest docs <libraryId> \"<query>\"  # narrow queries rank far better than broad ones\n\
             Add --research only if the default answer is shallow; it spawns sandboxed agents and costs more."
        )));
    }

    let full = hits.iter().find(|h| h.kind == Kind::Full);
    let index = hits.iter().find(|h| h.kind == Kind::Index);

    // Plenty of docs hosts answer a HEAD without content-length, so an unknown
    // size is no evidence of a large corpus — only a measured overrun is. The
    // capped fetch is what makes trying safe: it walks away at the ceiling
    // instead of downloading a corpus it would discard.
    let known_oversized = |h: &Hit| {
        h.bytes
            .is_some_and(|n| usize::try_from(n).is_ok_and(|n| n > max_bytes))
    };

    if let Some(f) = full.filter(|f| !known_oversized(f)) {
        let got = fetch_capped(client, &f.url, max_bytes).await?;
        // Whole corpus in one go — the ideal case.
        if !got.overflowed {
            return Ok(text(got.body));
        }
        if index.is_none() {
            return Ok(text(render(got, max_bytes)));
        }
    }

    // Corpus too big: hand back the page index so the next call can request one
    // page by URL.
    if let Some(i) = index {
        let mut out = String::new();
        if let Some(f) = full {
            let size = match f.bytes {
                Some(_) => format!("is {}", human(f.bytes)),
                None => "went".into(),
            };
            let _ = writeln!(
                out,
                "llms-full.txt at {} {size} past the {max_bytes}-byte budget, so it is not \
                 returned whole. Page index below; call this tool again with `url` set to the \
                 page you need.\n",
                f.url
            );
        }
        out.push_str(&render(
            fetch_capped(client, &i.url, max_bytes).await?,
            max_bytes,
        ));
        return Ok(text(out));
    }

    Err(failed("probe found a hit but classified neither kind"))
}

#[cfg(test)]
mod tests {
    use super::{host_of, human};

    #[test]
    fn host_is_extracted_from_any_input_form() {
        assert_eq!(host_of("react.dev"), "react.dev");
        assert_eq!(
            host_of("https://docs.cloudflare.com"),
            "docs.cloudflare.com"
        );
        assert_eq!(host_of("http://a.io/docs/x"), "a.io");
        assert_eq!(host_of("nextjs.org/"), "nextjs.org");
    }

    #[test]
    fn sizes_render_at_the_right_scale() {
        assert_eq!(human(None), "?");
        assert_eq!(human(Some(512)), "512B");
        assert_eq!(human(Some(489_472)), "478KB");
        assert_eq!(human(Some(48_234_496)), "46.0MB");
    }
}
