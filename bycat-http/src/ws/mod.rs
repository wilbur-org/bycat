mod callback;
mod compat;
mod error;
mod get_stream;
mod handshake;
mod stream;
mod upgrade;
mod websocket;

pub use self::{
    error::WebsocketError,
    upgrade::{OnFailedUpgrade, WebSocketUpgrade},
    websocket::WebSocket,
};

pub use tungstenite::protocol::{CloseFrame, Message, frame::coding::CloseCode};
