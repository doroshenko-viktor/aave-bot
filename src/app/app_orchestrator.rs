use crate::{
    accounts::account_store::UserAccountStore,
    blockchain_events::events::BlockchainEvent, tokens::entities::Token,
};
use eyre::Result;
use tokio::sync::mpsc::Receiver;

/**
Handle asset price changes invoking liquidation if price change will affect some account becomes unhealthy
 */
pub async fn spawn_handling_price_changes(
    user_account_store: UserAccountStore,
    mut changed_price_token_rx: Receiver<&'static Token>,
) -> Result<()> {
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
pub async fn start_handling_account_state_changes(
    user_data_store: UserAccountStore,
    mut blockchain_events_rx: Receiver<BlockchainEvent>,
) -> Result<()> {
    while let Some(event) = blockchain_events_rx.recv().await {
        user_data_store.handle_event(event).await;
    }
    // TODO: handle exit from stream - probably panic

    Ok(())
}
