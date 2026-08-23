use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
};
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_handler, tool_router};

use crate::tools::{ast, docs, pkg, repo};

#[derive(Clone, Debug)]
pub(crate) struct Ora {
    http: reqwest::Client,
    // Read by the code `#[tool_handler]` generates, which dead-code analysis misses.
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl Ora {
    pub(crate) fn new() -> Self {
        let http = reqwest::Client::builder()
            .user_agent(concat!("ora-mcp/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            http,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Search code by AST structure via ast-grep. Use when the target is defined by \
                       code shape rather than text — async functions containing await, calls with a \
                       specific argument shape, a node nested inside another. Returns file:line:col \
                       plus metavariable bindings. Prefer plain grep for literal text."
    )]
    async fn ast_search(
        &self,
        Parameters(args): Parameters<ast::AstSearchArgs>,
    ) -> Result<CallToolResult, McpError> {
        ast::search(args).await
    }

    #[tool(
        description = "Outline a file's or directory's symbols, exports and imports without reading \
                       it. Use to map structure before opening large files, or to see a module's \
                       public surface. Syntax-only, no indexing step."
    )]
    async fn outline(
        &self,
        Parameters(args): Parameters<ast::OutlineArgs>,
    ) -> Result<CallToolResult, McpError> {
        ast::outline(args).await
    }

    #[tool(
        description = "Look up the latest stable version, publish date and deprecation status of \
                       public packages via deps.dev. Use before adding or bumping a dependency, and \
                       to check whether something is deprecated. Batch all packages into one call. \
                       Public registries only."
    )]
    async fn pkg_versions(
        &self,
        Parameters(args): Parameters<pkg::PkgVersionsArgs>,
    ) -> Result<CallToolResult, McpError> {
        pkg::run(&self.http, args).await
    }

    #[tool(
        description = "Fetch library documentation from the author-published llms.txt of a docs \
                       site, falling back to a page index when the corpus is too large to read \
                       whole. Prefer this over search engines for API references, changelogs and \
                       breaking changes: it is written by the library authors, so there is no \
                       curation lag. Not for local code."
    )]
    async fn lib_docs(
        &self,
        Parameters(args): Parameters<docs::LibDocsArgs>,
    ) -> Result<CallToolResult, McpError> {
        docs::run(&self.http, args).await
    }

    #[tool(
        description = "Read a file from a public repository, or shallow-clone one into a local \
                       cache for a deeper dive. Use instead of fetching github.com or \
                       raw.githubusercontent.com URLs. Reading a single file is GitHub-only; \
                       cloning works with any git host."
    )]
    async fn repo_fetch(
        &self,
        Parameters(args): Parameters<repo::RepoFetchArgs>,
    ) -> Result<CallToolResult, McpError> {
        repo::run(args).await
    }
}

impl Default for Ora {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_handler]
// The macro emits `async fn`s that never await; nothing we control.
#[allow(clippy::unused_async_trait_impl)]
impl ServerHandler for Ora {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::from_build_env())
            .with_protocol_version(ProtocolVersion::V_2025_06_18)
    }
}
