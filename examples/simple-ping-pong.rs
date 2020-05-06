use std::env;
use std::sync::Arc;

use async_discord::client::{Client, ClientBuilder};
use async_discord::gateway::event::DispatchEvent;
use async_discord::gateway::Gateway;
use async_discord::http::channel::create_message::{CreateMessage, CreateMessageFields};
use async_discord::middleware::{Middleware, Next, PrefixMiddleware};

use async_trait::async_trait;

use log::info;

pub struct PingCommand {}

pub struct State {
  client: Arc<Client>,
}

#[async_trait]
impl Middleware<State> for PingCommand {
  async fn handle<'a>(&'a self, state: Arc<State>, event: DispatchEvent, next: Next<'a, State>) {
    match event {
      DispatchEvent::MessageCreate(ref msg) => {
        let fields = CreateMessageFields {
          content: "Pong!".to_string(),
          nonce: None,
          tts: None,
          embed: None,
        };

        let msg = CreateMessage::new(&state.client.http, msg.channel_id, fields);

        msg.execute().await;
      }
      _ => {}
    }

    next.run(state, event).await;
  }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
  pretty_env_logger::init();

  let token = env::var("DISCORD_TOKEN").expect("Environment variable 'DISCORD_TOKEN' is required.");
  let client = ClientBuilder::new(token).create().await?;
  let mut gateway = Gateway::with_state(State {
    client: client.clone(),
  })
  .middleware(PrefixMiddleware::new("~ping"))
  .middleware(PingCommand {})
  .connect(client.clone())
  .await?;

  gateway.process().await;

  Ok(())
}
