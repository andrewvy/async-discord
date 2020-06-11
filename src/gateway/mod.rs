use async_std::{stream::StreamExt, sync::Arc, task};
use futures::{
  channel::mpsc::{channel, Receiver, Sender},
  future::{self, AbortHandle, Either},
};
use log::debug;

mod connection;
pub mod event;
mod session;
mod session_state;
mod shard_processor;

use self::event::{DispatchEvent, GatewayAction};
use self::session::Session;
use self::session_state::SessionState;
use self::shard_processor::ShardProcessor;

use crate::cache::Cache;
use crate::client::Client;
use crate::interceptors::Interceptor;
use crate::middleware::{Context, Middleware, Next};
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

pub type GatewayActionSender = Sender<Box<GatewayAction>>;
pub type GatewayActionReceiver = Receiver<Box<GatewayAction>>;

pub struct Gateway<State> {
  shards: Vec<Shard>,
  tx: Sender<Box<DispatchEvent>>,
  rx: Receiver<Box<DispatchEvent>>,
  gateway_rx: GatewayActionReceiver,
  gateway_tx: GatewayActionSender,
  state: Arc<State>,
  middleware: Arc<Vec<Arc<dyn Middleware<State>>>>,
  client: Arc<Client>,
  cache: Arc<Cache>,
  interceptors: Vec<Interceptor>,
}

const GATEWAY_CHANNEL_SIZE: usize = 100;

impl Gateway<()> {
  /// Create a new [`Gateway`] with () as the State.
  pub fn new(client: Arc<Client>) -> Gateway<()> {
    Self::with_state(client, ())
  }
}

impl<State: Send + Sync + 'static> Gateway<State> {
  /// Creates a [`Gateway`] with a defined State.
  pub fn with_state(client: Arc<Client>, state: State) -> Gateway<State> {
    let (tx, rx) = channel(GATEWAY_CHANNEL_SIZE);
    let (gateway_tx, gateway_rx) = channel(GATEWAY_CHANNEL_SIZE);
    let cache = Arc::new(Cache::new());

    Self {
      shards: Vec::default(),
      tx,
      rx,
      gateway_tx,
      gateway_rx,
      client,
      cache,
      state: Arc::new(state),
      middleware: Arc::new(vec![]),
      interceptors: Vec::new(),
    }
  }

  /// Add a new middleware to the middleware stack.
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
  pub async fn connect(mut self) -> Result<Self, BoxError> {
    let number_of_shards = 1;
    let mut shards = Vec::with_capacity(number_of_shards);

    for idx in 0..number_of_shards {
      let shard = Shard::connect(idx as i32, self.client.clone(), self.tx.clone()).await?;
      shards.push(shard);
    }

    self.shards = shards;

    Ok(self)
  }

  /// Processes the next action or event from the gateway (from all the shards.)
  /// Actions can be sent from processing middlewares to communicate with
  /// this gateway. Events are sent from all shards to here, and then
  /// subsequently dispatched through the middleware stack.
  pub async fn next(&mut self) -> bool {
    let gateway_action = self.gateway_rx.next();
    let dispatch_event = self.rx.next();
    let action_or_event = future::select(gateway_action, dispatch_event).await;

    match action_or_event {
      // Process an action.
      Either::Left((Some(action), _)) => {
        match *action {
          GatewayAction::AddInterceptor(interceptor) => {
            self.interceptors.push(interceptor);
          }
        }

        true
      }
      // Process an event.
      Either::Right((Some(event), _)) => {
        fn retain<T, F>(vec: &mut Vec<T>, mut f: F)
        where
          F: FnMut(&mut T) -> bool,
        {
          let len = vec.len();
          let mut del = 0;
          {
            let v = &mut **vec;

            for i in 0..len {
              if !f(&mut v[i]) {
                del += 1;
              } else if del > 0 {
                v.swap(i - del, i);
              }
            }
          }

          if del > 0 {
            vec.truncate(len - del);
          }
        }

        // If we have any interceptors, check if the event
        // should be intercepted by any of them.
        let intercepted = if !self.interceptors.is_empty() {
          let intercepted_event = Arc::new(event.as_ref().clone());
          let mut intercepted = false;

          retain(&mut self.interceptors, move |interceptor| {
            if interceptor.matches(&intercepted_event) {
              intercepted = interceptor.send(intercepted_event.clone());
              return intercepted;
            }

            true
          });

          intercepted
        } else {
          false
        };

        if !intercepted {
          // Construct a new context for this event,
          // and pass it through the middleware stack.
          self.send_through_middlewares(event);
        }

        true
      }
      Either::Left((None, _)) => false,
      Either::Right((None, _)) => false,
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

  pub fn send_through_middlewares(&self, event: Box<DispatchEvent>) {
    let state = self.state.clone();
    let middleware = self.middleware.clone();
    let client = self.client.clone();
    let cache = self.cache.clone();
    let gateway = self.gateway_tx.clone();

    task::spawn(async move {
      let next = Next {
        next_middleware: middleware.as_slice(),
      };

      next
        .run(state, Context::new(client, cache, gateway, *event))
        .await;
    });
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
