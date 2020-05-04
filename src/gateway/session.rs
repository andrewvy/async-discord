use std::convert::TryFrom;
use std::sync::atomic::{AtomicU8, Ordering};

use super::session_state::SessionState;

pub(crate) struct Session {
  state: AtomicU8,
}

impl Session {
  pub(crate) fn new() -> Self {
    Self {
      state: AtomicU8::new(SessionState::Disconnected as u8),
    }
  }

  pub(crate) fn get_state(&self) -> SessionState {
    SessionState::try_from(self.state.load(Ordering::Relaxed)).unwrap_or_default()
  }

  pub(crate) fn set_state(&self, state: SessionState) {
    self.state.store(state as u8, Ordering::Release);
  }
}
