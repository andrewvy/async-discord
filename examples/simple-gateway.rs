use std::env;

use async_discord::client::ClientBuilder;
use async_discord::gateway::Gateway;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
  pretty_env_logger::init();

  let token = env::var("DISCORD_TOKEN").expect("Environment variable 'DISCORD_TOKEN' is required.");
  let client = ClientBuilder::new(token).create().await?;
  let mut gateway = Gateway::new().connect(client.clone()).await?;

  gateway.process().await;

  Ok(())
}
