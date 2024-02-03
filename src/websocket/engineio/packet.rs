use std::time::Duration;

use bytes::Bytes;
use serde::Deserialize;

use super::error::EngineIoError;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum PacketId {
    Open = 0,
    Close = 1,
    Ping = 2,
    Pong = 3,
    Message = 4,
    Upgrade = 5,
}

impl PacketId {
    /// Returns the byte that represents the [`PacketId`] as a [`char`].
    fn to_string_byte(self) -> u8 {
        self as u8 + b'0'
    }
}

impl TryFrom<u8> for PacketId {
    type Error = EngineIoError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'0' => Ok(Self::Open),
            b'1' => Ok(Self::Close),
            b'2' => Ok(Self::Ping),
            b'3' => Ok(Self::Pong),
            b'4' => Ok(Self::Message),
            b'5' => Ok(Self::Upgrade),
            _ => Err(EngineIoError::InvalidPacketId(value)),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Packet {
    pub packet_id: PacketId,
    pub data: Bytes,
}

impl Packet {
    /// Creates a new [`Packet`].
    pub(crate) fn new(packet_id: PacketId, data: Bytes) -> Self {
        Packet { packet_id, data }
    }

    /// Encodes a [`Packet`] into a byte vec.
    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(1 + self.data.len());
        bytes.push(self.packet_id.to_string_byte());
        bytes.extend_from_slice(self.data.as_ref());

        bytes
    }

    pub(crate) fn from_bytes(bytes: &Bytes) -> Result<Self, EngineIoError> {
        let packet_id: PacketId = bytes
            .first()
            .copied()
            .ok_or(EngineIoError::IncompletePacket)?
            .try_into()?;

        if bytes.len() == 1 && packet_id == PacketId::Message {
            return Err(EngineIoError::IncompletePacket);
        }

        let data: Bytes = bytes.slice(1..);

        Ok(Self { packet_id, data })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct HandshakePacket {
    pub ping_interval: u64,
    pub ping_timeout: u64,
}

impl HandshakePacket {
    pub(super) fn heartbeat_interval(&self) -> Duration {
        Duration::from_millis(self.ping_interval + self.ping_timeout)
    }
}
