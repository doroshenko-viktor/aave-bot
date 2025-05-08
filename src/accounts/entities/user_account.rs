use crate::{
    accounts::types::AssetAmount,
    configuration::globals,
    operational::aave_events::AaveAccountMovement,
    price_feed::prices_aggregator,
    shared::persistence::AccountAddress,
    tokens::{entities::Token, token_service, types::TokenId},
};
use alloy::primitives::Uint;
use eyre::{Result, eyre};
use hashbrown::{HashMap, hash_map::Entry};
use std::{collections::HashSet, sync::Arc};
use tracing::{debug, error, info, warn};

const PRECISION: u32 = 100;

pub type UserAccountRef = Arc<tokio::sync::RwLock<UserAccount>>;

#[derive(Debug, Clone, Copy)]
pub struct Collateral {
    pub amount: AssetAmount,
    pub a_token_amount: AssetAmount,
}

impl Collateral {
    pub fn new(amount: AssetAmount, a_token_amount: AssetAmount) -> Self {
        Self { amount, a_token_amount }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Debt {
    pub amount: AssetAmount,
    pub debt_token_amount: AssetAmount,
}

impl Debt {
    pub fn new(amount: AssetAmount, debt_token_amount: AssetAmount) -> Self {
        Self { amount, debt_token_amount }
    }
}

#[derive(Debug)]
pub struct UserAccount {
    /* Account address on blockchain */
    pub(crate) address: AccountAddress,
    // TODO: change to be EnumMap for better CPU caching
    // or [Option<_>; N] where N is amount of tokens configured and Some for asset present in the
    // system
    // pub(crate) collaterals: [Option<AssetAmount>; 32],
    // pub(crate) debts: [Option<AssetAmount>; 32],
    pub(crate) collaterals: HashMap<TokenId, Collateral>,
    pub(crate) debts: HashMap<TokenId, Debt>,
}

impl UserAccount {
    pub fn new_with_address(address: AccountAddress) -> Self {
        Self {
            address,
            collaterals: HashMap::new(),
            debts: HashMap::new(),
        }
    }

    pub fn new(
        address: AccountAddress,
        collaterals: HashMap<TokenId, Collateral>,
        debts: HashMap<TokenId, Debt>,
    ) -> Self {
        Self { address, collaterals, debts }
    }

    pub fn get_address(&self) -> AccountAddress {
        self.address
    }

    // TODO: maybe maintain set of related tokens on account and change it on blockchain events, because it happens not as often as price change events to reduce compute time
    pub fn get_related_tokens(&self) -> HashSet<TokenId> {
        self.collaterals
            .iter()
            .map(|(token_id, _)| *token_id)
            .chain(self.debts.iter().map(|(token_id, _)| *token_id))
            .collect()
    }

    /**
     * Health Factor= ∑(Collateral Amount×Collateral Price×Liquidation Threshold)/ ∑(Borrowed Amount×Borrowed Asset Price)
     */
    // TODO: this calculation uses thresold for both collaterals and debts, but should only for
    // collaterals
    pub async fn calculate_health_factor(&self) -> Result<f32> {
        Ok(self
            .get_asset_sum(
                self.collaterals.iter().map(|(token_id, collateral)| {
                    (*token_id, collateral.amount)
                }),
            )
            .await?
            / self
                .get_asset_sum(
                    self.debts
                        .iter()
                        .map(|(token_id, debt)| (*token_id, debt.amount)),
                )
                .await?)
    }

    async fn get_asset_sum(
        &self,
        list: impl Iterator<Item = (TokenId, AssetAmount)>,
    ) -> Result<f32> {
        let asset_rules = globals::get_asset_rules();
        let health_factor = list
            .filter_map(|(token_id, asset_amount)| {
                let token = token_service::get_token_by_id(token_id);
                let threshold =
                    asset_rules.get_asset_liquidation_threshold(token_id);
                let Some(price) = prices_aggregator::get_usd_price(token_id)
                else {
                    warn!(
                        "Price for token {} not found",
                        token.get_symbol()
                    );

                    return None;
                };

                // TODO:need to divide or asset amount or price for decimals because price is not
                // for wei
                Some((asset_amount, price, threshold))
            })
            .fold(
                0f32,
                |mut res, (asset_amount, price, threshold)| {
                    // TODO: make proper calculations with tests
                    // res += price * asset_amount * threshold;
                    res
                },
            );

        Ok(health_factor)
    }

    pub fn get_collateral_list(
        &self,
    ) -> impl Iterator<Item = (TokenId, AssetAmount)> {
        self.collaterals
            .iter()
            .map(|(token_id, collateral)| (*token_id, collateral.amount))
    }

    pub fn get_debt_list(
        &self,
    ) -> impl Iterator<Item = (TokenId, AssetAmount)> {
        self.debts
            .iter()
            .map(|(token_id, debt)| (*token_id, debt.amount)) // TODO: check if amount vs a_tokenamount 
    }

    pub fn supply(&mut self, token: &Token, data: &AaveAccountMovement) {
        self.collaterals
            .entry(token.id)
            .and_modify(|val| {
                info!(
                    "Existing collateral amount {}",
                    val.amount
                );
                val.amount += &data.native_token_amount;
                val.a_token_amount += &data.reserve_token_amount;
                info!(
                    "Updated collateral amount {}",
                    val.amount
                );
            })
            .or_insert_with(|| {
                debug!(
                    "Adding collateral with init amount {}",
                    data.native_token_amount
                );
                Collateral::new(
                    data.native_token_amount,
                    data.reserve_token_amount,
                )
            });
    }

    pub fn borrow(
        &mut self,
        token: &Token,
        event_body: &AaveAccountMovement,
    ) -> Result<()> {
        #[cfg(debug_assertions)]
        {
            if self.collaterals.is_empty() {
                info!("{:#?}", self.collaterals);
                return Err(eyre!(
                    "Borrowing {} {} for address {} with no collateral",
                    token.get_repr(event_body.native_token_amount)?,
                    token.get_symbol(),
                    self.address
                ));
            }
        }
        self.debts
            .entry(token.id)
            .and_modify(|val| {
                info!(
                    "Initial debt amount {}",
                    token.get_repr(val.amount).unwrap()
                );
                val.amount += &event_body.native_token_amount;
                val.debt_token_amount += &event_body.reserve_token_amount;
                info!(
                    "Updated debt amount {}",
                    token.get_repr(val.amount).unwrap()
                );
            })
            .or_insert_with(|| {
                debug!(
                    "Adding debt with init amount {}",
                    token.get_repr(event_body.native_token_amount).unwrap()
                );
                Debt::new(
                    event_body.native_token_amount,
                    event_body.reserve_token_amount,
                )
            });

        Ok(())
    }

    pub fn withdraw(
        &mut self,
        token: &Token,
        event_data: &AaveAccountMovement,
    ) -> Result<()> {
        match self.collaterals.entry(token.id) {
            Entry::Occupied(mut val) => {
                let val = val.get_mut();
                // TODO: check if amount is not more than available
                info!(
                    "Initial collateral reserve token amount {}",
                    token.get_repr(val.a_token_amount)?
                );
                if val.amount < event_data.native_token_amount {
                    val.amount = Uint::from(0);
                } else {
                    val.amount -= &event_data.native_token_amount;
                }

                let precision = Uint::from(PRECISION); // TODO: optimization-move to constants...
                if val.a_token_amount
                    < event_data.reserve_token_amount - precision
                {
                    return Err(eyre::eyre!(
                        "Withdrawal of {} reserve token is less than balance({})",
                        event_data.reserve_token_amount,
                        val.a_token_amount
                    ));
                } else {
                    val.a_token_amount -= &event_data.reserve_token_amount;
                }
                info!(
                    "Withdraw updated a-token-amount: {}; collateral amount {}",
                    token.get_repr(val.a_token_amount)?,
                    token.get_repr(val.amount)?
                );
                Ok(())
            }
            Entry::Vacant(_) => Err(eyre!(
                "Collateral for token_id '{}' not found at account '{}'",
                token.id,
                self.address
            )),
        }
    }

    pub fn repay(
        &mut self,
        token: &Token,
        event_body: &AaveAccountMovement,
    ) -> Result<()> {
        let asset = if let Some(asset) = self.debts.get_mut(&token.id) {
            asset
        } else {
            return Err(eyre!(
                "Not found debt asset to be repaid for address: {}",
                self.address
            ));
        };

        info!(
            "Initial debt_token_amount: {}; debt amount {}",
            token.get_repr(asset.debt_token_amount)?,
            token.get_repr(asset.amount)?
        );
        let precision = Uint::from(PRECISION); // TODO: optimization-move to constants...
        if asset.debt_token_amount < event_body.reserve_token_amount - precision
        {
            return Err(eyre!(
                "Debt amount {} is less than {}",
                token.get_repr(asset.debt_token_amount)?,
                token.get_repr(event_body.reserve_token_amount)?
            ));
        }
        asset.debt_token_amount -= event_body.reserve_token_amount;

        if asset.amount < event_body.native_token_amount {
            asset.amount = Uint::from(0);
        } else {
            asset.amount -= event_body.native_token_amount;
        }

        info!(
            "Updated debt_token_amount: {}; debt amount {}",
            token.get_repr(asset.debt_token_amount)?,
            token.get_repr(asset.amount)?
        );

        Ok(())
    }

    pub fn liquidate(
        &mut self,
        liqudation_repay_debt: &AaveAccountMovement,
        liquidation_withdraw_collateral: &AaveAccountMovement,
    ) -> Result<()> {
        let debt = if let Some(debt) =
            self.debts.get_mut(&liqudation_repay_debt.token.id)
        {
            debt
        } else {
            return Err(eyre!(
                "Not found debt asset to be repaid by liquidation"
            ));
        };

        let collateral = if let Some(collateral) = self
            .collaterals
            .get_mut(&liquidation_withdraw_collateral.token.id)
        {
            collateral
        } else {
            return Err(eyre!(
                "Not found collateral asset to be liquidated"
            ));
        };

        if debt.debt_token_amount < liqudation_repay_debt.reserve_token_amount {
            return Err(eyre!(
                "Debt amount {} is less than {}",
                debt.debt_token_amount,
                liqudation_repay_debt.reserve_token_amount
            ));
        }
        if collateral.a_token_amount
            < liquidation_withdraw_collateral.reserve_token_amount
        {
            return Err(eyre!(
                "Collateral amount {} is less than {}",
                collateral.a_token_amount,
                liquidation_withdraw_collateral.reserve_token_amount
            ));
        }

        debt.amount -= liqudation_repay_debt.native_token_amount;
        debt.debt_token_amount -= liqudation_repay_debt.reserve_token_amount;
        collateral.amount -=
            liquidation_withdraw_collateral.native_token_amount;
        collateral.a_token_amount -=
            liquidation_withdraw_collateral.reserve_token_amount;

        Ok(())
    }

    pub fn handle_transfer_out(
        &mut self,
        event: &AaveAccountMovement,
    ) -> Result<()> {
        match self.collaterals.entry(event.token.id) {
            Entry::Occupied(mut entry) => {
                let val = entry.get_mut();
                let token = event.token;
                info!(
                    "INITIAL a-token-amount {} {}; subtracted a-token-amount {}",
                    event.token.get_repr(val.a_token_amount)?,
                    token.get_symbol(),
                    event.get_reserve_normalized_amount()?
                );
                if val.a_token_amount < event.reserve_token_amount {
                    return Err(eyre::eyre!(
                        "Collateral of {} for account '{}' is not enough",
                        token,
                        self.address
                    ));
                }

                val.a_token_amount -= event.reserve_token_amount;
                if val.amount < event.native_token_amount {
                    val.amount = Uint::from(0);
                } else {
                    val.amount -= event.native_token_amount;
                }
                info!(
                    "Updated collateral a-token-amount: {} amount {}",
                    token.get_repr(val.a_token_amount)?,
                    token.get_repr(val.amount)?
                );
                Ok(())
            }
            Entry::Vacant(_) => Err(eyre::eyre!(
                "Collateral of token: {} for account '{:#x}' not found",
                event.token.get_symbol(),
                self.address
            )),
        }
    }

    pub fn handle_transfer_in(
        &mut self,
        event: &AaveAccountMovement,
    ) -> Result<()> {
        match self.collaterals.entry(event.token.id) {
            Entry::Occupied(mut entry) => {
                let val = entry.get_mut();
                val.amount += event.native_token_amount;
                val.a_token_amount += event.reserve_token_amount;
                info!(
                    "Updated collateral amount {} {}",
                    event.token.get_repr(val.amount)?,
                    self.address,
                );
            }
            Entry::Vacant(entry) => {
                warn!(
                    "Collateral with token_id: {} for account '{}' not found. Creating new account.",
                    event.token.id, self.address
                );
                entry.insert(Collateral::new(
                    event.native_token_amount,
                    event.reserve_token_amount,
                ));
            }
        };

        Ok(())
    }

    pub fn address(&self) -> AccountAddress {
        self.address
    }
}

impl std::hash::Hash for UserAccount {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.address.hash(state);
    }
}

impl PartialEq for UserAccount {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}
