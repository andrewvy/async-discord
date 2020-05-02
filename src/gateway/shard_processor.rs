use async_std::sync::Arc;
use futures::prelude::*;

use super::session::Session;

use crate::utils::WebsocketStream;

/// Listens on the websocket events, handles any session-related events, and forwards messages upstream.
pub struct ShardProcessor {
  websocket: WebsocketStream,
  session: Arc<Session>,
}

impl ShardProcessor {
  pub fn new(websocket: WebsocketStream, session: Arc<Session>) -> Self {
    let processor = Self { websocket, session };

    processor
  }

  pub async fn process(mut self) {
    loop {
      let msg = self.websocket.next().await;
    }
  }
}
