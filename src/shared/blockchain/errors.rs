use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetEventLogsError {
    #[error("Too many logs retrieved")]
    TooManyLogs(Option<u64>),
    #[error(transparent)]
    Other(#[from] eyre::Report),
}

/**
ErrorPayload {
    code: -32005,
    message: "query returned more than 10000 results. Try with this block range [0x13823C1, 0x13835FF].",
    data: Some(
        RawValue(
            {"from":"0x13823C1","limit":10000,"to":"0x13835FF"},
        ),
    ),
},
*/
#[derive(Debug, Deserialize)]
pub struct TooManyLogsErrPayload {
    pub to: String,
}
