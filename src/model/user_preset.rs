use std::collections::HashMap;

use hyper::{body::Bytes, StatusCode};
use serde::Deserialize;
use serde_json::Value;
use time::OffsetDateTime;

use crate::{request::Requestable, util::datetime::deserialize_datetime, ClientError};

/// Preset render settings of a user
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct UserPreset {
    /// Name of the preset
    #[serde(rename = "presetName")]
    name: String,
    /// Timestamp of last preset save
    #[serde(rename = "lastSavedOn", deserialize_with = "deserialize_datetime")]
    last_saved_on: OffsetDateTime,
    /// All setting values
    #[serde(flatten)]
    settings: HashMap<String, Value>,
}

impl Requestable for UserPreset {
    fn response_error(status: StatusCode, bytes: Bytes) -> ClientError {
        if status == StatusCode::NOT_FOUND {
            ClientError::PresetNotFound
        } else {
            ClientError::response_error(bytes, status.as_u16())
        }
    }
}
