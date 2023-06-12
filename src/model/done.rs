use serde::Deserialize;

/// Data that is received in `on_render_done` websocket events.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash)]
pub struct RenderDone {
    /// The id of the render.
    #[serde(rename = "renderID")]
    pub render_id: u32,
    /// The url of the rendered video.
    #[serde(rename = "videoUrl")]
    pub video_url: Box<str>,
}
