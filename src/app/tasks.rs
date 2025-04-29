use crate::accounts::{self, UserAccountStore};
use crate::blockchain::BlockchainDataProviderFacade;
use crate::{
    blockchain_events::events::BlockchainEvent, tokens::entities::Token,
};
use eyre::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;

/**
Handle asset price changes invoking liquidation if price change will affect some account becomes unhealthy
 */
pub async fn spawn_handling_price_changes<T>(
    user_account_store: Arc<UserAccountStore<T>>,
    mut changed_price_token_rx: Receiver<&'static Token>,
) -> Result<()>
where
    T: BlockchainDataProviderFacade + Send + Sync + 'static,
{
    tokio::spawn(async move {
        while let Some(token) = changed_price_token_rx.recv().await {
            let accounts_to_liquidate = user_account_store
                .get_accounts_to_liquidate_on_price_change(token.get_id())
                .await;
            // TODO: call liquidation functionality
        }
    });

    Ok(())
}

/**
Handle blockchain events from AAVE maintaining account balances.
*/
pub async fn start_handling_account_state_changes<T>(
    user_data_store: Arc<UserAccountStore<T>>,
    mut blockchain_events_rx: Receiver<BlockchainEvent>,
) -> Result<()>
where
    T: BlockchainDataProviderFacade + Send + Sync + 'static,
{
    while let Some(event) = blockchain_events_rx.recv().await {
        user_data_store.handle_event(event).await;
    }
    // TODO: handle exit from stream - probably panic

    Ok(())
}

/**
Spawns a separate process to periodically back up collected account data.

Note: Account data is not saved to the database on each event arrival to optimize performance. Instead, it is collected over a period of time and then saved along with the current block ID at the moment of saving. This allows for data restoration after an application restart and ensures that event handling can resume from the last saved block number.

This approach will not lead to improper account values even if the application is stopped some time after the last backup. On the next run, the account store will be recalculated starting from the stored block ID, and any data not included in the backup will be retrieved from the blockchain again.
*/
pub fn spawn_user_accounts_store_backup_task<T>(
    user_data_store: Arc<UserAccountStore<T>>,
    period: Duration,
) where
    T: BlockchainDataProviderFacade + Send + Sync + 'static,
{
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(period).await;
            accounts::backup_user_data_store(&user_data_store).await;
        }
    });
}

/**
Spawns a background task to periodically check the health factor of current accounts. Accounts found to be unhealthy are added to a shortlist of liquidation candidates.

__Note__:

While continuously checking the health factor on every AAVE and price event would provide the most up-to-date list of accounts with low health factors, it would also lead to excessive calculations. Rechecking the health factor at intervals of several seconds offers a sufficient balance between responsiveness and computational efficiency.

On each new price change, we only examine this shortlist. It is improbable that accounts not currently on the shortlist would become unhealthy faster than this background task can re-evaluate their health in its next cycle.
*/
pub fn spawn_low_hf_accounts_refresh_task<T>(
    user_account_store: Arc<UserAccountStore<T>>,
    period: Duration,
) where
    T: BlockchainDataProviderFacade + Send + Sync + 'static,
{
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(period).await;
            user_account_store.refresh_low_hf_accounts();
        }
    });
}
