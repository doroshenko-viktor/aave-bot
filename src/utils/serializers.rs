use std::num::ParseIntError;

use alloy::primitives::Address;
use serde::de::Error as SerdeError;
use serde::{Deserialize, Deserializer};

pub fn deserialize_eth_address<'de, D>(
    deserializer: D,
) -> Result<Option<Address>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        let address = s.parse::<Address>().map_err(SerdeError::custom)?;
        Ok(Some(address))
    }
}

pub fn deserialize_comma_separated_values_str<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        let tokens: Vec<String> = s.split(',').map(String::from).collect();
        Ok(Some(tokens))
    }
}

pub fn deserialize_comma_separated_values_i32<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<i32>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        match s
            .split(',')
            .map(|x| x.trim().parse::<i32>()) // Trim whitespace before parsing
            .collect::<Result<Vec<i32>, ParseIntError>>()
        {
            Ok(tokens) => Ok(Some(tokens)),
            Err(e) => Err(SerdeError::custom(format!(
                "invalid integer in comma-separated string: {}",
                e
            ))),
        }
    }
}
