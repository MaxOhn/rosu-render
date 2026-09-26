use hyper::{body::Bytes, StatusCode};
use serde::Deserialize;

use crate::{request::Requestable, ClientError};

/// A temporary video download link.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct DynLink {
    /// The temporary download URL of the rendered video.
    pub url: Box<str>,
}

impl Requestable for DynLink {
    fn response_error(status: StatusCode, bytes: Bytes) -> ClientError {
        ClientError::response_error(bytes, status.as_u16())
    }
}
