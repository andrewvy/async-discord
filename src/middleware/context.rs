/// Context-wrapped event that provides some niceties for storing
/// additional data for the duration of the event as it moves through
/// the middleware stack.
pub struct Context<Event> {
  /// Wrapped event.
  pub event: Event,
}

impl<Event> Context<Event> {
  pub fn new(event: Event) -> Context<Event> {
    Self { event }
  }
}
