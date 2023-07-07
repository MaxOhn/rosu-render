pub mod error;

pub(crate) mod packet;
pub(crate) mod socket;
pub(crate) mod tls;

use bytes::Bytes;

use crate::websocket::engineio::packet::Packet;

use self::{error::EngineIoError, packet::PacketId, socket::Socket};

pub(crate) struct EngineIo {
    socket: Socket,
}

impl EngineIo {
    pub(crate) async fn connect() -> Result<Self, EngineIoError> {
        Socket::new().await.map(|socket| Self { socket })
    }

    pub(crate) async fn next_message(&mut self) -> Result<Option<Bytes>, EngineIoError> {
        loop {
            match self.socket.next_packet().await? {
                Some(packet) => match packet.packet_id {
                    PacketId::Message => return Ok(Some(packet.data)),
                    PacketId::Close => return Ok(None),
                    PacketId::Ping => self.socket.pong().await?,
                    PacketId::Open | PacketId::Pong | PacketId::Upgrade => {}
                },
                None => return Ok(None),
            }
        }
    }

    pub(crate) async fn emit(&mut self, packet: Packet) -> Result<(), EngineIoError> {
        self.socket.emit(packet).await
    }

    pub(crate) async fn disconnect(self) -> Result<(), EngineIoError> {
        self.socket.disconnect().await
    }

    pub(crate) async fn reconnect(&mut self) -> Result<(), EngineIoError> {
        trace!("Reconnecting engine.io");
        self.socket = Socket::new().await?;

        Ok(())
    }
}
