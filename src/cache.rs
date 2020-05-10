use std::collections::HashMap;

use async_std::sync::RwLock;
use futures::future;
use twilight_model::{
  channel::GuildChannel,
  guild::Guild,
  id::{GuildId, UserId},
  user::{CurrentUser, User},
};

/// Simple caching mechanism that processes DispatchEvents, and can be later retrieved.
#[derive(Debug)]
pub struct Cache {
  /// The current bot user.
  pub me: RwLock<Option<CurrentUser>>,

  /// All cached guilds by GuildId.
  pub guilds: RwLock<HashMap<GuildId, Guild>>,

  /// All guild channels by ChannelId.
  pub channels_guild: RwLock<HashMap<GuildId, RwLock<Vec<GuildChannel>>>>,

  /// All users by UserId.
  pub users: RwLock<HashMap<UserId, User>>,
}

impl Cache {
  pub fn new() -> Self {
    Self {
      me: RwLock::new(None),
      guilds: RwLock::new(HashMap::new()),
      channels_guild: RwLock::new(HashMap::new()),
      users: RwLock::new(HashMap::new()),
    }
  }

  pub async fn cache_me(&self, me: CurrentUser) {
    let mut current_user = self.me.write().await;
    *current_user = Some(me);
  }

  pub async fn cache_guild(&self, guild: Guild) {
    let guild_id = guild.id;

    let _ = future::join_all(
      guild
        .channels
        .into_iter()
        .map(|(_, channel)| self.cache_guild_channel(guild_id, channel)),
    )
    .await;

    let _ = future::join_all(
      guild
        .members
        .into_iter()
        .map(|(_, member)| self.cache_user(member.user)),
    )
    .await;
  }

  async fn cache_user(&self, user: User) {
    let mut users = self.users.write().await;

    users.insert(user.id, user);
  }

  async fn cache_guild_channel(&self, guild_id: GuildId, guild_channel: GuildChannel) {
    let mut channels_guild = self.channels_guild.write().await;
    let channels = channels_guild
      .entry(guild_id)
      .or_insert(RwLock::new(Vec::new()));
    channels.write().await.push(guild_channel);
  }
}
