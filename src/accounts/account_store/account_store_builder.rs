use crate::accounts::entities::UserAccountRef;
use crate::blockchain::BlockchainDataProviderFacade;
use crate::shared::persistence::AccountAddress;
use dashmap::DashMap;

use super::UserAccountStore;

pub struct AccountStoreBuilder {
    last_block: u64,
    accounts: DashMap<AccountAddress, UserAccountRef>,
}

impl AccountStoreBuilder {
    pub fn new(
        last_block: u64,
        accounts: DashMap<AccountAddress, UserAccountRef>,
    ) -> Self {
        Self { last_block, accounts }
    }

    pub fn build<T>(self, blockchain_facade: T) -> UserAccountStore<T>
    where
        T: BlockchainDataProviderFacade,
    {
        UserAccountStore::new(
            self.accounts,
            self.last_block,
            blockchain_facade,
        )
    }
}
