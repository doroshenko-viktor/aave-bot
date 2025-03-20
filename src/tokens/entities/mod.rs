use std::fmt::Display;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Token {
    pub(in crate::tokens) id: u8,
    pub(in crate::tokens) token_symbol: String,
}

impl Token {
    pub fn new(id: u8, token_symbol: String) -> Self {
        Self { id, token_symbol }
    }

    pub fn get_symbol(&self) -> String {
        self.token_symbol.clone()
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: implement
        todo!()
    }
}
