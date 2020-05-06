use async_std::sync::Arc;
use async_trait::async_trait;

use crate::gateway::event::DispatchEvent as Event;

mod prefix;

pub use prefix::PrefixMiddleware;

#[async_trait]
pub trait Middleware<State>: Send + Sync + 'static {
  async fn handle<'a>(&'a self, state: Arc<State>, event: Event, next: Next<'a, State>);
}

pub struct Next<'a, State> {
  pub next_middleware: &'a [Arc<dyn Middleware<State>>],
}

impl<'a, State: 'static> Next<'a, State> {
  pub async fn run(mut self, state: Arc<State>, event: Event) {
    if let Some((current, next)) = self.next_middleware.split_first() {
      self.next_middleware = next;
      current.handle(state, event, self).await;
    }
  }
}
