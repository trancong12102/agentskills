use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};

mod server;
mod tools;

#[tokio::main]
async fn main() -> Result<()> {
    let service = server::Ora::new().serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
