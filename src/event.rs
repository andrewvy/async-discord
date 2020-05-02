use async_std::sync::Arc;

/// A Discord Event.
///
/// `Event` gives middleware/handlers access to information about the underlying discord event,
/// as well as functionality to access the data easier.
///
/// Events contains `State` which can be used by middleware/handlers to store information that can
/// be communicated between handlers / different events.
pub struct Event<State> {
  state: Arc<State>,
}

impl<State> Event<State> {
  pub(crate) fn new(state: Arc<State>) -> Event<State> {
    Event { state }
  }
}
