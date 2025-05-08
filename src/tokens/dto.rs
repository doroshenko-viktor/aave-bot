use super::entities::Token;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TokenDto {
    pub id: i32,
    pub token_symbol: String,
    pub address: String,
    pub decimals: i32,
    pub total_supply: String,
    pub a_token_address: String,
    pub variable_debt_token_address: String,
    pub stable_debt_token_address: String,
}

impl From<Token> for TokenDto {
    fn from(token: Token) -> Self {
        Self {
            id: token.id as i32,
            token_symbol: token.token_symbol,
            address: token.address.to_string(),
            decimals: token.decimals as i32,
            total_supply: token.total_supply.to_string(),
            a_token_address: token.a_token_address.to_string(),
            variable_debt_token_address: token
                .variable_debt_token_address
                .to_string(),
            stable_debt_token_address: token
                .stable_debt_token_address
                .to_string(),
        }
    }
}
