use async_std::{stream::StreamExt, sync::Arc, task};
use futures::{
  channel::mpsc::{channel, Receiver, Sender},
  future::{self, AbortHandle},
};
use log::debug;

mod connection;
mod event;
mod session;
mod session_state;
mod shard_processor;

use self::event::DispatchEvent;
use self::session::Session;
use self::shard_processor::ShardProcessor;

use crate::client::Client;
use crate::utils::BoxError;

pub(crate) struct Shard {
  id: i32,
  processor_handle: AbortHandle,
  session: Arc<Session>,
}

impl Shard {
  pub async fn connect(
    id: i32,
    client: Arc<Client>,
    tx: Sender<Box<DispatchEvent>>,
  ) -> Result<Self, BoxError> {
    let session = Arc::new(Session::new());
    let processor = ShardProcessor::start(client.clone(), session.clone(), tx).await?;
    let (process, processor_handle) = future::abortable(processor.process());

    task::spawn(async move {
      let _ = process.await;
      debug!("[shard] Shard #{} processor has finished executing.", id);
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
  rx: Receiver<Box<DispatchEvent>>,
}

impl Gateway {
  pub async fn connect(client: Arc<Client>) -> Result<Self, BoxError> {
    let number_of_shards = 1;
    let mut shards = Vec::with_capacity(number_of_shards);

    let (tx, rx) = channel(100);

    for idx in 0..number_of_shards {
      let shard = Shard::connect(idx as i32, client.clone(), tx.clone()).await?;
      shards.push(shard);
    }

    Ok(Self { shards, client, rx })
  }

  pub async fn next(&mut self) -> Option<Box<DispatchEvent>> {
    self.rx.next().await
  }
}
