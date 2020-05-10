use std::env;
use std::sync::Arc;

use async_discord::typemap::Key;
use async_discord::{ClientBuilder, Context, DispatchEvent, Gateway, Middleware, Next};
use async_trait::async_trait;

type State = ();

/// Populates the current authenticated user from somewhere (DB).
pub struct AuthMiddleware {}

/// Some authenticated user struct.
pub struct AuthenticatedUser {}

impl Key for AuthenticatedUser {
  type Value = Self;
}

#[async_trait]
impl Middleware<State> for AuthMiddleware {
  async fn handle<'a>(
    &'a self,
    state: Arc<State>,
    ctx: Context<DispatchEvent>,
    next: Next<'a, State>,
  ) {
    let data = ctx.local();
    data
      .write()
      .await
      .insert::<AuthenticatedUser>(AuthenticatedUser {});
    next.run(state, ctx).await;
  }
}

pub struct RejectIfNotAuthed {}

#[async_trait]
impl Middleware<State> for RejectIfNotAuthed {
  async fn handle<'a>(
    &'a self,
    state: Arc<State>,
    ctx: Context<DispatchEvent>,
    next: Next<'a, State>,
  ) {
    let data = ctx.local();

    if data.read().await.contains::<AuthenticatedUser>() {
      next.run(state, ctx).await;
    }
  }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
  pretty_env_logger::init();

  let token = env::var("DISCORD_TOKEN").expect("Environment variable 'DISCORD_TOKEN' is required.");
  let client = ClientBuilder::new(token).create().await?;
  let mut gateway = Gateway::new(client.clone())
    .middleware(AuthMiddleware {})
    .middleware(RejectIfNotAuthed {})
    .connect()
    .await?;

  gateway.process().await;

  Ok(())
}
