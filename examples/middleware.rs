use std::env;
use std::sync::Arc;

use async_discord::client::ClientBuilder;
use async_discord::gateway::event::DispatchEvent;
use async_discord::gateway::Gateway;
use async_discord::middleware::{Middleware, Next};

use async_trait::async_trait;

use log::info;

pub struct LogMiddleware {}

type State = ();

#[async_trait]
impl Middleware<State> for LogMiddleware {
  async fn handle<'a>(
    &'a self,
    state: Arc<State>,
    event: Box<DispatchEvent>,
    next: Next<'a, State>,
  ) {
    info!("Received event {:?}", event);

    next.run(state, event).await;
  }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
  pretty_env_logger::init();

  let token = env::var("DISCORD_TOKEN").expect("Environment variable 'DISCORD_TOKEN' is required.");
  let client = ClientBuilder::new(token).create().await?;
  let mut gateway = Gateway::new()
    .middleware(LogMiddleware {})
    .connect(client.clone())
    .await?;

  gateway.process().await;

  Ok(())
}
