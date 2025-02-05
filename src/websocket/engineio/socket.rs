use std::time::Duration;

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use tokio::{net::TcpStream, time::Instant};
use tokio_websockets::{ClientBuilder, Connector, Limits, MaybeTlsStream, Message};
use url::Url;

use crate::websocket::engineio::packet::{Packet, PacketId};

use super::{error::EngineIoError, packet::HandshakePacket};

const WS_URL: &str = "https://apis.issou.best";
const WS_PATH: &str = "/ordr/ws/";
const ENGINE_IO_VERSION: &str = "4";

/// [`tokio_websockets`] library Websocket connection.
type Connection = tokio_websockets::WebSocketStream<MaybeTlsStream<TcpStream>>;

pub(super) struct Socket {
    connection: Connection,
    heartbeat_interval: Duration,
    last_heartbeat: Instant,
}

impl Socket {
    pub(super) async fn new() -> Result<Self, EngineIoError> {
        let mut url = Url::parse(WS_URL).expect("WS_URL is valid url");
        url.set_path(WS_PATH);
        url.query_pairs_mut().append_pair("EIO", ENGINE_IO_VERSION);

        let timeout = Duration::from_secs(30);
        let handshake_fut = Self::handshake(url);

        let (connection, handshake) = tokio::time::timeout(timeout, handshake_fut)
            .await
            .map_err(|_| EngineIoError::HandshakeTimeout)??;

        Ok(Self {
            connection,
            heartbeat_interval: handshake.heartbeat_interval(),
            last_heartbeat: Instant::now(),
        })
    }

    async fn handshake(mut url: Url) -> Result<(Connection, HandshakePacket), EngineIoError> {
        url.query_pairs_mut().append_pair("transport", "websocket");
        url.set_scheme("wss").expect("wss is valid scheme");

        let tls = Connector::new().unwrap();

        let (mut connection, _) = ClientBuilder::new()
            .uri(url.as_str())
            .unwrap()
            .limits(Limits::unlimited())
            .connector(&tls)
            .connect()
            .await
            .map_err(EngineIoError::Reconnect)?;

        let msg = connection
            .next()
            .await
            .expect("websocket is open at this point")
            .map_err(EngineIoError::WebsocketReceive)?;

        let bytes = Bytes::from(msg.into_payload());
        let Packet { data, .. } = Packet::from_bytes(&bytes)?;

        let handshake: HandshakePacket = serde_json::from_slice(&data)
            .map_err(|source| EngineIoError::Deserialize { source, data })?;

        trace!(?handshake, "Handshake successful");

        Ok((connection, handshake))
    }

    pub(super) async fn next_packet(&mut self) -> Result<Option<Packet>, EngineIoError> {
        let timeout = self.heartbeat_deadline();

        let message = match tokio::time::timeout_at(timeout, self.connection.next()).await {
            Ok(Some(message)) => message,
            Ok(None) => return Ok(None),
            Err(_) => {
                trace!(
                    interval = ?self.heartbeat_interval,
                    since_last_heartbeat = ?self.last_heartbeat.elapsed(),
                    "Heartbeat timed out",
                );

                return Ok(None);
            }
        };

        trace!(?message, "Websocket message");

        let message = message.map_err(EngineIoError::WebsocketReceive)?;

        if message.is_close() {
            return Ok(None);
        }

        let bytes = Bytes::from(message.into_payload());

        Packet::from_bytes(&bytes).map(Some)
    }

    pub(super) async fn emit(&mut self, packet: Packet) -> Result<(), EngineIoError> {
        Self::emit_static(&mut self.connection, packet).await
    }

    pub(super) async fn pong(&mut self) -> Result<(), EngineIoError> {
        self.last_heartbeat = Instant::now();

        self.emit(Packet::new(PacketId::Pong, Bytes::new())).await
    }

    pub(super) async fn disconnect(mut self) -> Result<(), EngineIoError> {
        self.emit(Packet::new(PacketId::Close, Bytes::new())).await
    }

    fn heartbeat_deadline(&self) -> Instant {
        self.last_heartbeat + self.heartbeat_interval
    }

    async fn emit_static(connection: &mut Connection, packet: Packet) -> Result<(), EngineIoError> {
        let msg = String::from_utf8(packet.to_bytes())
            .map(Message::text)
            .map_err(|err| EngineIoError::InvalidUtf8(err.utf8_error()))?;

        trace!("Emitting packet {packet:?}");

        connection
            .send(msg)
            .await
            .map_err(EngineIoError::WebsocketSend)
    }
}
