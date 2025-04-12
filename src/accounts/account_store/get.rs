use std::str::FromStr;
use std::sync::Arc;

use super::UserAccountStore;
use crate::accounts::entities::{UserAccount, UserAccountRef};
use crate::accounts::{AccountCollateralDto, AccountDebtDto};
use crate::configuration::persistent_config::{ConfigurationKey, config_get};
use crate::shared::persistence::{AccountAddress, DbPool};
use crate::tokens::token_service;
use crate::utils::uint_ext::BigDecimalExt;
use alloy::primitives::Address;
use dashmap::DashMap;
use eyre::Result;
use hashbrown::HashMap;
use tokio::sync::RwLock;

pub async fn get_accounts(pool: &DbPool) -> Result<Vec<AccountAddress>> {
    Ok(sqlx::query!(
        r#"
        select 
            address
        from operational.accounts
    "#
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|record| {
        Address::from_str(&record.address)
            .expect("Expected to get a valid account address from db")
    })
    .collect())
}

pub async fn get_collaterals(
    pool: &DbPool,
) -> Result<Vec<AccountCollateralDto>> {
    let collaterals = sqlx::query_as!(
        AccountCollateralDto,
        r#"
        select
            account_address,
            token_symbol,
            amount
        from operational.account_collaterals
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(collaterals)
}

pub async fn get_debts(pool: &DbPool) -> Result<Vec<AccountDebtDto>> {
    let debts = sqlx::query_as!(
        AccountDebtDto,
        r#"
        select 
            account_address, 
            token_symbol, 
            amount 
        from operational.account_debts
    "#
    )
    .fetch_all(pool)
    .await?;

    Ok(debts)
}

pub async fn restore_user_account_store(
    pool: &DbPool,
) -> Result<UserAccountStore> {
    let mut collaterals_by_account_address =
        get_collaterals(pool).await?.into_iter().fold(
            HashMap::<AccountAddress, Vec<AccountCollateralDto>>::new(),
            |mut res, cur| {
                let address = Address::from_str(&cur.account_address).expect(
                    "Expected to receive a valid account address from db",
                );
                res.entry(address)
                    .and_modify(|accounts| {
                        accounts.push(cur);
                    })
                    .or_insert_with(|| Vec::new());
                res
            },
        );
    let mut debts_by_account_address = get_debts(pool).await?.into_iter().fold(
        HashMap::<AccountAddress, Vec<AccountDebtDto>>::new(),
        |mut res, cur| {
            let address = Address::from_str(&cur.account_address)
                .expect("Expected to receive a valid account address from db");
            res.entry(address)
                .and_modify(|accounts| {
                    accounts.push(cur);
                })
                .or_insert_with(|| Vec::new());
            res
        },
    );
    let accounts = get_accounts(pool).await?;

    let accounts: DashMap<AccountAddress, UserAccountRef> =
        accounts.into_iter().fold(
            DashMap::new(),
            |res, account_address| {
                let collaterals = collaterals_by_account_address
                    .remove(&account_address)
                    .unwrap_or_else(|| Vec::new())
                    .into_iter()
                    .fold(HashMap::new(), |mut res, cur| {
                        let token = token_service::get_token_by_symbol(
                            &cur.token_symbol,
                        );
                        res.insert(
                            token.get_id(),
                            cur.amount
                                .to_uint()
                                .expect("Failed to convert amount to uint"),
                        );
                        res
                    });
                let debts = debts_by_account_address
                    .remove(&account_address)
                    .unwrap_or_else(|| Vec::new())
                    .into_iter()
                    .fold(HashMap::new(), |mut res, cur| {
                        let token = token_service::get_token_by_symbol(
                            &cur.token_symbol,
                        );
                        res.insert(
                            token.get_id(),
                            cur.amount
                                .to_uint()
                                .expect("Failed to convert amount to uint"),
                        );
                        res
                    });

                res.insert(
                    account_address.clone(),
                    Arc::new(RwLock::new(UserAccount::new(
                        account_address,
                        collaterals,
                        debts,
                    ))),
                );

                res
            },
        );

    let last_user_balances_block_number: u64 =
        config_get(ConfigurationKey::LastUserBalancesBlock)
            .await?
            .parse()?;

    Ok(UserAccountStore::new(
        Arc::new(accounts),
        last_user_balances_block_number,
    ))
}
