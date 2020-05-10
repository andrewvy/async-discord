use std::sync::Arc;

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
}

impl<Event> Context<Event> {
  pub fn new(client: Arc<Client>, cache: Arc<Cache>, event: Event) -> Context<Event> {
    Self {
      client,
      event,
      cache,
    }
  }
}
