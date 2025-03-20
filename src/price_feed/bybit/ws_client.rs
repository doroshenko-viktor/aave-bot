use std::{marker::PhantomData, sync::Arc};

use eyre::Result;
use futures::{SinkExt, StreamExt, stream::SplitSink};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream,
    tungstenite::{Message, Utf8Bytes},
};
use tracing::{debug, error, info};

use crate::price_feed::{ChangedTokenPriceTX, bybit::spot, entities::TickerSymbol};

pub type WsSender = Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>;

pub trait BybitClientType {}

pub struct BybitClientSpot {}
impl BybitClientType for BybitClientSpot {}

#[derive(Debug)]
pub struct BybitWsConnectionManager<T: BybitClientType> {
    sender: WsSender,
    marker: PhantomData<T>,
}

impl<T: BybitClientType> BybitWsConnectionManager<T> {
    fn new(sender: WsSender) -> Self {
        Self {
            sender,
            marker: PhantomData,
        }
    }
}

impl BybitWsConnectionManager<BybitClientSpot> {
    pub async fn subscribe_price(&mut self, ticker: TickerSymbol) -> Result<()> {
        let subscribe_msg = format!(
            r#"{{
                "op": "subscribe",
                "args": [tickers.{}]
            }}"#,
            ticker
        );

        let _ = self
            .sender
            .lock()
            .await
            .send(Message::Text(Utf8Bytes::from(subscribe_msg)))
            .await?;

        Ok(())
    }
}

pub async fn run_bybit_ws_channel<T: BybitClientType>(
    url: String,
    changed_token_price_tx: ChangedTokenPriceTX,
) -> Result<BybitWsConnectionManager<T>> {
    debug!("URL: {}", url);
    debug!("Creating channel");
    let (ws_stream, _) = tokio_tungstenite::connect_async(&url).await?;
    let (write, mut read) = ws_stream.split();
    let write = Arc::new(Mutex::new(write));
    let (result_tx, mut result_rx) = tokio::sync::mpsc::channel::<Result<Utf8Bytes>>(100);
    debug!("Starting to listen new stream");

    {
        let write = write.clone();
        tokio::spawn(async move {
            while let Some(result) = read.next().await {
                match result {
                    Ok(Message::Text(message)) => {
                        if let Err(e) = result_tx.send(Ok(message)).await {
                            error!("Error sending StreamEvent::Message: {}", e);
                        }
                    }
                    Ok(Message::Ping(val)) => {
                        debug!("WS Stream::received PING message");
                        if let Err(e) = write.lock().await.send(Message::Pong(val)).await {
                            error!("Error sending StreamEvent::Ping: {}", e);
                        }
                    }
                    Ok(Message::Pong(_val)) => {
                        debug!("WS Stream::received PONG message");
                    }
                    Ok(Message::Close(_val)) => {
                        debug!("Received CLOSE message");
                        break;
                    }
                    Ok(Message::Binary(_val)) => {
                        todo!("No websocket BINARY handler implemented");
                    }
                    Ok(Message::Frame(_val)) => {
                        debug!("No websocket FRAME handler implemented");
                    }
                    Err(e) => {
                        error!("WS Stream::error happened serving stream: {}", e);
                        let _ = result_tx.send(Err(e.into())).await;
                        break;
                    }
                };
            }
        });
    }

    spot::listen(result_rx, changed_token_price_tx);

    Ok(BybitWsConnectionManager::<T>::new(write))
}
