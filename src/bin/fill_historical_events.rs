use aave_bot::{
    blockchain_events::{self},
    clients,
    configuration::{
        globals::{get_db_pool, get_global_settings},
        persistent_config::{ConfigurationKey, config_get},
    },
    setup::{self},
};
use tracing::info;

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    info!("starting bot");
    setup::configure_app().await?;
    let db_pool = get_db_pool();
    let provider =
        setup::get_new_rpc_provider(&get_global_settings().rpc_config)?;
    let last_downloaded_block_id: u64 =
        config_get(ConfigurationKey::LastDownloadedAaveEventsBlock)
            .await?
            .parse()?;
    let last_blockchain_block_number =
        clients::blocks::get_current_block(&provider).await?;
    let last_downloaded_block_id = 16496817;

    blockchain_events::handle_events::get_historical_aave_events(
        db_pool,
        &provider,
        last_downloaded_block_id,
        last_blockchain_block_number,
    )
    .await?;

    info!("FINISHED HISTORICAL EVENTS CALCULATION");

    Ok(())
}
