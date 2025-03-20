mod user_account;
use bigdecimal::BigDecimal;

pub type AssetAmount = BigDecimal;
pub use user_account::{UserAccount, UserAccountRef};
