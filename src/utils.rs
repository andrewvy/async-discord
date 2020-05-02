use async_native_tls::TlsStream;
use async_std::net::TcpStream;
use async_tungstenite::{stream::Stream, WebSocketStream};

/// Helper type that represents a websocket stream that is optionally wrapped in TLS.
pub type WebsocketStream = WebSocketStream<Stream<TcpStream, TlsStream<TcpStream>>>;
