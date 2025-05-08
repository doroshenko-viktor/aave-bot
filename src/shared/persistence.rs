use core::fmt;
use std::{
    fmt::{Display, LowerHex},
    io::Read,
    ops::Deref,
    str::FromStr,
};

use alloy::primitives::Address;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, Visitor},
};
use sqlx::{Pool, Postgres};

pub const SCHEMA_BLOCKCHAIN: &str = "blockchain";
pub const SCHEMA_CONFIG: &str = "config";

pub type DbPool = Pool<Postgres>;

// TODO: move these types to shared
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AccountAddress {
    pub address: Address,
}

impl AccountAddress {
    pub fn new(address: Address) -> Self {
        Self { address }
    }
}

impl From<Address> for AccountAddress {
    fn from(address: Address) -> Self {
        Self { address }
    }
}

impl Display for AccountAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.address)
    }
}

impl LowerHex for AccountAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.address)
    }
}

impl serde::Serialize for AccountAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_string = format!("{:#x}", self.address);
        serializer.serialize_str(&hex_string)
    }
}

impl<'de> Deserialize<'de> for AccountAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AddressVisitor;

        impl<'de> Visitor<'de> for AddressVisitor {
            type Value = AccountAddress;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("20-byte Ethereum address")
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let address = Address::from_str(&v).map_err(|_| {
                    E::custom(format!("Invalid address: {}", v))
                })?;
                Ok(AccountAddress::new(address))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let address = Address::from_str(v).map_err(|_| {
                    E::custom(format!("Invalid address: {}", v))
                })?;
                Ok(AccountAddress::new(address))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<AccountAddress, E>
            where
                E: de::Error,
            {
                let address = Address::from_slice(v);
                Ok(AccountAddress::new(address))
            }
        }

        deserializer.deserialize_bytes(AddressVisitor)
    }
}

impl Deref for AccountAddress {
    type Target = Address;

    fn deref(&self) -> &Self::Target {
        &self.address
    }
}

pub type AccountAddressBin = [u8; 20];

pub fn get_placeholders_str(
    placeholders_amount: usize,
    rows_amount: usize,
) -> String {
    let mut ind = 0;
    let mut placeholders = Vec::<String>::with_capacity(rows_amount);

    for _ in 0..rows_amount {
        let mut x = Vec::<String>::with_capacity(placeholders_amount * 3);
        for _ in 0..placeholders_amount {
            ind += 1;
            x.push(format!("${}", ind));
        }
        let cur_placeholder = x.join(",");
        placeholders.push(format!("({})", cur_placeholder));
    }

    placeholders.join(",")
}

pub struct CustomPlaceholder {
    pub placeholder_prefix: String,
    pub placeholder_postfix: String,
}
impl CustomPlaceholder {
    pub fn new(
        placeholder_prefix: String,
        placeholder_postfix: String,
    ) -> Self {
        Self { placeholder_prefix, placeholder_postfix }
    }

    pub fn get_placeholder(&self, index: usize) -> String {
        format!(
            "{}${}{}",
            self.placeholder_prefix, index, self.placeholder_postfix
        )
    }
    pub fn length(&self) -> usize {
        self.placeholder_prefix.len() + self.placeholder_postfix.len() + 3
    }
}

pub fn get_custom_placeholders_str(
    custom_placeholders: Vec<Option<CustomPlaceholder>>,
    rows_amount: usize,
) -> String {
    let mut ind = 0;
    let mut placeholders = Vec::<String>::with_capacity(rows_amount);

    let min_line_length: usize = custom_placeholders
        .iter()
        .map(
            |x| {
                if let Some(p) = x { p.length() } else { 3 }
            },
        )
        .sum();
    let mut x = Vec::<String>::with_capacity(min_line_length);

    for _ in 0..rows_amount {
        for placeholder in &custom_placeholders {
            ind += 1;
            match placeholder {
                Some(p) => x.push(p.get_placeholder(ind)),
                None => x.push(format!("${}", ind)),
            };
        }
        let cur_placeholder = x.join(",");
        placeholders.push(format!("({})", cur_placeholder));

        x.clear();
    }

    placeholders.join(",")
}

#[cfg(test)]
pub mod tests {
    use super::get_placeholders_str;

    #[test]
    pub fn test_1() {
        let res = get_placeholders_str(1, 1);
        assert_eq!("($1)", res);
    }

    #[test]
    pub fn test_2() {
        let res = get_placeholders_str(4, 1);
        assert_eq!("($1,$2,$3,$4)", res);
    }

    #[test]
    pub fn test_3() {
        let res = get_placeholders_str(5, 5);
        assert_eq!(
            "($1,$2,$3,$4,$5),($6,$7,$8,$9,$10),($11,$12,$13,$14,$15),($16,$17,$18,$19,$20),($21,$22,$23,$24,$25)",
            res
        );

        println!("{}", res);
    }
}
