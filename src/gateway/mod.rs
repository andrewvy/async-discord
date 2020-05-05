use async_std::{stream::StreamExt, sync::Arc, task};
use futures::{
  channel::mpsc::{channel, Receiver, Sender},
  future::{self, AbortHandle},
};
use log::debug;

mod connection;
pub mod event;
mod session;
mod session_state;
mod shard_processor;

use self::event::DispatchEvent;
use self::session::Session;
use self::session_state::SessionState;
use self::shard_processor::ShardProcessor;

use crate::client::Client;
use crate::middleware::{Middleware, Next};
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

  pub async fn shutdown(&mut self) {
    self.processor_handle.abort();
  }
}

pub struct Gateway<State> {
  shards: Vec<Shard>,
  tx: Sender<Box<DispatchEvent>>,
  rx: Receiver<Box<DispatchEvent>>,
  state: Arc<State>,
  middleware: Arc<Vec<Arc<dyn Middleware<State>>>>,
}

const GATEWAY_CHANNEL_SIZE: usize = 100;

impl Gateway<()> {
  /// Create a new [`Gateway`].
  pub fn new() -> Gateway<()> {
    Self::with_state(())
  }
}

impl<State: Send + Sync + 'static> Gateway<State> {
  pub fn with_state(state: State) -> Gateway<State> {
    let (tx, rx) = channel(GATEWAY_CHANNEL_SIZE);

    Self {
      shards: Vec::default(),
      tx,
      rx,
      state: Arc::new(state),
      middleware: Arc::new(vec![]),
    }
  }

  pub fn middleware<M>(mut self, middleware: M) -> Self
  where
    M: Middleware<State>,
  {
    let middlewares =
      Arc::get_mut(&mut self.middleware).expect("Unable to register new middleware.");
    middlewares.push(Arc::new(middleware));
    self
  }

  /// Given a [`Client`] for configuration, connects to the gateway.
  pub async fn connect(mut self, client: Arc<Client>) -> Result<Self, BoxError> {
    let number_of_shards = 1;
    let mut shards = Vec::with_capacity(number_of_shards);

    for idx in 0..number_of_shards {
      let shard = Shard::connect(idx as i32, client.clone(), self.tx.clone()).await?;
      shards.push(shard);
    }

    self.shards = shards;

    Ok(self)
  }

  /// Processes the next event from the gateway (from all the shards.)
  pub async fn next(&mut self) -> bool {
    let middleware = self.middleware.clone();
    let state = self.state.clone();

    if let Some(event) = self.rx.next().await {
      let next = Next {
        next_middleware: &middleware,
      };

      debug!("[gateway] Processing message through middleware stack.");

      next.run(state, *event).await;

      true
    } else {
      false
    }
  }

  /// Processes all events through the gateway.
  pub async fn process(&mut self) {
    loop {
      if !self.next().await {
        break;
      }
    }
  }

  /// Shutdown all shards.
  pub async fn shutdown(&mut self) {
    for shard in self.shards.iter_mut() {
      shard.shutdown().await;
    }
  }

  /// Gets all the shard statuses.
  /// Returns (id, SessionState).
  pub fn get_shard_status(&self) -> Vec<(i32, SessionState)> {
    self
      .shards
      .iter()
      .map(|shard| (shard.id, shard.session.get_state()))
      .collect()
  }
}
