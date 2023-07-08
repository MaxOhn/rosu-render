#![cfg(any(
    feature = "native",
    feature = "rustls-native-roots",
    feature = "rustls-webpki-roots"
))]

use crate::WebsocketError;

use self::{
    engineio::{
        packet::{Packet as EnginePacket, PacketId as EnginePacketId},
        EngineIo,
    },
    event::RawEvent,
    packet::{Packet, PacketKind},
    reconnect::Reconnect,
};

mod engineio;
mod packet;
mod reconnect;

pub mod error;
pub mod event;

/// Connection to the o!rdr websocket.
///
/// Await events with [`OrdrWebsocket::next_event`].
///
/// To gracefully shut the connection down, use [`OrdrWebsocket::disconnect`].
pub struct OrdrWebsocket {
    engineio: EngineIo,
    reconnect: Reconnect,
}

impl OrdrWebsocket {
    /// Connect to the o!rdr websocket.
    pub async fn connect() -> Result<Self, WebsocketError> {
        let engineio = EngineIo::connect().await?;

        let mut this = Self {
            engineio,
            reconnect: Reconnect::default(),
        };

        this.open().await?;

        Ok(this)
    }

    /// Await the next o!rdr websocket event.
    pub async fn next_event(&mut self) -> Result<RawEvent, WebsocketError> {
        loop {
            let Some(bytes) = self.engineio.next_message().await? else {
                self.reconnect().await?;

                continue;
            };

            let packet = Packet::from_bytes(&bytes)?;

            match packet.kind {
                PacketKind::Event => {}
                PacketKind::Ack => self.ack(&packet).await?,
                PacketKind::Connect => continue,
                PacketKind::Disconnect | PacketKind::ConnectError => {
                    self.reconnect().await?;

                    continue;
                }
            }

            if let Some(data) = packet.data {
                return RawEvent::from_bytes(data);
            }
        }
    }

    /// Gracefully disconnect from the websocket.
    pub async fn disconnect(self) -> Result<(), WebsocketError> {
        self.engineio
            .disconnect()
            .await
            .map_err(WebsocketError::EngineIo)
    }

    async fn reconnect(&mut self) -> Result<(), WebsocketError> {
        if let Some(delay) = self.reconnect.delay() {
            trace!(?delay, "Delaying reconnect...");
            tokio::time::sleep(delay).await;
        }

        let err = match self.engineio.reconnect().await {
            Ok(_) => match self.open().await {
                Ok(_) => return Ok(()),
                Err(err) => err,
            },
            Err(err) => WebsocketError::EngineIo(err),
        };

        self.reconnect.backoff();

        Err(err)
    }

    async fn emit(&mut self, packet: Packet) -> Result<(), WebsocketError> {
        let msg = EnginePacket::new(EnginePacketId::Message, packet.to_bytes());

        self.engineio
            .emit(msg)
            .await
            .map_err(WebsocketError::EngineIo)
    }

    async fn open(&mut self) -> Result<(), WebsocketError> {
        self.emit(Packet::new(PacketKind::Connect, None)).await
    }

    async fn ack(&mut self, packet: &Packet) -> Result<(), WebsocketError> {
        let Some(id) = packet.id else { return Ok(()) };

        self.emit(Packet::new_ack(id)).await
    }
}
