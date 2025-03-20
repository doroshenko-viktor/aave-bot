use std::{fmt::Debug, path::Display};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
/*
Ticker representation inside of the system. It does contain only ticker id in ticker_service.
This is implemented this way, because tickers are used in multiple places in the system and sometimes require copy.
But in most cases system does not require any ticker data except it's identity. Usually we need to know exact ticker values on the input/output layer, when we receive price data and send some commands to price providers or blockchain.
*/
pub struct TickerId {
    id: u8,
}

impl TickerId {
    pub(in crate::price_feed) fn new(id: u8) -> Self {
        Self { id }
    }
}
