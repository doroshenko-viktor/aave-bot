use std::{ops::Deref, sync::Arc};

use crate::{
    accounts::{
        AccountCollateralDto, AccountDebtDto, AccountInsertDto,
        entities::{UserAccount, UserAccountRef},
    },
    configuration::globals::get_db_pool,
    shared::persistence::{DbPool, get_placeholders_str},
};
use eyre::Result;
use futures::{StreamExt, future::join_all};
use sqlx::{Postgres, Transaction};
use tracing::{debug, error};

use super::data_store::UserAccountStore;

pub async fn backup_user_data_store<'a>(user_data_store: &'a UserAccountStore) -> Result<()> {
    let pool: &DbPool = get_db_pool();

    let (accounts, collaterals, debts) =
        join_all(user_data_store.accounts.iter().map(async |kvp| {
            let account = kvp.value().read().await;
            let account_insert_dto = AccountInsertDto {
                address: account.get_address(),
            };
            let collaterals: Vec<AccountCollateralDto> = account
                .collaterals
                .iter()
                .map(|(token, asset)| AccountCollateralDto {
                    account_address: account.address.to_checksum(None),
                    token_symbol: token.get_symbol(),
                    amount: asset.amount.clone(),
                })
                .collect();
            let debts: Vec<AccountDebtDto> = account
                .debts
                .iter()
                .map(|(token, asset)| AccountDebtDto {
                    account_address: account.address.to_checksum(None),
                    token_symbol: token.get_symbol(),
                    amount: asset.amount.clone(),
                })
                .collect();
            (account_insert_dto, collaterals, debts)
        }))
        .await
        .into_iter()
        .fold(
            (
                Vec::<AccountInsertDto>::new(),
                Vec::<AccountCollateralDto>::new(),
                Vec::<AccountDebtDto>::new(),
            ),
            |mut res, curr| {
                res.0.push(curr.0);
                res.1.extend(curr.1);
                res.2.extend(curr.2);
                res
            },
        );
    // let (accounts, collaterals, debts) =
    // let x = futures::stream::iter(user_data_store.accounts.iter().map(async |account| {
    //     let account = account.read().await;
    //     let account_insert_dto = AccountInsertDto {
    //         address: account.get_address(),
    //     };
    //     let collaterals: Vec<AccountCollateralDto> = account
    //         .collaterals
    //         .iter()
    //         .map(|(token, asset)| AccountCollateralDto {
    //             account_address: account.address.to_checksum(None),
    //             token_symbol: token.get_symbol(),
    //             amount: asset.amount.clone(),
    //         })
    //         .collect();
    //     let debts: Vec<AccountDebtDto> = account
    //         .debts
    //         .iter()
    //         .map(|(token, asset)| AccountDebtDto {
    //             account_address: account.address.to_checksum(None),
    //             token_symbol: token.get_symbol(),
    //             amount: asset.amount.clone(),
    //         })
    //         .collect();
    //     (account_insert_dto, collaterals, debts)
    // }))
    // .buffer_unordered(8)
    // .collect()
    // .await;

    // .fold(
    //     (Vec::new(), Vec::new(), Vec::new()),
    //     async |mut res, curr| {
    //         res.0.push(curr.0);
    //         res.1.extend(curr.1);
    //         res.2.extend(curr.2);
    //         res
    //     },
    // )
    // .await;

    if let Err(e) =
        save_account_transaction(pool.begin().await?, accounts, collaterals, debts).await
    {
        error!("Error during user data store backup transaction: {}", e);
    };

    Ok(())
}

async fn save_account_transaction(
    mut tx: Transaction<'_, Postgres>,
    accounts: Vec<AccountInsertDto>,
    collaterals: Vec<AccountCollateralDto>,
    debts: Vec<AccountDebtDto>,
) -> Result<()> {
    save_accounts(&mut tx, accounts).await?;
    save_collaterals(&mut tx, collaterals).await?;
    save_debts(&mut tx, debts).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn save_accounts(
    tx: &mut Transaction<'_, Postgres>,
    accounts: Vec<AccountInsertDto>,
) -> Result<()> {
    debug!("Saving {} accounts", accounts.len());
    let mut query_builder = r#"
        INSERT INTO operational.accounts (
            address
        ) VALUES "#
        .to_string();

    let placeholders = get_placeholders_str(1, accounts.len());
    query_builder.push_str(&placeholders);
    query_builder.push_str(
        r#"
        ON CONFLICT (address) DO UPDATE SET address = excluded.address"#,
    );
    let mut query = sqlx::query(&query_builder);

    for event in accounts {
        query = query.bind(event.address.to_checksum(None));
    }
    let res = query.execute(tx.as_mut()).await?;

    Ok(())
}

pub async fn save_collaterals(
    tx: &mut Transaction<'_, Postgres>,
    collaterals: Vec<AccountCollateralDto>,
) -> Result<()> {
    let mut query_builder = r#"
        INSERT INTO operational.account_collaterals (
            account_address,
            token_symbol,
            amount
        ) VALUES "#
        .to_string();

    let placeholders = get_placeholders_str(3, collaterals.len());
    query_builder.push_str(&placeholders);
    query_builder.push_str(
        r#"
        ON CONFLICT (account_address, token_symbol) 
        DO UPDATE SET amount = excluded.amount"#,
    );
    let mut query = sqlx::query(&query_builder);

    for collateral in collaterals {
        query = query
            .bind(collateral.account_address)
            .bind(collateral.token_symbol)
            .bind(collateral.amount);
    }
    let _ = query.execute(tx.as_mut()).await?;

    Ok(())
}

pub async fn save_debts(
    tx: &mut Transaction<'_, Postgres>,
    debts: Vec<AccountDebtDto>,
) -> Result<()> {
    let mut query_builder = r#"
        INSERT INTO operational.account_debts (
            account_address,
            token_symbol,
            amount
        ) VALUES "#
        .to_string();

    let placeholders = get_placeholders_str(3, debts.len());
    query_builder.push_str(&placeholders);
    query_builder.push_str(
        r#"
        ON CONFLICT (account_address, token_symbol) 
        DO UPDATE SET amount = excluded.amount"#,
    );
    let mut query = sqlx::query(&query_builder);

    for debt in debts {
        query = query
            .bind(debt.account_address)
            .bind(debt.token_symbol)
            .bind(debt.amount);
    }
    let _ = query.execute(tx.as_mut()).await?;

    Ok(())
}
