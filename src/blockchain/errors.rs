use alloy::transports::{
    HttpError, RpcError, TransportError, TransportErrorKind,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlockchainContractCallError {
    #[error("Too many call attempts")]
    TooManyAttempts(String),

    #[error(transparent)]
    Other(#[from] eyre::Report),
}

impl From<alloy_contract::Error> for BlockchainContractCallError {
    fn from(err: alloy_contract::Error) -> Self {
        match err {
            alloy_contract::Error::TransportError(RpcError::Transport(
                alloy::transports::TransportErrorKind::HttpError(HttpError {
                    status,
                    body,
                }),
            )) if status == 429 => {
                BlockchainContractCallError::TooManyAttempts(body)
            }
            _ => BlockchainContractCallError::Other(err.into()),
        }
    }
}

impl From<RpcError<TransportErrorKind>> for BlockchainContractCallError {
    fn from(err: RpcError<TransportErrorKind>) -> Self {
        match err {
            RpcError::Transport(TransportErrorKind::HttpError(HttpError {
                status,
                body,
            })) if status == 429 => {
                BlockchainContractCallError::TooManyAttempts(body)
            }
            _ => BlockchainContractCallError::Other(err.into()),
        }
    }
}
