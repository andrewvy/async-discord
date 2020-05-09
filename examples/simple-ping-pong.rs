use std::env;
use std::sync::Arc;

use async_discord::client::{Client, ClientBuilder};
use async_discord::gateway::event::DispatchEvent;
use async_discord::gateway::Gateway;
use async_discord::http::channel::create_message::{CreateMessage, CreateMessageFields};
use async_discord::middleware::{Context, Middleware, Next, PrefixMiddleware};

use async_trait::async_trait;

pub struct PingCommand {}

pub type State = ();

#[async_trait]
impl Middleware<State> for PingCommand {
  async fn handle<'a>(
    &'a self,
    state: Arc<State>,
    ctx: Context<DispatchEvent>,
    next: Next<'a, State>,
  ) {
    match ctx.event {
      DispatchEvent::MessageCreate(ref msg) => {
        let fields = CreateMessageFields {
          content: "Pong!".to_string(),
          nonce: None,
          tts: None,
          embed: None,
        };

        let msg = CreateMessage::new(&ctx.client.http, msg.channel_id, fields);
        let _ = msg.execute().await;
      }
      _ => {}
    }

    next.run(state, ctx).await;
  }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
  pretty_env_logger::init();

  let token = env::var("DISCORD_TOKEN").expect("Environment variable 'DISCORD_TOKEN' is required.");
  let client = ClientBuilder::new(token).create().await?;
  let mut gateway = Gateway::new(client.clone())
    .middleware(PrefixMiddleware::new("~ping"))
    .middleware(PingCommand {})
    .connect()
    .await?;

  gateway.process().await;

  Ok(())
}
