use aave_bot::{
    blockchain_events::handle_events,
    clients,
    configuration::{
        config_get,
        globals::{get_db_pool, get_global_settings},
        persistent_config::ConfigurationKey,
    },
    setup,
};
use eyre::Result;

#[tokio::main]
pub async fn main() -> Result<()> {
    setup::configure_app().await?;
    let db_pool = get_db_pool();
    let provider =
        setup::get_new_rpc_provider(&get_global_settings().rpc_config)?;
    let last_downloaded_block_id: u64 =
        config_get(ConfigurationKey::LastDownloadedTransfersBlock)
            .await?
            .parse()?;
    let last_blockchain_block_number =
        clients::blocks::get_current_block(&provider).await?;
    handle_events::get_historical_asset_transfers(
        db_pool,
        &provider,
        last_downloaded_block_id,
        last_blockchain_block_number,
    )
    .await?;

    Ok(())
}
