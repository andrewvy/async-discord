/// Shamelessly, 99% of this code piggybacks off of the gateway implementation from twilight-rs/twilight, thank you!
use std::{
  convert::TryFrom,
  fmt::{Formatter, Result as FmtResult},
};

use serde::{
  de::{Deserializer, MapAccess, Visitor},
  Deserialize,
};
use serde_json::Value;
use twilight_model::gateway::{payload::*, OpCode};

use crate::error::Error;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Field {
  D,
  Op,
  S,
  T,
}

#[derive(Clone, Debug)]
pub enum DispatchEvent {
  BanAdd(BanAdd),
  BanRemove(BanRemove),
  ChannelCreate(ChannelCreate),
  ChannelDelete(ChannelDelete),
  ChannelPinsUpdate(ChannelPinsUpdate),
  ChannelUpdate(ChannelUpdate),
  GuildCreate(Box<GuildCreate>),
  GuildDelete(Box<GuildDelete>),
  GuildEmojisUpdate(GuildEmojisUpdate),
  GuildIntegrationsUpdate(GuildIntegrationsUpdate),
  GuildUpdate(Box<GuildUpdate>),
  InviteCreate(Box<InviteCreate>),
  InviteDelete(InviteDelete),
  MemberAdd(Box<MemberAdd>),
  MemberRemove(MemberRemove),
  MemberUpdate(Box<MemberUpdate>),
  MemberChunk(MemberChunk),
  MessageCreate(Box<MessageCreate>),
  MessageDelete(MessageDelete),
  MessageDeleteBulk(MessageDeleteBulk),
  MessageUpdate(Box<MessageUpdate>),
  PresenceUpdate(Box<PresenceUpdate>),
  PresencesReplace,
  ReactionAdd(Box<ReactionAdd>),
  ReactionRemove(Box<ReactionRemove>),
  ReactionRemoveAll(ReactionRemoveAll),
  ReactionRemoveEmoji(ReactionRemoveEmoji),
  Ready(Box<Ready>),
  Resumed,
  RoleCreate(RoleCreate),
  RoleDelete(RoleDelete),
  RoleUpdate(RoleUpdate),
  TypingStart(Box<TypingStart>),
  UserUpdate(UserUpdate),
  VoiceServerUpdate(VoiceServerUpdate),
  VoiceStateUpdate(Box<VoiceStateUpdate>),
  WebhooksUpdate(WebhooksUpdate),
}

impl TryFrom<(&str, Value)> for DispatchEvent {
  type Error = Box<dyn std::error::Error>;

  fn try_from((kind, v): (&str, Value)) -> Result<Self, Self::Error> {
    Ok(match kind {
      "CHANNEL_CREATE" => Self::ChannelCreate(ChannelCreate::deserialize(v)?),
      "CHANNEL_DELETE" => Self::ChannelDelete(ChannelDelete::deserialize(v)?),
      "CHANNEL_PINS_UPDATE" => Self::ChannelPinsUpdate(ChannelPinsUpdate::deserialize(v)?),
      "CHANNEL_UPDATE" => Self::ChannelUpdate(ChannelUpdate::deserialize(v)?),
      "GUILD_BAN_ADD" => Self::BanAdd(BanAdd::deserialize(v)?),
      "GUILD_BAN_REMOVE" => Self::BanRemove(BanRemove::deserialize(v)?),
      "GUILD_CREATE" => Self::GuildCreate(Box::new(GuildCreate::deserialize(v)?)),
      "GUILD_DELETE" => Self::GuildDelete(Box::new(GuildDelete::deserialize(v)?)),
      "GUILD_EMOJIS_UPDATE" => Self::GuildEmojisUpdate(GuildEmojisUpdate::deserialize(v)?),
      "GUILD_INTEGRATIONS_UPDATE" => {
        Self::GuildIntegrationsUpdate(GuildIntegrationsUpdate::deserialize(v)?)
      }
      "GUILD_MEMBERS_CHUNK" => Self::MemberChunk(MemberChunk::deserialize(v)?),
      "GUILD_MEMBER_ADD" => Self::MemberAdd(Box::new(MemberAdd::deserialize(v)?)),
      "GUILD_MEMBER_REMOVE" => Self::MemberRemove(MemberRemove::deserialize(v)?),
      "GUILD_MEMBER_UPDATE" => Self::MemberUpdate(Box::new(MemberUpdate::deserialize(v)?)),
      "GUILD_ROLE_CREATE" => Self::RoleCreate(RoleCreate::deserialize(v)?),
      "GUILD_ROLE_DELETE" => Self::RoleDelete(RoleDelete::deserialize(v)?),
      "GUILD_ROLE_UPDATE" => Self::RoleUpdate(RoleUpdate::deserialize(v)?),
      "GUILD_UPDATE" => Self::GuildUpdate(Box::new(GuildUpdate::deserialize(v)?)),
      "INVITE_CREATE" => Self::InviteCreate(Box::new(InviteCreate::deserialize(v)?)),
      "INVITE_DELETE" => Self::InviteDelete(InviteDelete::deserialize(v)?),
      "MESSAGE_CREATE" => Self::MessageCreate(Box::new(MessageCreate::deserialize(v)?)),
      "MESSAGE_DELETE" => Self::MessageDelete(MessageDelete::deserialize(v)?),
      "MESSAGE_DELETE_BULK" => Self::MessageDeleteBulk(MessageDeleteBulk::deserialize(v)?),
      "MESSAGE_REACTION_ADD" => Self::ReactionAdd(Box::new(ReactionAdd::deserialize(v)?)),
      "MESSAGE_REACTION_REMOVE" => Self::ReactionRemove(Box::new(ReactionRemove::deserialize(v)?)),
      "MESSAGE_REACTION_REMOVE_EMOJI" => {
        Self::ReactionRemoveEmoji(ReactionRemoveEmoji::deserialize(v)?)
      }
      "MESSAGE_REACTION_REMOVE_ALL" => Self::ReactionRemoveAll(ReactionRemoveAll::deserialize(v)?),
      "MESSAGE_UPDATE" => Self::MessageUpdate(Box::new(MessageUpdate::deserialize(v)?)),
      "PRESENCE_UPDATE" => Self::PresenceUpdate(Box::new(PresenceUpdate::deserialize(v)?)),
      "PRESENCES_REPLACE" => Self::PresencesReplace,
      "READY" => Self::Ready(Box::new(Ready::deserialize(v)?)),
      "RESUMED" => Self::Resumed,
      "TYPING_START" => Self::TypingStart(Box::new(TypingStart::deserialize(v)?)),
      "USER_UPDATE" => Self::UserUpdate(UserUpdate::deserialize(v)?),
      "VOICE_SERVER_UPDATE" => Self::VoiceServerUpdate(VoiceServerUpdate::deserialize(v)?),
      "VOICE_STATE_UPDATE" => Self::VoiceStateUpdate(Box::new(VoiceStateUpdate::deserialize(v)?)),
      "WEBHOOKS_UPDATE" => Self::WebhooksUpdate(WebhooksUpdate::deserialize(v)?),

      // @TODO(vy): We should want to know what message could not be deserialized into the above types.
      _other => return Err(Box::new(Error::DeserializationError)),
    })
  }
}

#[derive(Clone, Debug)]
pub enum GatewayEvent {
  Dispatch(u64, Box<DispatchEvent>),
  Heartbeat(u64),
  HeartbeatAck,
  Hello(u64),
  InvalidateSession(bool),
  Reconnect,
}

struct GatewayEventVisitor;

impl<'de> Visitor<'de> for GatewayEventVisitor {
  type Value = GatewayEvent;

  fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
    formatter.write_str("struct GatewayEvent")
  }

  fn visit_map<V>(self, mut map: V) -> Result<GatewayEvent, V::Error>
  where
    V: MapAccess<'de>,
  {
    static VALID_OPCODES: &[&str] = &[
      "EVENT",
      "HEARTBEAT",
      "HEARTBEAT_ACK",
      "HELLO",
      "IDENTIFY",
      "INVALID_SESSION",
      "RECONNECT",
    ];

    // Have to use a serde_json::Value here because serde has no
    // abstract container type.
    let mut d = None::<Value>;
    let mut op = None::<OpCode>;
    let mut s = None::<u64>;
    let mut t = None::<String>;

    while let Some(key) = map.next_key()? {
      match key {
        Field::D => {
          if d.is_some() {
            return Err(serde::de::Error::duplicate_field("d"));
          }

          d = Some(map.next_value()?);
        }
        Field::Op => {
          if op.is_some() {
            return Err(serde::de::Error::duplicate_field("op"));
          }

          op = Some(map.next_value()?);
        }
        Field::S => {
          if s.is_some() {
            return Err(serde::de::Error::duplicate_field("s"));
          }

          s = map.next_value::<Option<_>>()?;
        }
        Field::T => {
          if t.is_some() {
            return Err(serde::de::Error::duplicate_field("t"));
          }

          t = map.next_value::<Option<_>>()?;
        }
      }
    }

    let op = op.ok_or_else(|| serde::de::Error::missing_field("op"))?;

    Ok(match op {
      OpCode::Event => {
        let d = d.ok_or_else(|| serde::de::Error::missing_field("d"))?;
        let s = s.ok_or_else(|| serde::de::Error::missing_field("s"))?;
        let t = t.ok_or_else(|| serde::de::Error::missing_field("t"))?;
        let dispatch =
          DispatchEvent::try_from((t.as_ref(), d)).map_err(serde::de::Error::custom)?;
        #[cfg(feature = "metrics")]
        counter!("DispatchEvent", 1, "DispatchEvent" => t);
        GatewayEvent::Dispatch(s, Box::new(dispatch))
      }
      OpCode::Heartbeat => {
        let s = s.ok_or_else(|| serde::de::Error::missing_field("s"))?;

        GatewayEvent::Heartbeat(s)
      }
      OpCode::HeartbeatAck => GatewayEvent::HeartbeatAck,
      OpCode::Hello => {
        #[derive(Deserialize)]
        struct Hello {
          heartbeat_interval: u64,
        }

        let d = d.ok_or_else(|| serde::de::Error::missing_field("d"))?;
        let hello = Hello::deserialize(d).map_err(serde::de::Error::custom)?;

        GatewayEvent::Hello(hello.heartbeat_interval)
      }
      OpCode::InvalidSession => {
        let d = d.ok_or_else(|| serde::de::Error::missing_field("d"))?;
        let resumeable = bool::deserialize(d).map_err(serde::de::Error::custom)?;

        GatewayEvent::InvalidateSession(resumeable)
      }
      OpCode::Identify => return Err(serde::de::Error::unknown_variant("Identify", VALID_OPCODES)),
      OpCode::Reconnect => GatewayEvent::Reconnect,
      OpCode::RequestGuildMembers => {
        return Err(serde::de::Error::unknown_variant(
          "RequestGuildMembers",
          VALID_OPCODES,
        ))
      }
      OpCode::Resume => return Err(serde::de::Error::unknown_variant("Resume", VALID_OPCODES)),
      OpCode::StatusUpdate => {
        return Err(serde::de::Error::unknown_variant(
          "StatusUpdate",
          VALID_OPCODES,
        ))
      }
      OpCode::VoiceServerPing => {
        return Err(serde::de::Error::unknown_variant(
          "VoiceServerPing",
          VALID_OPCODES,
        ))
      }
      OpCode::VoiceStateUpdate => {
        return Err(serde::de::Error::unknown_variant(
          "VoiceStateUpdate",
          VALID_OPCODES,
        ))
      }
    })
  }
}

impl<'de> Deserialize<'de> for GatewayEvent {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    const FIELDS: &[&str] = &["d", "op", "s", "t"];

    deserializer.deserialize_struct("GatewayEvent", FIELDS, GatewayEventVisitor)
  }
}
