/*
 * Interceptors:
 * - Creates and sends an interceptor to the Gateway.
 * - It is added to the list of interceptors.
 * - When an event matches the description of the interceptor:
 *   - Send to the interceptor TX.
 * - Interceptor keeps awaiting events until fulfilled.
 * - Once Interceptor is complete, close the RX/TX.
 * - Gateway when trying to send event, if the TX is closed
 *   - We can remove the interceptor from the list of interceptors.
 */

use std::sync::Arc;

use futures::channel::mpsc::{channel, Receiver, Sender};

use crate::gateway::event::{DispatchEvent, GatewayAction};
use crate::gateway::GatewayActionSender;
use twilight_model::id::{ChannelId, GuildId, MessageId};

#[derive(Debug)]
pub enum FilterEvent {
  Reaction,
}

#[derive(Debug)]
pub struct Filter {
  guild_id: Option<GuildId>,
  channel_id: Option<ChannelId>,
  message_id: Option<MessageId>,
  event_type: FilterEvent,
  max: usize,
}

impl Filter {
  pub fn new(event_type: FilterEvent) -> Self {
    Self {
      guild_id: None,
      channel_id: None,
      message_id: None,
      max: 1,
      event_type,
    }
  }
  fn matches(&self, event: &DispatchEvent) -> bool {
    match self.event_type {
      FilterEvent::Reaction => matches!(event, DispatchEvent::ReactionAdd { .. }),
    }
  }
}

#[derive(Debug)]
pub struct Interceptor {
  tx: Sender<Arc<DispatchEvent>>,
  filter: Filter,
  current: usize,
}

impl Interceptor {
  pub fn matches(&self, event: &DispatchEvent) -> bool {
    self.filter.matches(event)
  }

  /// Sends a message through to the interceptor.
  ///
  /// Returns true if accepted, false if rejected.
  pub fn send(&mut self, event: Arc<DispatchEvent>) -> bool {
    if self.current > self.filter.max {
      return false;
    }

    if let Err(_) = self.tx.try_send(event) {
      return false;
    }

    self.current += 1;

    true
  }
}

pub struct ReactionFilter {
  pub rx: Receiver<Arc<DispatchEvent>>,
}

impl ReactionFilter {
  pub fn new(gateway: &mut GatewayActionSender) -> Self {
    let (tx, rx) = channel(100);

    let interceptor = Interceptor {
      tx,
      filter: Filter::new(FilterEvent::Reaction),
      current: 0,
    };

    let _ = gateway.try_send(Box::new(GatewayAction::AddInterceptor(interceptor)));

    Self { rx }
  }
}
