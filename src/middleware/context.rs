use async_std::sync::{Arc, RwLock};

use typemap::{ShareMap, TypeMap};

use crate::cache::Cache;
use crate::client::Client;

/// Context-wrapped event that provides some niceties for storing
/// additional data for the duration of the event as it moves through
/// the middleware stack.
pub struct Context<Event> {
  /// Wrapped event.
  pub event: Event,
  pub client: Arc<Client>,
  pub cache: Arc<Cache>,
  local: Arc<RwLock<ShareMap>>,
}

impl<Event> Context<Event> {
  pub fn new(client: Arc<Client>, cache: Arc<Cache>, event: Event) -> Context<Event> {
    Self {
      client,
      event,
      cache,
      local: Arc::new(RwLock::new(TypeMap::custom())),
    }
  }

  pub fn local(&self) -> Arc<RwLock<ShareMap>> {
    self.local.clone()
  }
}
