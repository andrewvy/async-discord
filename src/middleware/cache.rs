use std::sync::Arc;

use crate::{Context, DispatchEvent, Middleware, Next};
use async_trait::async_trait;
use twilight_model::guild::GuildStatus;

pub struct Cache {}

impl Cache {
  pub fn new() -> Self {
    Self {}
  }
}

#[async_trait]
impl<State: Send + Sync + 'static> Middleware<State> for Cache {
  async fn handle<'a>(
    &'a self,
    state: Arc<State>,
    ctx: Context<DispatchEvent>,
    next: Next<'a, State>,
  ) {
    match ctx.event {
      DispatchEvent::Ready(ref ready) => {
        ctx.cache.cache_me(ready.user.clone()).await;

        for guild_status in ready.guilds.values() {
          if let GuildStatus::Online(guild) = guild_status {
            ctx.cache.cache_guild(guild.clone()).await
          }
        }
      }
      DispatchEvent::GuildCreate(ref guild_create) => {
        ctx.cache.cache_guild(guild_create.0.clone()).await
      }
      _ => {}
    }

    next.run(state, ctx).await;
  }
}
