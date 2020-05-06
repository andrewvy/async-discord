use serde::Serialize;
use twilight_model::{channel::Message, id::ChannelId};

use crate::http::HttpClient;
use crate::utils::BoxError;

#[derive(Serialize)]
pub struct Embed {}

#[derive(Serialize)]
pub struct CreateMessageFields {
  pub content: String,
  pub nonce: Option<String>,
  pub tts: Option<bool>,
  pub embed: Option<Embed>,
}

pub struct CreateMessage<'a> {
  fields: CreateMessageFields,
  http: &'a HttpClient,
  channel_id: ChannelId,
}

impl<'a> CreateMessage<'a> {
  pub fn new(http: &'a HttpClient, channel_id: ChannelId, fields: CreateMessageFields) -> Self {
    Self {
      http,
      channel_id,
      fields,
    }
  }

  pub async fn execute(&self) -> Result<Message, BoxError> {
    let message: Message = self
      .http
      .post(&format!("/channels/{}/messages", self.channel_id))
      .body_json(&self.fields)?
      .recv_json()
      .await?;

    Ok(message)
  }
}
