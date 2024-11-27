use std::fmt::{Debug, Formatter, Result as FmtResult};

/// Specifying your verification key will allow you to bypass the default ratelimit of one render per five minutes.
/// Alternatively, you can specify a dev mode to simulate events and not spam the real backend with requests while testing things.
#[derive(Clone)]
pub enum Verification {
    /// Verification key assigned by the o!rdr dev. DM MasterIO#4588 on Discord to get one.
    Key(Box<str>),
    /// Simulates a request that will successfully render a video. Listen to the websocket to get the state of this fake render
    /// (`render_added`, `render_progress` and `render_done` are emitted).
    DevModeSuccess,
    /// Simulates a request that will fail on the API level. No websocket events are emitted.
    DevModeFail,
    /// Simulates a request that will fail on the Websocket level (`render_failed` event is emitted).
    DevModeWsFail,
}

impl Verification {
    #[must_use]
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
            Self::DevModeSuccess => f.write_str("DevModeSuccess"),
            Self::DevModeFail => f.write_str("DevModeFail"),
            Self::DevModeWsFail => f.write_str("DevModeWsFail"),
        }
    }
}
