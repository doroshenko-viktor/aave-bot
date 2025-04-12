use alloy::{rpc::types::Log, sol_types::SolEvent};

use eyre::Error;

use super::BlockchainEvent;

pub fn parse_event_logs(logs: &[Log]) -> (Vec<BlockchainEvent>, Vec<Error>) {
    logs.iter()
        .map(|log| {
            let event = BlockchainEvent::try_from(log)?;
            Ok(event)
        })
        .fold(
            (Vec::<BlockchainEvent>::new(), Vec::<Error>::new()),
            |mut acc, cur| {
                match cur {
                    Ok(event) => acc.0.push(event),
                    Err(err) => acc.1.push(err),
                }
                acc
            },
        )
}
