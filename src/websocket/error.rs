use std::str::Utf8Error;

use bytes::Bytes;
use serde_json::Error as SerdeError;
use thiserror::Error as ThisError;

use crate::websocket::engineio::error::EngineIoError;

#[derive(Debug, ThisError)]
pub enum WebsocketError {
    #[error("Failed to deserialize data={data:?}")]
    Deserialize {
        #[source]
        source: SerdeError,
        data: Bytes,
    },
    #[error("engine.io error")]
    EngineIo(#[from] EngineIoError),
    #[error("The websocket packet contained an invalid o!rdr event payload=\"{0:?}\"")]
    InvalidEvent(Bytes),
    #[error("Invalid packet id {0}")]
    InvalidPacketId(char),
    #[error("Got an invalid packet which did not follow the protocol format")]
    InvalidPacket,
    #[error("Failed to decode binary as UTF-8")]
    InvalidUtf8(#[from] Utf8Error),
}
