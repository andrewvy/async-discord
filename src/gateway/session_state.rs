use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum SessionState {
  Disconnected,
  Handshaking,
  Identifying,
  Connected,
  Resuming,
}

#[derive(Clone, Debug)]
pub enum SessionStateConversionError {
  InvalidInteger { value: u8 },
}

impl Display for SessionStateConversionError {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    match self {
      Self::InvalidInteger { value } => {
        write!(f, "The integer {} is not a valid SessionState.", value)
      }
    }
  }
}

impl Error for SessionStateConversionError {}

impl Default for SessionState {
  fn default() -> Self {
    Self::Disconnected
  }
}

impl Display for SessionState {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    f.write_str(match self {
      Self::Disconnected => "Disconnected",
      Self::Handshaking => "Handshaking",
      Self::Identifying => "Identifying",
      Self::Connected => "Connected",
      Self::Resuming => "Resuming",
    })
  }
}

impl TryFrom<u8> for SessionState {
  type Error = SessionStateConversionError;

  fn try_from(num: u8) -> Result<Self, Self::Error> {
    Ok(match num {
      0 => Self::Disconnected,
      1 => Self::Handshaking,
      2 => Self::Identifying,
      3 => Self::Connected,
      4 => Self::Resuming,
      unknown => return Err(SessionStateConversionError::InvalidInteger { value: unknown }),
    })
  }
}
