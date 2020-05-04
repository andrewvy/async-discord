use std::convert::TryInto;
use std::env::consts;
use std::time::{Duration, Instant};

use async_std::sync::Arc;
use async_tungstenite::tungstenite::Message as WebsocketMessage;
use flate2::{Decompress, DecompressError, FlushDecompress};
use futures::prelude::*;
use futures::{
  channel::mpsc::{channel, Receiver, Sender},
  stream::{SplitSink, SplitStream},
};
use log::debug;
use serde_json::json;
use twilight_model::gateway::OpCode;

use super::event::{DispatchEvent, GatewayEvent};
use super::session::Session;
use super::session_state::SessionState;

use crate::client::Client;
use crate::utils::WebsocketStream;

const ZLIB_SUFFIX: [u8; 4] = [0x00, 0x00, 0xff, 0xff];
const BUFFER_SIZE: usize = 16 * 1024;

/// Simple buffer struct that consumes input, and you can check if
/// the current buffer ends with a ZLIB_SUFFIX marker.
struct ZlibBuffer {
  /// Buffer of compressed data.
  buffer: Vec<u8>,

  /// Internal buffer that holds temporary decoded chunks from the decompressor.
  decode_buffer: Vec<u8>,

  /// Output buffer that holds uncompressed content.
  output_buffer: Vec<u8>,

  /// Zlib context.
  decompressor: Decompress,
}

impl ZlibBuffer {
  pub fn new() -> Self {
    Self {
      buffer: Vec::with_capacity(BUFFER_SIZE),
      decode_buffer: Vec::with_capacity(BUFFER_SIZE),
      output_buffer: Vec::with_capacity(BUFFER_SIZE),
      decompressor: Decompress::new(true),
    }
  }

  pub fn extend(&mut self, slice: &[u8]) {
    self.buffer.extend_from_slice(&slice);
  }

  pub fn consume(&mut self) -> Result<Option<&[u8]>, DecompressError> {
    let len = self.buffer.len();

    if len >= 4 && self.buffer[(len - 4)..] == ZLIB_SUFFIX {
      let begin = self.decompressor.total_in();
      let mut offset = 0;

      // We want to continually decompress until we have consumed all the input
      // from `self.buffer`.
      loop {
        self.decode_buffer.clear();

        self.decompressor.decompress_vec(
          &self.buffer[offset..],
          &mut self.decode_buffer,
          FlushDecompress::Sync,
        )?;

        offset = (self.decompressor.total_in() - begin)
          .try_into()
          .unwrap_or(0);

        self
          .output_buffer
          .extend_from_slice(&self.decode_buffer[..]);

        if offset >= self.buffer.len() {
          break;
        }
      }

      Ok(Some(&self.output_buffer))
    } else {
      Ok(None)
    }
  }

  pub fn clear(&mut self) {
    self.buffer.clear();
    self.decode_buffer.clear();
    self.output_buffer.clear();
  }
}

// Any gateway event could trigger these internal actions that we should act upon.
#[derive(Debug)]
enum ProcessorAction {
  ShouldReconnect,
  ShouldResume,
  ShouldHeartbeat,
  ShouldIdentify,
}

/// Listens on the websocket events, handles any session-related events, and forwards messages upstream.
pub(crate) struct ShardProcessor {
  client: Arc<Client>,
  decoder: ZlibBuffer,

  /// Receiving-end of the direct WebsocketStream.
  rx: SplitStream<WebsocketStream>,

  /// A sender that queues any messages we want to forward from this processor to the downstream client consumer.
  queue: Sender<Box<DispatchEvent>>,

  /// A receiver for the above sender, primarily used by any downstream consumers of dispatch events.
  receiver: Receiver<Box<DispatchEvent>>,

  /// A sender that passes messages directly upstream through the websocket connection.
  sender: SplitSink<WebsocketStream, WebsocketMessage>,

  /// Shared session state.
  session: Arc<Session>,

  /// The interval of the heartbeat in ms.
  heartbeat_interval: u64,

  /// The last time we sent a heartbeat to the gateway.
  last_heartbeat: Option<Instant>,

  /// The last time we received a heartbeat ack from the gateway.
  last_heartbeat_ack: Option<Instant>,

  /// Was our last recent heartbeat acknowledged by the gateway?
  was_heartbeat_acknowledged: bool,

  /// Discord event sequence number, will be used for resuming + heartbeats.
  seq: u64,
}

impl ShardProcessor {
  pub(crate) fn new(
    client: Arc<Client>,
    websocket: WebsocketStream,
    session: Arc<Session>,
  ) -> Self {
    let (tx, rx) = websocket.split();

    // @TODO(vy): Configurable forward channel sizes.
    let (to_client, gateway_to_client) = channel(100);

    Self {
      rx,
      client,
      decoder: ZlibBuffer::new(),
      queue: to_client,
      receiver: gateway_to_client,
      sender: tx,
      session,
      heartbeat_interval: 15_000,
      last_heartbeat_ack: None,
      last_heartbeat: Some(Instant::now()),
      was_heartbeat_acknowledged: true,
      seq: 0,
    }
  }

  /// The main process loop of the ShardProcessor.
  /// - Gets the next [`GatewayEvent`] from the websocket.
  /// - Processes the [`GatewayEvent`], optionally returning a [`ProcessorAction`].
  /// - Processes the [`ProcessorAction`].
  pub(crate) async fn process(mut self) {
    loop {
      if let Some(processor_action) = self.check_internal_timers() {
        self.process_processor_action(processor_action).await;
      }

      if let Some(gateway_event) = self.next().await {
        if let Some(processor_action) = self.process_gateway_event(gateway_event).await {
          self.process_processor_action(processor_action).await;
        }
      }
    }
  }

  fn check_internal_timers(&mut self) -> Option<ProcessorAction> {
    if let Some(last_heartbeat) = self.last_heartbeat {
      let interval = Duration::from_millis(self.heartbeat_interval);

      // Check if we should heartbeat again based on the heartbeat_interval.
      if last_heartbeat.elapsed() > interval {
        if !self.was_heartbeat_acknowledged {
          return Some(ProcessorAction::ShouldReconnect);
        } else {
          return Some(ProcessorAction::ShouldHeartbeat);
        }
      }
    }

    None
  }

  async fn process_processor_action(&mut self, processor_action: ProcessorAction) {
    match processor_action {
      ProcessorAction::ShouldHeartbeat => {
        let heartbeat = json!({
          "op": OpCode::Heartbeat,
          "d": self.seq
        });

        self.was_heartbeat_acknowledged = false;
        self.last_heartbeat = Some(Instant::now());

        if let Ok(msg) = serde_json::to_string(&heartbeat).map(WebsocketMessage::Text) {
          let _ = self.sender.send(msg).await;
        }
      }
      ProcessorAction::ShouldIdentify => {
        let identify = json!({
          "op": OpCode::Identify,
          "d": {
            "token": self.client.token,
            "properties": {
              "$os": consts::OS,
              "$browser": "andrewvy/async-discord",
              "$device": "andrewvy/async-discord"
            },
            "guild_subscriptions": self.client.gateway_guild_subscriptions
          }
        });

        if let Ok(msg) = serde_json::to_string(&identify).map(WebsocketMessage::Text) {
          let _ = self.sender.send(msg).await;
        }
      }
      ProcessorAction::ShouldReconnect => {}
      ProcessorAction::ShouldResume => {}
    }
  }

  /// Processes a gateway event, and optionally returns an action that we should act upon.
  async fn process_gateway_event(
    &mut self,
    gateway_event: GatewayEvent,
  ) -> Option<ProcessorAction> {
    match gateway_event {
      GatewayEvent::Dispatch(seq, event) => {
        // @TODO(vy): Should check for out-of-sequence here.
        self.seq = seq;

        // @TODO(vy): This could fail if the queue is full.
        let _ = self.queue.send(event).await;
      }
      GatewayEvent::Hello(heartbeat_interval) => {
        if self.session.get_state() == SessionState::Resuming {
          self.heartbeat_interval = heartbeat_interval;
        } else {
          self.session.set_state(SessionState::Identifying);
          self.heartbeat_interval = heartbeat_interval;
          return Some(ProcessorAction::ShouldIdentify);
        }
      }
      GatewayEvent::HeartbeatAck => {
        self.last_heartbeat_ack = Some(Instant::now());
        self.was_heartbeat_acknowledged = true;
      }
      _ => {}
    }

    None
  }

  async fn next(&mut self) -> Option<GatewayEvent> {
    let timeout = async_std::future::timeout(Duration::from_millis(100), self.rx.next());

    if let Ok(payload) = timeout.await {
      if let Some(Ok(msg)) = payload {
        match msg {
          WebsocketMessage::Text(text) => {
            let event: Option<GatewayEvent> = serde_json::from_str(&text)
              .map_err(|err| {
                debug!(
                  "[processor] Deserialize error occured with text: {:?}; text: {:?}",
                  err, text
                );

                err
              })
              .ok();

            return event;
          }
          WebsocketMessage::Binary(bytes) => {
            self.decoder.extend(&bytes);

            if let Ok(Some(compressed_msg)) = self.decoder.consume() {
              let event: Option<GatewayEvent> = serde_json::from_reader(compressed_msg)
                .map_err(|err| {
                  debug!(
                    "[processor] Deserialize error occured with byte: {:?}; bytes: {:?}",
                    err, bytes
                  );

                  self.decoder.clear();

                  err
                })
                .ok();

              self.decoder.clear();

              return event;
            }
          }
          WebsocketMessage::Ping(_) => return None,
          WebsocketMessage::Pong(_) => return None,
          WebsocketMessage::Close(_) => return None,
        }
      }
    }

    None
  }
}
