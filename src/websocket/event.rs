use bytes::Bytes;
use serde_json::Error as SerdeError;

use crate::model::{
    CustomSkinProcessUpdate, Event, RenderAdded, RenderDone, RenderFailed, RenderProgress,
};

/// Websocket [`Event`] that has not been fully deserialized yet.
/// This lets you check if you're interested in the event and only then deserialize it.
///
/// # Example
/// ```rust
/// use std::collections::HashSet;
/// use rosu_render::websocket::event::RawEvent;
///
/// fn handle_event(event: RawEvent, interesting_render_ids: &HashSet<u32>) {
///     match event {
///         RawEvent::RenderDone(event) if interesting_render_ids.contains(&event.render_id) => {
///             match event.deserialize() {
///                 Ok(done) => println!("{done:?}"),
///                 Err(err) => println!("Failed to deserialize {event:?} {err:?}"),
///             }
///         }
///         _ => {} // Ignore
///     }
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawEvent {
    RenderAdded(RawRenderAdded),
    RenderDone(RawRenderDone),
    RenderFailed(RawRenderFailed),
    RenderProgress(RawRenderProgress),
    CustomSkinProcessUpdate(RawCustomSkinProcessUpdate),
}

impl RawEvent {
    pub(crate) fn from_bytes(bytes: Bytes) -> Result<Self, crate::WebsocketError> {
        fn split_bytes(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
            let comma_idx = bytes.iter().position(|&byte| byte == b',')?;

            let ([b'[', b'"', prefix @ .., b'"'], [_, suffix @ .., b']']) =
                bytes.split_at(comma_idx)
            else {
                return None;
            };

            Some((prefix, suffix))
        }

        fn find_render_id(mut bytes: &[u8]) -> Option<u32> {
            loop {
                let idx = bytes.iter().position(|&byte| byte == b'r')?;

                let Some(rest) = bytes[idx..].strip_prefix(b"renderID\":") else {
                    bytes = &bytes[idx + 1..];

                    continue;
                };

                rest.first().copied().filter(u8::is_ascii_digit)?;

                let render_id = rest
                    .iter()
                    .copied()
                    .take_while(u8::is_ascii_digit)
                    .fold(0, |num, byte| num * 10 + u32::from(byte & 0xF));

                return Some(render_id);
            }
        }

        let Some((event, payload)) = split_bytes(&bytes) else {
            return Err(crate::WebsocketError::InvalidEvent(bytes));
        };

        let payload_bytes = bytes.slice_ref(payload);

        match event {
            b"render_progress_json" => find_render_id(payload)
                .map(|render_id| {
                    Self::RenderProgress(RawRenderProgress {
                        render_id,
                        bytes: payload_bytes,
                    })
                })
                .ok_or(crate::WebsocketError::InvalidEvent(bytes)),
            b"render_added_json" => Ok(Self::RenderAdded(RawRenderAdded {
                bytes: payload_bytes,
            })),
            b"render_done_json" => find_render_id(payload)
                .map(|render_id| {
                    Self::RenderDone(RawRenderDone {
                        render_id,
                        bytes: payload_bytes,
                    })
                })
                .ok_or(crate::WebsocketError::InvalidEvent(bytes)),
            b"render_failed_json" => find_render_id(payload)
                .map(|render_id| {
                    Self::RenderFailed(RawRenderFailed {
                        render_id,
                        bytes: payload_bytes,
                    })
                })
                .ok_or(crate::WebsocketError::InvalidEvent(bytes)),
            b"custom_skin_process_update" => {
                Ok(Self::CustomSkinProcessUpdate(RawCustomSkinProcessUpdate {
                    bytes: payload_bytes,
                }))
            }
            _ => Err(crate::WebsocketError::InvalidEvent(bytes)),
        }
    }

    /// Deserialize into an [`Event`].
    pub fn deserialize(&self) -> Result<Event, SerdeError> {
        match self {
            RawEvent::RenderAdded(event) => event.deserialize().map(Event::RenderAdded),
            RawEvent::RenderDone(event) => event.deserialize().map(Event::RenderDone),
            RawEvent::RenderFailed(event) => event.deserialize().map(Event::RenderFailed),
            RawEvent::RenderProgress(event) => event.deserialize().map(Event::RenderProgress),
            RawEvent::CustomSkinProcessUpdate(event) => {
                event.deserialize().map(Event::CustomSkinProcessUpdate)
            }
        }
    }
}

/// [`RenderAdded`] that has not been fully deserialized yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawRenderAdded {
    pub bytes: Bytes,
}

impl RawRenderAdded {
    /// Deserialize into a [`RenderAdded`] event.
    pub fn deserialize(&self) -> Result<RenderAdded, SerdeError> {
        serde_json::from_slice(&self.bytes)
    }
}

/// [`RenderProgress`] that has not been fully deserialized yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawRenderProgress {
    pub render_id: u32,
    pub bytes: Bytes,
}

impl RawRenderProgress {
    /// Deserialize into a [`RenderProgress`] event.
    pub fn deserialize(&self) -> Result<RenderProgress, SerdeError> {
        serde_json::from_slice(&self.bytes)
    }
}

/// [`RenderFailed`] that has not been fully deserialized yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawRenderFailed {
    pub render_id: u32,
    pub bytes: Bytes,
}

impl RawRenderFailed {
    /// Deserialize into a [`RenderFailed`] event.
    pub fn deserialize(&self) -> Result<RenderFailed, SerdeError> {
        serde_json::from_slice(&self.bytes)
    }
}

/// [`RenderDone`] that has not been fully deserialized yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawRenderDone {
    pub render_id: u32,
    pub bytes: Bytes,
}

impl RawRenderDone {
    /// Deserialize into a [`RenderDone`] event.
    pub fn deserialize(&self) -> Result<RenderDone, SerdeError> {
        serde_json::from_slice(&self.bytes)
    }
}

/// [`CustomSkinProcessUpdate`] that has not been fully deserialized yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawCustomSkinProcessUpdate {
    pub bytes: Bytes,
}

impl RawCustomSkinProcessUpdate {
    /// Deserialize into a [`CustomSkinProcessUpdate`] event.
    pub fn deserialize(&self) -> Result<CustomSkinProcessUpdate, SerdeError> {
        serde_json::from_slice(&self.bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::error::ErrorCode;

    #[test]
    fn parse_progress_event() {
        let frame = br#"["render_progress_json",{"description":"","progress":"45%","renderID":123,"renderer":"r","username":"u"}]"#;

        let progress = match RawEvent::from_bytes(Bytes::from_static(frame)).unwrap() {
            RawEvent::RenderProgress(progress) => progress,
            event => panic!("unexpected event: {event:?}"),
        };

        assert_eq!(progress.render_id, 123);
        assert_eq!(progress.deserialize().unwrap().progress.as_ref(), "45%");
    }

    #[test]
    fn parse_done_event() {
        let frame = br#"["render_done_json",{"renderID":1,"videoUrl":"https://ordr.issou.best/video.mp4"}]"#;

        let done = match RawEvent::from_bytes(Bytes::from_static(frame)).unwrap() {
            RawEvent::RenderDone(done) => done,
            event => panic!("unexpected event: {event:?}"),
        };

        assert_eq!(done.render_id, 1);
        assert_eq!(
            done.deserialize().unwrap().video_url.as_ref(),
            "https://ordr.issou.best/video.mp4"
        );
    }

    #[test]
    fn parse_failed_event() {
        let frame =
            br#"["render_failed_json",{"renderID":2,"errorCode":2,"errorMessage":"bad replay"}]"#;

        let failed = match RawEvent::from_bytes(Bytes::from_static(frame)).unwrap() {
            RawEvent::RenderFailed(failed) => failed,
            event => panic!("unexpected event: {event:?}"),
        };

        let failed = failed.deserialize().unwrap();

        assert_eq!(failed.render_id, 2);
        assert_eq!(failed.error_code, Some(ErrorCode::ReplayParsingError));
        assert_eq!(failed.error_message.as_ref(), "bad replay");
    }

    #[test]
    fn parse_added_event() {
        let frame = br#"["render_added_json",{"renderID":9}]"#;

        let added = match RawEvent::from_bytes(Bytes::from_static(frame)).unwrap() {
            RawEvent::RenderAdded(added) => added,
            event => panic!("unexpected event: {event:?}"),
        };

        assert_eq!(added.deserialize().unwrap().render_id, 9);
    }

    #[test]
    fn parse_custom_skin_event() {
        let frame = br#"["custom_skin_process_update",{"skinId":7}]"#;

        assert!(matches!(
            RawEvent::from_bytes(Bytes::from_static(frame)).unwrap(),
            RawEvent::CustomSkinProcessUpdate(_),
        ));
    }

    #[test]
    fn reject_unknown_event() {
        let frame = br#"["nope_json",{"a":1}]"#;

        assert!(RawEvent::from_bytes(Bytes::from_static(frame)).is_err());
    }

    #[test]
    fn reject_event_without_render_id() {
        let frame = br#"["render_progress_json",{"description":""}]"#;

        assert!(RawEvent::from_bytes(Bytes::from_static(frame)).is_err());
    }
}
