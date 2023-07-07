use std::{error::Error as StdError, str::Utf8Error};

use bytes::Bytes;
use hyper::Error as HyperError;
use serde_json::Error as SerdeError;
use thiserror::Error as ThisError;
use tokio_tungstenite::tungstenite::{Error as TungsteniteError, Message};

#[derive(Debug, ThisError)]
pub enum EngineIoError {
    #[error("Failed to chunk response")]
    ChunkingResponse(#[source] HyperError),
    #[error("Failed to deserialize data={data:?}")]
    Deserialize {
        #[source]
        source: SerdeError,
        data: Bytes,
    },
    #[error("Server timed out while attempting to handshake")]
    HandshakeTimeout,
    #[error("Server did not fulfill its heartbeat obligation")]
    HeartbeatTimeout,
    #[error("Incomplete package")]
    IncompletePacket,
    #[error("Received invalid handshake response: {0:?}")]
    InvalidHandshake(Message),
    #[error("Failed to decode binary as UTF-8")]
    InvalidUtf8(#[from] Utf8Error),
    #[error("Invalid packet id {0}")]
    InvalidPacketId(u8),
    #[error("Failed to load the TLS connector or its certificates")]
    LoadingTls(#[source] Box<dyn StdError + Send + Sync>),
    #[error("Failed to reconnect websocket")]
    Reconnect(#[source] TungsteniteError),
    #[error("Failed to receive response")]
    ReceiveResponse(#[source] HyperError),
    #[error("Failed to upgrade websocket reason=\"{reason}\"")]
    WebsocketUpgrade { reason: &'static str },
    #[error("Failed to receive message from websocket")]
    WebsocketReceive(#[source] TungsteniteError),
    #[error("Failed to send message through websocket")]
    WebsocketSend(#[source] TungsteniteError),
}
