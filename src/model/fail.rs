use serde::Deserialize;

use crate::error::ErrorCode;

/// Data that is received in `on_render_failed` websocket events.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct RenderFail {
    /// The id of the render.
    #[serde(rename = "renderID")]
    pub render_id: u32,
    /// The error code as specified by o!rdr.
    pub error_code: Option<ErrorCode>,
    /// An error message.
    pub error_message: Box<str>,
}
