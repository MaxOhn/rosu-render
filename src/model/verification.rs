use std::fmt::{Debug, Formatter, Result as FmtResult};

#[derive(Clone)]
pub enum Verification {
    /// Verification key assigned by the o!rdr dev. DM MasterIO#4588 on Discord to get one.
    Key(Box<str>),
    /// Simulates a request that will successfully render a video. Listen to the websocket to get the state of this fake render
    /// (render_added, render_progress and render_done are emitted).
    DevModeSuccess,
    /// Simulates a request that will fail on the API level. No websocket events are emitted.
    DevModeFail,
    /// Simulates a request that will fail on the Websocket level (render_failed event is emitted).
    DevModeWsFail,
}

impl Verification {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Key(key) => key,
            Self::DevModeSuccess => "devmode_success",
            Self::DevModeFail => "devmode_fail",
            Self::DevModeWsFail => "devmode_wsfail",
        }
    }
}

impl Debug for Verification {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Key(_) => f.debug_tuple("Key").field(&"<redacted>").finish(),
            Self::DevModeSuccess => write!(f, "DevModeSuccess"),
            Self::DevModeFail => write!(f, "DevModeFail"),
            Self::DevModeWsFail => write!(f, "DevModeWsFail"),
        }
    }
}
