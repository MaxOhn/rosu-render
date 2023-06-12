use serde::Deserialize;

/// Data that is received in `on_render_progress` websocket events.
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
