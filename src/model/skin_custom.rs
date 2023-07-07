use hyper::{body::Bytes, StatusCode};
use serde::Deserialize;

use crate::{request::Requestable, ClientError};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct SkinInfo {
    /// The name of the skin.
    #[serde(rename = "skinName")]
    pub name: Box<str>,
    /// The author (skinner, parsed from the skin's skin.ini) of the skin.
    #[serde(rename = "skinAuthor")]
    pub author: Box<str>,
    /// The download link for this custom skin, from issou.best servers.
    #[serde(rename = "downloadLink")]
    pub download_link: Box<str>,
}

impl Requestable for SkinInfo {
    fn response_error(status: StatusCode, bytes: Bytes) -> ClientError {
        if status == StatusCode::NOT_FOUND {
            match serde_json::from_slice(&bytes) {
                Ok(error) => ClientError::SkinDeleted { error },
                Err(source) => ClientError::Parsing {
                    body: bytes.into(),
                    source,
                },
            }
        } else {
            ClientError::response_error(bytes, status.as_u16())
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct SkinDeleted {
    /// true if found, false if not.
    pub found: bool,
    /// true if removed, false if not.
    pub removed: bool,
    /// The info message, in english, of an error.
    pub message: Box<str>,
    /// The name of the skin.
    pub name: Option<Box<str>>,
    /// The author (skinner, parsed from the skin's skin.ini) of the skin.
    pub author: Option<Box<str>>,
}
