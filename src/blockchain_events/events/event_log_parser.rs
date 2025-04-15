use super::BlockchainEvent;
use crate::utils::result_utils::CollectionResult;
use alloy::rpc::types::Log;

pub fn parse_event_logs(logs: &[Log]) -> CollectionResult<BlockchainEvent> {
    // (Vec<BlockchainEvent>, Vec<Error>) {
    logs.iter()
        .map(|log| {
            let event = BlockchainEvent::try_from(log)?;
            Ok(event)
        })
        .fold(
            CollectionResult::new(),
            |mut acc, cur| {
                match cur {
                    Ok(event) => acc.results.push(event),
                    Err(err) => acc.errors.push(err),
                }
                acc
            },
        )
}
