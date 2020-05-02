use async_std::{sync::Arc, task};
use async_tungstenite::async_std::connect_async;
use futures::future::{self, AbortHandle};
use log::debug;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

mod session;
mod session_state;
mod shard_processor;

use self::session::Session;
use self::shard_processor::ShardProcessor;

use crate::client::Client;
use crate::error::Error;
use crate::utils::WebsocketStream;

#[derive(Default)]
pub struct Connection {}

impl Connection {
  pub async fn connect(client: Arc<Client>) -> Result<WebsocketStream, BoxError> {
    match connect_async(&client.gateway_url).await {
      Ok((ws_stream, _)) => Ok(ws_stream),
      Err(_error) => Err(Box::new(Error::GatewayConnectionError)),
    }
  }
}

pub struct Shard {
  id: i32,
  processor_handle: AbortHandle,
  session: Arc<Session>,
}

impl Shard {
  pub async fn connect(id: i32, client: Arc<Client>) -> Result<Self, BoxError> {
    let websocket = Connection::connect(client.clone()).await?;
    let session = Arc::new(Session::new());
    let processor = ShardProcessor::new(websocket, session.clone());
    let (process, processor_handle) = future::abortable(processor.process());

    task::spawn(async move {
      let _ = process.await;
      debug!("[shard] Shard #{} processor has finished.", id);
    });

    Ok(Self {
      id,
      processor_handle,
      session,
    })
  }
}

pub struct Gateway {
  shards: Vec<Shard>,
  client: Arc<Client>,
}

impl Gateway {
  pub async fn connect(client: Arc<Client>) -> Result<Self, BoxError> {
    let number_of_shards = 1;
    let mut shards = Vec::with_capacity(number_of_shards);

    for idx in 0..number_of_shards {
      let shard = Shard::connect(idx as i32, client.clone()).await?;
      shards.push(shard);
    }

    Ok(Self { shards, client })
  }
}
