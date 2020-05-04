use async_std::sync::Arc;
use async_tungstenite::async_std::connect_async_with_config;

use crate::client::Client;
use crate::error::Error;
use crate::utils::{BoxError, WebsocketStream};

#[derive(Default)]
pub(crate) struct Connection {}

impl Connection {
  pub(crate) async fn connect(client: Arc<Client>) -> Result<WebsocketStream, BoxError> {
    // @TODO(vy): Configurable gateway parameters.
    match connect_async_with_config(
      &format!("{}/?v=6&compress=zlib-stream", client.gateway_url),
      Some(async_tungstenite::tungstenite::protocol::WebSocketConfig {
        max_message_size: None,
        max_frame_size: None,
        max_send_queue: None,
      }),
    )
    .await
    {
      Ok((ws_stream, _)) => Ok(ws_stream),
      Err(error) => {
        dbg!(error);
        Err(Box::new(Error::GatewayConnectionError))
      }
    }
  }
}
