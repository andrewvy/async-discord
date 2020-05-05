use async_std::sync::Arc;

use crate::http;

#[derive(Clone, Debug)]
pub struct Client {
  pub(crate) token: String,
  pub(crate) gateway_url: String,
  pub(crate) gateway_guild_subscriptions: bool,
}

impl Client {}

pub struct ClientBuilder {
  token: String,
  gateway_guild_subscriptions: bool,
}

impl ClientBuilder {
  pub fn new(token: impl Into<String>) -> Self {
    Self {
      token: token.into(),
      gateway_guild_subscriptions: true,
    }
  }

  pub fn set_guild_subscriptions(&mut self, guild_subscriptions: bool) {
    self.gateway_guild_subscriptions = guild_subscriptions;
  }

  pub async fn create(&mut self) -> Result<Arc<Client>, Box<dyn std::error::Error + Send + Sync>> {
    let gateway_response = http::get_gateway::get_gateway().await?;
    let gateway_url = gateway_response.url;

    let client = Arc::new(Client {
      token: self.token.to_owned(),
      gateway_url: gateway_url.into(),
      gateway_guild_subscriptions: self.gateway_guild_subscriptions,
    });

    Ok(client)
  }
}
