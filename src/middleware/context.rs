use std::sync::Arc;

use crate::client::Client;

/// Context-wrapped event that provides some niceties for storing
/// additional data for the duration of the event as it moves through
/// the middleware stack.
pub struct Context<Event> {
  /// Wrapped event.
  pub event: Event,
  pub client: Arc<Client>,
}

impl<Event> Context<Event> {
  pub fn new(client: Arc<Client>, event: Event) -> Context<Event> {
    Self { client, event }
  }
}
