use aave_bot::{
    accounts,
    app::app_orchestrator,
    blockchain_events::{self, events::BlockchainEvent},
    clients,
    configuration::{
        globals::{get_db_pool, get_global_settings},
        persistent_config::{ConfigurationKey, config_get},
    },
    setup::{self, get_new_rpc_provider},
};
use tracing::info;

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    info!("starting bot");
    setup::configure_app().await?;
    let db_pool = get_db_pool();
    let user_account_store = accounts::account_store::restore(db_pool).await?;
    let provider =
        get_new_rpc_provider(&get_global_settings().rpc_config).await?;
    let last_downloaded_block_id: u64 =
        config_get(ConfigurationKey::LastDownloadedBlock)
            .await?
            .parse()?;
    let last_blockchain_block_number =
        clients::blocks::get_current_block(&provider).await?;

    let (aave_events_tx, mut aave_events_rx) =
        tokio::sync::mpsc::channel::<BlockchainEvent>(100);

    let _ = tokio::spawn(async move {
        tokio::task::yield_now().await;
        let provider = get_new_rpc_provider(&get_global_settings().rpc_config)
            .await
            .unwrap();
        blockchain_events::handle_events::get_new_historical_events(
            db_pool,
            &provider,
            &aave_events_tx,
            last_downloaded_block_id,
            last_blockchain_block_number,
        )
        .await
        .unwrap();
    });

    while let Some(event) = aave_events_rx.recv().await {
        drop(event);
    }

    // app_orchestrator::start_handling_account_state_changes(
    //     user_account_store.clone(),
    //     aave_events_rx,
    // )
    // .await
    // .unwrap();

    accounts::account_store::backup(&user_account_store).await?;

    info!("FINISHED HISTORICAL EVENTS CALCULATION");

    // Ok(tokio::signal::ctrl_c().await?)
    Ok(())
}
