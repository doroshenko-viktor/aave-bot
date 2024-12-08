use std::{collections::HashMap, hash::Hash};

use alloy::rpc::types::{Block, Log};
use async_trait::async_trait;
use eyre::{eyre, Error, OptionExt};
use sqlx::{
    types::{
        time::{OffsetDateTime, PrimitiveDateTime},
        BigDecimal,
    },
    Pool, Postgres,
};
use tracing::info;

use crate::persistence::get_placeholders_str;

use super::PostgresPool;

#[derive(Debug, Eq, sqlx::FromRow)]
pub struct BlockDto {
    pub block_hash: Vec<u8>,
    pub block_number: BigDecimal,
    pub block_timestamp: PrimitiveDateTime,
}

impl BlockDto {
    pub fn new(
        block_hash: Vec<u8>,
        block_number: BigDecimal,
        block_timestamp: PrimitiveDateTime,
    ) -> Self {
        Self {
            block_hash,
            block_number,
            block_timestamp,
        }
    }
}

impl PartialEq for BlockDto {
    fn eq(&self, other: &Self) -> bool {
        self.block_number == other.block_number
    }
}

impl Hash for BlockDto {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.block_number.hash(state);
    }
}

impl TryFrom<&Block> for BlockDto {
    type Error = eyre::Error;

    fn try_from(value: &Block) -> Result<Self, Self::Error> {
        let block_hash = value.header.hash.to_vec();
        let block_number = BigDecimal::from(value.header.number);
        let block_timestamp =
            match OffsetDateTime::from_unix_timestamp(value.header.timestamp as i64) {
                Ok(x) => Ok(PrimitiveDateTime::new(x.date(), x.time())),
                Err(e) => Err(eyre!("Can't parse timestamp: {}", e)),
            }?;

        Ok(BlockDto {
            block_hash,
            block_number,
            block_timestamp,
        })
    }
}

// impl TryFrom<&Log> for BlockDto {
//     type Error = eyre::Error;

//     fn try_from(value: &Log) -> Result<Self, Self::Error> {
//         let block_hash = value.block_hash.map(|x|x.to_vec());
//         let block_number = BigDecimal::from(value.block_number.ok_or_eyre("event log must have a block number")?);
//         let block_timestamp =value.block_timestamp.map(|timestamp|{
//             match OffsetDateTime::from_unix_timestamp(timestamp as i64) {
//                 Ok(x) => Ok(PrimitiveDateTime::new(x.date(), x.time())),
//                 Err(e) => Err(eyre!("Can't parse timestamp: {}", e)),
//             }
//         }).transpose()?;

//         Ok(BlockDto {
//             block_hash,
//             block_number,
//             block_timestamp,
//         })
//     }
// }

pub type BlockNumber = BigDecimal;
pub type BlockDbId = i32;
pub type BlockDatabaseIds = HashMap<BlockNumber, BlockDbId>;

#[derive(Debug, sqlx::FromRow)]
struct BlockInsertResult {
    db_id: BlockDbId,
    block_number: BigDecimal,
}

#[async_trait]
pub trait BlocksPersistence {
    async fn save_blocks(&self, blocks: Vec<BlockDto>) -> Result<BlockDatabaseIds, Error>;
}

#[async_trait]
impl BlocksPersistence for Pool<Postgres> {
    async fn save_blocks(&self, blocks: Vec<BlockDto>) -> Result<BlockDatabaseIds, Error> {
        return save_blocks(self, blocks).await;
    }
}

pub async fn save_blocks(
    pool: &PostgresPool,
    blocks: Vec<BlockDto>,
) -> Result<BlockDatabaseIds, Error> {
    info!("Saving blocks into database");
    let mut query_builder = String::from(
        r#"INSERT INTO eth.blocks (
            block_hash,
            block_number,
            block_timestamp
        ) VALUES "#,
    );

    query_builder.push_str(&get_placeholders_str(3, blocks.len()));
    query_builder.push_str(
        r#" ON CONFLICT (block_number) DO UPDATE SET block_number = EXCLUDED.block_number RETURNING id as db_id, block_number"#,
    );

    let mut query = sqlx::query_as::<_, BlockInsertResult>(&query_builder);

    for block in blocks {
        query = query
            .bind(block.block_hash)
            .bind(block.block_number)
            .bind(block.block_timestamp);
    }
    let res = query.fetch_all(pool).await?;
    let ids = res
        .into_iter()
        .fold(HashMap::<BigDecimal, i32>::new(), |mut acc, x| {
            acc.entry(x.block_number).or_insert_with(|| x.db_id);
            acc
        });

    Ok(ids)
}
