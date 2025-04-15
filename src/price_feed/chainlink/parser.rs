use eyre::{Error, eyre};
use std::str::FromStr;
use time::{OffsetDateTime, PrimitiveDateTime};

use crate::contracts::chainlink::Chainlink::{self};

// use super::ChainlinkFeedDto;
//
// impl TryFrom<Chainlink::AnswerUpdated> for ChainlinkFeedDto {
//     type Error = Error;
//
//     fn try_from(value: Chainlink::AnswerUpdated) -> Result<Self, Self::Error> {
//         let updated_at = match OffsetDateTime::from_unix_timestamp(
//             value.updatedAt.try_into()?,
//         ) {
//             Ok(x) => Ok(PrimitiveDateTime::new(
//                 x.date(),
//                 x.time(),
//             )),
//             Err(e) => Err(eyre!("Can't parse timestamp: {}", e)),
//         }?;
//         Ok(ChainlinkFeedDto {
//             current: BigDecimal::from_str(&value.current.to_string())?,
//             round_id: BigDecimal::from_str(&value.roundId.to_string())?,
//             updated_at,
//         })
//     }
// }
