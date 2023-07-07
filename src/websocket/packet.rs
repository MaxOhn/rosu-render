use std::str::from_utf8 as str_from_utf8;

use bytes::{BufMut, Bytes, BytesMut};
use itoa::Buffer;

use super::error::WebsocketError;

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub(super) enum PacketKind {
    Connect = 0,
    Disconnect = 1,
    Event = 2,
    Ack = 3,
    ConnectError = 4,
}

impl TryFrom<char> for PacketKind {
    type Error = WebsocketError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '0' => Ok(PacketKind::Connect),
            '1' => Ok(PacketKind::Disconnect),
            '2' => Ok(PacketKind::Event),
            '3' => Ok(PacketKind::Ack),
            '4' => Ok(PacketKind::ConnectError),
            _ => Err(WebsocketError::InvalidPacketId(value)),
        }
    }
}

#[derive(Debug)]
pub(super) struct Packet {
    pub kind: PacketKind,
    pub data: Option<Bytes>,
    pub id: Option<i32>,
}

impl Default for Packet {
    fn default() -> Self {
        Self {
            kind: PacketKind::Event,
            data: None,
            id: None,
        }
    }
}

impl Packet {
    pub(super) fn new(kind: PacketKind, id: Option<i32>) -> Self {
        Self {
            kind,
            data: None,
            id,
        }
    }

    pub(super) fn new_ack(id: i32) -> Self {
        Self {
            kind: PacketKind::Ack,
            data: Some(Bytes::from_static(b"[]")),
            id: Some(id),
        }
    }

    pub(super) fn to_bytes(&self) -> Bytes {
        let mut bytes = BytesMut::new();
        bytes.put_u8(self.kind as u8 + b'0');

        if let Some(id) = self.id {
            let mut itoa_buf = Buffer::new();
            bytes.extend_from_slice(itoa_buf.format(id).as_bytes());
        }

        if let Some(data) = self.data.as_ref() {
            bytes.extend_from_slice(data);
        }

        bytes.freeze()
    }

    pub(super) fn from_bytes(bytes: &Bytes) -> Result<Self, WebsocketError> {
        let mut payload = str_from_utf8(bytes).map_err(WebsocketError::InvalidUtf8)?;
        let mut packet = Packet::default();

        let id_char = payload
            .chars()
            .next()
            .ok_or(WebsocketError::InvalidPacket)?;

        packet.kind = PacketKind::try_from(id_char)?;
        payload = &payload[id_char.len_utf8()..];

        if payload.starts_with('/') {
            let (_, rest) = payload
                .split_once(',')
                .ok_or(WebsocketError::InvalidPacket)?;

            payload = rest;
        }

        let Some((non_digit_idx, _)) = payload.char_indices().find(|(_, c)| !c.is_ascii_digit()) else {
            return Ok(packet);
        };

        if non_digit_idx > 0 {
            let (prefix, rest) = payload.split_at(non_digit_idx);
            payload = rest;
            packet.id = Some(prefix.parse().map_err(|_| WebsocketError::InvalidPacket)?);
        }

        packet.data = Some(bytes.slice_ref(payload.as_bytes()));

        Ok(packet)
    }
}
