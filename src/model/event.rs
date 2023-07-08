use bytes::Bytes;
use hyper::StatusCode;
use serde::Deserialize;

use crate::{client::error::ErrorCode, request::Requestable, ClientError};

/// Deserialized [`Event`](crate::model::Event) received through the websocket.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Event {
    RenderAdded(RenderAdded),
    RenderProgress(RenderProgress),
    RenderFailed(RenderFailed),
    RenderDone(RenderDone),
    CustomSkinProcessUpdate(CustomSkinProcessUpdate),
}

/// Data that is received in `render_added_json` websocket events.
///
/// Also the response of the server when the render got created successfully.
#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Eq, Hash)]
pub struct RenderAdded {
    /// The render ID of your render that got created.
    #[serde(rename = "renderID")]
    pub render_id: u32,
}

impl Requestable for RenderAdded {
    fn response_error(status: StatusCode, bytes: Bytes) -> ClientError {
        ClientError::response_error(bytes, status.as_u16())
    }
}

/// Data that is received in `render_done_json` websocket events.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash)]
pub struct RenderDone {
    /// The id of the render.
    #[serde(rename = "renderID")]
    pub render_id: u32,
    /// The url of the rendered video.
    #[serde(rename = "videoUrl")]
    pub video_url: Box<str>,
}

/// Data that is received in `render_failed_json` websocket events.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct RenderFailed {
    /// The id of the render.
    #[serde(rename = "renderID")]
    pub render_id: u32,
    /// The error code as specified by o!rdr.
    pub error_code: Option<ErrorCode>,
    /// An error message.
    pub error_message: Box<str>,
}

/// Data that is received in `render_progress_json` websocket events.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash)]
pub struct RenderProgress {
    /// Description of the replay.
    pub description: Box<str>,
    /// Current render status.
    pub progress: Box<str>,
    /// The id of the render.
    #[serde(rename = "renderID")]
    pub render_id: u32,
    /// Server that renders the replay.
    pub renderer: Box<str>,
    /// User that commissioned the render.
    pub username: Box<str>,
}

/// Data that is received in `custom_skin_process_update` websocket events.
#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Eq, Hash)]
pub struct CustomSkinProcessUpdate {
    /// The id of the skin that was processed.
    #[serde(rename = "skinId")]
    skin_id: u32,
}
