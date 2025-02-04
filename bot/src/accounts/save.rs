use crate::{
    app::entities::AccountEntity,
    shared::persistence::{get_placeholders_str, DbPool},
};
use eyre::Result;
use tracing::debug;

use super::{AccountCollateralDto, AccountDebtDto, AccountInsertDto};

pub async fn save_accounts(
    pool: &DbPool,
    accounts: Vec<AccountInsertDto>,
) -> Result<Vec<AccountEntity>> {
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
        ON CONFLICT (address) DO UPDATE SET address = excluded.address
        RETURNING id, address"#,
    );
    let mut query = sqlx::query_as(&query_builder);

    for event in accounts {
        query = query.bind(event.address);
    }
    let res: Vec<AccountEntity> = query.fetch_all(pool).await?;

    Ok(res)
}

pub async fn save_collaterals(
    pool: &DbPool,
    collaterals: Vec<AccountCollateralDto>,
) -> Result<()> {
    let mut query_builder = r#"
        INSERT INTO operational.account_collaterals (
            account_id,
            asset_id,
            amount
        ) VALUES "#
        .to_string();

    let placeholders = get_placeholders_str(3, collaterals.len());
    query_builder.push_str(&placeholders);
    query_builder.push_str(
        r#"
        ON CONFLICT (account_id, asset_id) 
        DO UPDATE SET amount = excluded.amount"#,
    );
    let mut query = sqlx::query(&query_builder);

    for collateral in collaterals {
        query = query
            .bind(collateral.account_id)
            .bind(collateral.asset_id)
            .bind(collateral.amount);
    }
    let _ = query.execute(pool).await?;

    Ok(())
}

pub async fn save_debts(pool: &DbPool, debts: Vec<AccountDebtDto>) -> Result<()> {
    let mut query_builder = r#"
        INSERT INTO operational.account_debts (
            account_id,
            asset_id,
            amount
        ) VALUES "#
        .to_string();

    let placeholders = get_placeholders_str(3, debts.len());
    query_builder.push_str(&placeholders);
    query_builder.push_str(
        r#"
        ON CONFLICT (account_id, asset_id) 
        DO UPDATE SET amount = excluded.amount"#,
    );
    let mut query = sqlx::query(&query_builder);

    for debt in debts {
        query = query
            .bind(debt.account_id)
            .bind(debt.asset_id)
            .bind(debt.amount);
    }
    let _ = query.execute(pool).await?;

    Ok(())
}
