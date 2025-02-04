use eyre::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::data_store::UserDataStore;

pub async fn backup_user_data_store(data_store: Arc<RwLock<UserDataStore>>) -> Result<()> {
    // TODO: implement

    // let last_handled_block_number: u64 = config_get(ConfigurationKey::LastUserBalancesBlock)
    //     .await?
    //     .parse()?;
    Ok(())
}
