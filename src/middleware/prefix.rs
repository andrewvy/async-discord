use async_trait::async_trait;
use std::sync::Arc;

use super::{Context, Middleware, Next};
use crate::gateway::event::DispatchEvent;

pub struct PrefixMiddleware {
  prefix: String,
}

impl PrefixMiddleware {
  pub fn new(prefix: &str) -> Self {
    PrefixMiddleware {
      prefix: prefix.to_owned(),
    }
  }
}

#[async_trait]
impl<State: Send + Sync + 'static> Middleware<State> for PrefixMiddleware {
  async fn handle<'a>(
    &'a self,
    state: Arc<State>,
    ctx: Context<DispatchEvent>,
    next: Next<'a, State>,
  ) {
    match ctx.event {
      DispatchEvent::MessageCreate(ref msg) => {
        if msg.content.starts_with(&self.prefix) {
          next.run(state, ctx).await;
        }
      }
      _ => {}
    }
  }
}
