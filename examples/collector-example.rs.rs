use std::env;

use async_discord::client::ClientBuilder;
use async_discord::client::MessageExt;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
  let token = env::var("DISCORD_TOKEN").expect("Environment variable 'DISCORD_TOKEN' is required.");

  let client = ClientBuilder::new(token)
    .middleware(Collector::new())
    .spawn()
    .await?;

  while let Ok(message) = client.receive_message().await {
    if message.content.starts_with("ping") {
      message.reply("Pong!").await?;
    }
  }

  Ok(())
}
