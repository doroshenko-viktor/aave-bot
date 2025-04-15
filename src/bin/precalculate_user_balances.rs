use aave_bot::{
    accounts,
    blockchain_events::{self, events::BlockchainEvent},
    configuration::{self, globals, persistent_config::ConfigurationKey},
    setup,
};
use tracing::error;

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    setup::configure_app().await?;
    let db_pool = globals::get_db_pool();
    let user_account_store = accounts::account_store::restore(db_pool).await?;
    let last_downloaded_block_number =
        configuration::get_as::<u64>(ConfigurationKey::LastDownloadedBlock)
            .await?;

    let (result_tx, mut result_rx) =
        tokio::sync::mpsc::channel::<BlockchainEvent>(100);

    {
        let user_account_store = user_account_store.clone();
        tokio::spawn(async move {
            tokio::task::yield_now().await;
            if let Err(err) = blockchain_events::handle_events::handle_downloaded_historical_events(
            db_pool,
            &result_tx,
            user_account_store.get_last_block_number() + 1,
            last_downloaded_block_number,
        )
        .await {
            error!("Error downloading historical events: {}", err);
        }
        });
    }

    while let Some(event) = result_rx.recv().await {
        let _ = user_account_store.handle_event(event).await;
    }

    accounts::account_store::backup(&user_account_store).await?;

    Ok(())
}
