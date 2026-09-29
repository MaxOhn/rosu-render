#![cfg(any(
    feature = "native-tls",
    feature = "rustls-native-roots",
    feature = "rustls-webpki-roots"
))]

use bytes::Bytes;

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
///
/// Verified bots that should also receive events about their private and
/// unlisted renders must call [`OrdrWebsocket::authenticate`] after
/// connecting.
pub struct OrdrWebsocket {
    engineio: EngineIo,
    reconnect: Reconnect,
    auth: Option<Box<str>>,
}

impl OrdrWebsocket {
    /// Connect to the o!rdr websocket.
    pub async fn connect() -> Result<Self, WebsocketError> {
        let engineio = EngineIo::connect().await?;

        let mut this = Self {
            engineio,
            reconnect: Reconnect::default(),
            auth: None,
        };

        this.open().await?;

        Ok(this)
    }

    /// Await the next o!rdr websocket event.
    ///
    /// This can block indefinitely when the connection is alive but no events
    /// arrive, so apply an idle timeout around it if that should count as a
    /// dead connection.
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
                // The server's `bot_auth` reply is not a render event; the
                // authentication is handled internally
                if data.starts_with(b"[\"bot_auth\"") {
                    continue;
                }

                return RawEvent::from_bytes(data);
            }
        }
    }

    /// Authenticate this connection as a verified bot.
    ///
    /// Emits the `bot_auth` event with the bot's API key and waits for the
    /// server's reply on the same event. Once authenticated, events about the
    /// bot's private and unlisted renders are included in the stream.
    ///
    /// Like [`OrdrWebsocket::next_event`], this can block indefinitely while
    /// the connection is alive. Events that arrive before the authentication
    /// reply are dropped.
    pub async fn authenticate(&mut self, key: &str) -> Result<(), WebsocketError> {
        self.auth = Some(key.into());

        self.emit_auth(key).await?;

        loop {
            let Some(bytes) = self.engineio.next_message().await? else {
                self.reconnect().await?;

                continue;
            };

            let packet = Packet::from_bytes(&bytes)?;

            match packet.kind {
                PacketKind::Event => {
                    // The server replies on the same `bot_auth` event, other
                    // events arriving during authentication are dropped.
                    let Some(data) = packet.data else {
                        continue;
                    };

                    let Some(message) = bot_auth_message(&data) else {
                        continue;
                    };

                    if message.starts_with("Authentication successful") {
                        return Ok(());
                    }

                    return Err(WebsocketError::BotAuth { message });
                }
                PacketKind::Ack => self.ack(&packet).await?,
                PacketKind::Connect => {}
                PacketKind::Disconnect | PacketKind::ConnectError => self.reconnect().await?,
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
            Ok(()) => match self.open().await {
                Ok(()) => {
                    self.reconnect.reset();

                    // The server only keeps `bot_auth` per connection, so a
                    // reconnected one has to authenticate again. The key was
                    // already verified by the initial
                    // [`OrdrWebsocket::authenticate`], so the reply here is
                    // not awaited.
                    match self.auth.clone() {
                        Some(key) => match self.emit_auth(&key).await {
                            Ok(()) => return Ok(()),
                            Err(err) => err,
                        },
                        None => return Ok(()),
                    }
                }
                Err(err) => err,
            },
            Err(err) => WebsocketError::EngineIo(err),
        };

        self.reconnect.backoff();

        Err(err)
    }

    async fn emit_auth(&mut self, key: &str) -> Result<(), WebsocketError> {
        let payload = serde_json::to_string(&["bot_auth", key])
            .expect("a &str always serializes to a JSON string");

        self.emit(Packet::new_event(Bytes::from(payload))).await
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

/// The message of a `bot_auth` reply, if the event data is one.
///
/// The server answers the `bot_auth` event with
/// `["bot_auth","Authentication successful for <bot name>"]`.
fn bot_auth_message(data: &[u8]) -> Option<Box<str>> {
    let (name, message) = serde_json::from_slice::<(String, Box<str>)>(data).ok()?;

    (name == "bot_auth").then_some(message)
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::{bot_auth_message, Packet};

    #[test]
    fn bot_auth_emit_frame() {
        let payload = serde_json::to_string(&["bot_auth", "secret-key"]).unwrap();

        let frame = Packet::new_event(Bytes::from(payload)).to_bytes();

        assert_eq!(frame.as_ref(), &br#"2["bot_auth","secret-key"]"#[..]);
    }

    #[test]
    fn bot_auth_reply_frame() {
        let frame = br#"2["bot_auth","Authentication successful for bathbot"]"#;

        let packet = Packet::from_bytes(&Bytes::from_static(frame)).unwrap();
        let data = packet.data.unwrap();

        assert_eq!(
            bot_auth_message(&data),
            Some("Authentication successful for bathbot".into()),
        );
    }

    #[test]
    fn bot_auth_failure_message_is_captured() {
        assert_eq!(
            bot_auth_message(br#"["bot_auth","Invalid API key"]"#),
            Some("Invalid API key".into()),
        );
    }

    #[test]
    fn bot_auth_ignores_other_events() {
        let data = br#"["render_done_json",{"renderID":1}]"#;

        assert_eq!(bot_auth_message(data), None);
    }
}
