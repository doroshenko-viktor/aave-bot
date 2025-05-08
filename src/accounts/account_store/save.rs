use crate::{
    accounts::{AccountCollateralDto, AccountDebtDto, AccountInsertDto},
    blockchain::BlockchainDataProviderFacade,
    configuration::{
        config_set, globals::get_db_pool, persistent_config::ConfigurationKey,
    },
    shared::persistence::{DbPool, get_placeholders_str},
    tokens::token_service,
    utils::uint_ext::UintExt,
};
use eyre::Result;
use futures::future::join_all;
use sqlx::{Postgres, Transaction};
use tracing::{debug, error, info, warn};

use super::UserAccountStore;

pub async fn backup_user_data_store<'a>(
    user_account_store: &'a UserAccountStore,
) -> Result<()> {
    info!("Backing up user data store...");
    let pool: &DbPool = get_db_pool();

    let (accounts, collaterals, debts) = join_all(
        user_account_store.accounts.iter().map(async |kvp| {
            let account = kvp.value().read().await;
            let account_insert_dto =
                AccountInsertDto { address: account.get_address() };
            let collaterals: Vec<AccountCollateralDto> = account
                .collaterals
                .iter()
                .map(|(token_id, collateral)| {
                    let token = token_service::get_token_by_id(*token_id);
                    AccountCollateralDto {
                        account_address: account.address.to_checksum(None),
                        token_symbol: token.get_symbol(),
                        amount: collateral.amount.to_decimal(),
                        a_token_amount: collateral.a_token_amount.to_decimal(),
                    }
                })
                .collect();

            let debts: Vec<AccountDebtDto> = account
                .debts
                .iter()
                .map(|(token_id, debt)| {
                    let token = token_service::get_token_by_id(*token_id);
                    AccountDebtDto {
                        account_address: account.address.to_checksum(None),
                        token_symbol: token.get_symbol(),
                        amount: debt.amount.to_decimal(),
                        debt_token_amount: debt.debt_token_amount.to_decimal(),
                    }
                })
                .collect();
            (account_insert_dto, collaterals, debts)
        }),
    )
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

    if let Err(e) = save_account_transaction(
        pool.begin().await?,
        accounts,
        collaterals,
        debts,
    )
    .await
    {
        error!(
            "Error during user data store backup transaction: {}",
            e
        );
    };
    info!("Saving last user balances block...");
    config_set(
        ConfigurationKey::LastUserBalancesBlock,
        &user_account_store.get_last_block_number().to_string(),
    )
    .await?;

    Ok(())
}

async fn save_account_transaction(
    mut tx: Transaction<'_, Postgres>,
    accounts: Vec<AccountInsertDto>,
    collaterals: Vec<AccountCollateralDto>,
    debts: Vec<AccountDebtDto>,
) -> Result<()> {
    if !accounts.is_empty() {
        save_accounts(&mut tx, accounts).await?;
    }
    if !collaterals.is_empty() {
        save_collaterals(&mut tx, collaterals).await?;
    }
    if !debts.is_empty() {
        save_debts(&mut tx, debts).await?;
    }
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
            amount,
            a_token_amount
        ) VALUES "#
        .to_string();

    let placeholders = get_placeholders_str(4, collaterals.len());
    query_builder.push_str(&placeholders);
    query_builder.push_str(
        r#"
        ON CONFLICT (account_address, token_symbol) 
        DO UPDATE SET 
            amount = excluded.amount,
            a_token_amount = excluded.a_token_amount
        "#,
    );
    let mut query = sqlx::query(&query_builder);

    for collateral in collaterals {
        query = query
            .bind(collateral.account_address)
            .bind(collateral.token_symbol)
            .bind(collateral.amount)
            .bind(collateral.a_token_amount);
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
            amount,
            debt_token_amount
        ) VALUES "#
        .to_string();

    let placeholders = get_placeholders_str(4, debts.len());
    query_builder.push_str(&placeholders);
    query_builder.push_str(
        r#"
        ON CONFLICT (account_address, token_symbol) 
        DO UPDATE SET 
            amount = excluded.amount,
            debt_token_amount = excluded.debt_token_amount
        "#,
    );
    let mut query = sqlx::query(&query_builder);

    for debt in debts {
        query = query
            .bind(debt.account_address)
            .bind(debt.token_symbol)
            .bind(debt.amount)
            .bind(debt.debt_token_amount);
    }
    let _ = query.execute(tx.as_mut()).await?;

    Ok(())
}
