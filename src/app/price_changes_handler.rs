use async_trait::async_trait;

use crate::{shared::persistence::AccountAddress, tokens::entities::Token};

#[async_trait]
pub trait PriceChangesHandler {
    async fn handle_price_change(
        &self,
        token: &Token,
    ) -> Vec<&'static AccountAddress>;
}
