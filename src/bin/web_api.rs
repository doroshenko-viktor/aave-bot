use aave_bot::{setup, web_api};
use eyre::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    setup::configure_app().await?;
    info!("App configured");
    web_api::setup_web_app().await?;

    Ok(())
}
