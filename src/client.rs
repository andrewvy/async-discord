use std::collections::HashMap;

use async_std::{sync::Arc, task};
use async_trait::async_trait;
use twilight_model::{
  channel::{message::MessageType, Message},
  id::{ChannelId, MessageId, UserId},
  user::User,
};

use crate::error::Error;
use crate::gateway::Gateway;
use crate::http;

#[derive(Clone, Debug)]
pub struct Client {
  pub token: String,
  pub gateway_url: String,
}

impl Client {
  pub async fn receive_message(&self) -> Result<Message, Error> {
    Ok(Message {
      id: MessageId(0),
      activity: None,
      application: None,
      attachments: Vec::default(),
      author: User {
        id: UserId(0),
        avatar: None,
        bot: false,
        discriminator: "2132".into(),
        name: "Test".into(),
        mfa_enabled: None,
        locale: None,
        verified: None,
        email: None,
        flags: None,
        premium_type: None,
        system: None,
      },
      channel_id: ChannelId(0),
      content: "Hello world!".into(),
      edited_timestamp: None,
      embeds: Vec::default(),
      flags: None,
      guild_id: None,
      kind: MessageType::Regular,
      member: None,
      mention_channels: Vec::default(),
      mention_everyone: false,
      mention_roles: Vec::default(),
      mentions: HashMap::new(),
      pinned: false,
      reactions: Vec::default(),
      reference: None,
      timestamp: "2017-07-11T17:27:07.299000+00:00".into(),
      tts: false,
      webhook_id: None,
    })
  }
}

pub struct ClientBuilder {
  token: String,
}

impl ClientBuilder {
  pub fn new(token: impl Into<String>) -> Self {
    Self {
      token: token.into(),
    }
  }

  pub async fn create(&mut self) -> Result<Arc<Client>, Box<dyn std::error::Error + Send + Sync>> {
    let gateway_response = http::get_gateway::get_gateway().await?;
    let gateway_url = gateway_response.url;

    let client = Arc::new(Client {
      token: self.token.to_owned(),
      gateway_url: gateway_url.into(),
    });

    Ok(client)
  }
}

#[async_trait]
pub trait MessageExt {
  async fn reply(&self, content: &str) -> Result<(), Error>;
}

#[async_trait]
impl MessageExt for Message {
  async fn reply(&self, _content: &str) -> Result<(), Error> {
    Ok(())
  }
}
