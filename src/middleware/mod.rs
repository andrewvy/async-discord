use async_std::sync::Arc;
use async_trait::async_trait;

use crate::event::Event;

#[async_trait]
pub trait Middleware<State>: Send + Sync + 'static {
  async fn handle<'a>(&'a self, event: Event<State>, next: Next<'a, State>) -> ();
}

pub struct Next<'a, State> {
  pub next_middleware: &'a [Arc<dyn Middleware<State>>],
}

impl<'a, State: 'static> Next<'a, State> {
  pub async fn run(mut self, event: Event<State>) -> () {
    if let Some((current, next)) = self.next_middleware.split_first() {
      self.next_middleware = next;
      current.handle(event, self).await
    } else {
      ()
    }
  }
}
