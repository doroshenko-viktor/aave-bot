// use bot::app::events_history::{fetch_events, FetchEventsConfig};
// use bot::persistence::configuration::{get as get_config, ConfigurationKey};
// use bot::setup::{self, Application};
// use eyre::Error;
// use tracing::{debug, error, info, warn};

// #[tokio::main]
// async fn main() -> Result<(), Error> {
//     info!("starting bot");
//     let Application { rpc_rate_limiter } = setup::configure_app().await?;

//     let first_block: u64 = get_config(&db_pool, ConfigurationKey::LastHandledBlock)
//         .await?
//         .parse()?;

//     fetch_events(
//         &rpc_provider,
//         &db_pool,
//         FetchEventsConfig {
//             first_block,
//             indexing_step: settings.config.indexing_step,
//         },
//         rpc_rate_limiter,
//     )
//     .await?;

//     info!("Program finished execution");

//     Ok(())
// }
