mod spot;
mod ws_client;

use tokio::sync::mpsc::Receiver;
use tokio_tungstenite::tungstenite::Utf8Bytes;
pub use ws_client::{BybitClientSpot, BybitWsConnectionManager, run_bybit_ws_channel};

pub type BybitEventRX = Receiver<eyre::Result<Utf8Bytes>>;
