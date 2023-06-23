use std::{
    error::Error as StdError,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    str::from_utf8 as str_from_utf8,
};

use hyper::{body::Bytes, Body, Error as HyperError, Response};
use rust_socketio::Error as SocketIoError;
use serde::{
    de::{Deserializer, Error as DeError, Unexpected, Visitor},
    Deserialize,
};
use serde_json::Error as JsonError;
use serde_urlencoded::ser::Error as UrlError;
use thiserror::Error as ThisError;

use crate::model::SkinDeleted;

#[derive(Debug, ThisError)]
#[non_exhaustive]
pub enum Error {
    #[error("Failed to build the request")]
    BuildingRequest {
        #[source]
        source: Box<dyn StdError + Send + Sync + 'static>,
    },
    #[error("Failed to chunk the response")]
    ChunkingResponse {
        #[source]
        source: HyperError,
    },
    #[error("Failed to deserialize response body: {body}")]
    Parsing {
        body: StringOrBytes,
        #[source]
        source: JsonError,
    },
    #[error("Parsing or sending the response failed")]
    RequestError {
        #[source]
        source: HyperError,
    },
    #[error("Response error: status code {status_code}, {error}")]
    Response {
        body: Bytes,
        error: ApiError,
        status_code: u16,
    },
    #[error("Failed to serialize the query")]
    SerdeQuery {
        #[from]
        source: UrlError,
    },
    #[error("API may be temporarily unavailable (received a 503)")]
    ServiceUnavailable { response: Response<Body> },
    #[error("Skin was not found (received a 404)")]
    SkinDeleted { error: SkinDeleted },
    #[error("socket.io error")]
    SocketIo {
        #[from]
        source: SocketIoError,
    },
    #[error("Banned from o!rdr. All future requests will fail.")]
    Unauthorized,
}

impl Error {
    pub(crate) fn response_error(bytes: Bytes, status_code: u16) -> Self {
        match serde_json::from_slice(&bytes) {
            Ok(error) => Self::Response {
                body: bytes,
                error,
                status_code,
            },
            Err(source) => Self::Parsing {
                body: bytes.into(),
                source,
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct StringOrBytes {
    bytes: Bytes,
}

impl Display for StringOrBytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match str_from_utf8(&self.bytes) {
            Ok(string) => f.write_str(string),
            Err(_) => <[u8] as Debug>::fmt(&*self.bytes, f),
        }
    }
}

impl From<Bytes> for StringOrBytes {
    fn from(bytes: Bytes) -> Self {
        Self { bytes }
    }
}

#[derive(Debug, Deserialize)]
#[non_exhaustive]
#[serde(untagged)]
pub enum ApiError {
    Reasoned(ReasonedApiError),
    Coded(CodedApiError),
    General(GeneralApiError),
}

impl ApiError {
    pub fn code(&self) -> Option<ErrorCode> {
        match self {
            ApiError::Reasoned(err) => Some(err.code),
            ApiError::Coded(err) => Some(err.code),
            ApiError::General(_) => None,
        }
    }

    pub fn message(&self) -> &str {
        match self {
            ApiError::Reasoned(err) => &err.message,
            ApiError::Coded(err) => &err.message,
            ApiError::General(err) => &err.message,
        }
    }
}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Reasoned(err) => Display::fmt(err, f),
            Self::Coded(err) => Display::fmt(err, f),
            Self::General(err) => Display::fmt(err, f),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GeneralApiError {
    /// The response of the server.
    pub message: Box<str>,
}

impl Display for GeneralApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.message, f)
    }
}

#[derive(Debug, Deserialize)]
pub struct CodedApiError {
    /// The response of the server.
    pub message: Box<str>,
    /// The error code of the creation of this render.
    #[serde(rename = "errorCode")]
    pub code: ErrorCode,
}

impl Display for CodedApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "Error code {code}: {msg}",
            code = self.code,
            msg = self.message,
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct ReasonedApiError {
    /// The response of the server.
    pub message: Box<str>,
    /// The reason of the ban (if provided by admins).
    pub reason: Box<str>,
    /// The error code of the creation of this render.
    #[serde(rename = "errorCode")]
    pub code: ErrorCode,
}

impl Display for ReasonedApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "Error code {code}: {msg} (reason: {reason})",
            code = self.code,
            msg = self.message,
            reason = self.reason,
        )
    }
}

/// Error codes as defined by o!rdr
///
/// See <https://ordr.issou.best/docs/#section/Error-codes>
#[derive(Copy, Clone, Debug, ThisError, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ErrorCode {
    #[error("Replay parsing error (bad upload from the sender)")]
    ReplayParsingError = 2,
    #[error("Replay file corrupted")]
    ReplayFileCorrupted = 5,
    #[error("Invalid osu! gamemode (not 0 = std)")]
    InvalidGameMode = 6,
    #[error("The replay has no input data")]
    ReplayWithoutInputData = 7,
    #[error("Beatmap does not exist on osu! (probably because of custom difficulty or non-submitted map)")]
    BeatmapNotFound = 8,
    #[error("Audio for the map is unavailable (because of copyright claim)")]
    BeatmapAudioUnavailable = 9,
    #[error("Cannot connect to osu! api")]
    OsuApiConnection = 10,
    #[error("The replay has the autoplay mod")]
    ReplayIsAutoplay = 11,
    #[error("The replay username has invalid characters")]
    InvalidReplayUsername = 12,
    #[error("The beatmap is longer than 15 minutes")]
    BeatmapTooLong = 13,
    #[error("This player is banned from o!rdr")]
    PlayerBannedFromOrdr = 14,
    #[error("This IP is banned from o!rdr")]
    IpBannedFromOrdr = 16,
    #[error("This username is banned from o!rdr")]
    UsernameBannedFromOrdr = 17,
    #[error("Server-side problem while preparing the render")]
    ServerFailedPreparation = 23,
    #[error("The beatmap has no name")]
    BeatmapHasNoName = 24,
    #[error("The replay is missing input data")]
    ReplayMissingInputData = 25,
    #[error("The replay has incompatible mods")]
    ReplayIncompatibleMods = 26,
    #[error("The replay is already rendering or in queue")]
    ReplayAlreadyInQueue = 29,
    #[error("The star rating is greater than 20")]
    StarRatingTooHigh = 30,
    #[error("The mapper is blacklisted")]
    MapperIsBlacklisted = 31,
    #[error("The beatmapset is blacklisted")]
    BeatmapsetIsBlacklisted = 32,
    #[error("The replay has already errored less than an hour ago")]
    ReplayErroredRecently = 33,
    #[error("Unknown error code {0}")]
    Other(u8),
}

impl<'de> Deserialize<'de> for ErrorCode {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct ErrorCodeVisitor;

        impl<'de> Visitor<'de> for ErrorCodeVisitor {
            type Value = ErrorCode;

            fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
                f.write_str("u8")
            }

            fn visit_u8<E: DeError>(self, v: u8) -> Result<Self::Value, E> {
                let code = match v {
                    2 => ErrorCode::ReplayParsingError,
                    5 => ErrorCode::ReplayFileCorrupted,
                    6 => ErrorCode::InvalidGameMode,
                    7 => ErrorCode::ReplayWithoutInputData,
                    8 => ErrorCode::BeatmapNotFound,
                    9 => ErrorCode::BeatmapAudioUnavailable,
                    10 => ErrorCode::OsuApiConnection,
                    11 => ErrorCode::ReplayIsAutoplay,
                    12 => ErrorCode::InvalidReplayUsername,
                    13 => ErrorCode::BeatmapTooLong,
                    14 => ErrorCode::PlayerBannedFromOrdr,
                    16 => ErrorCode::IpBannedFromOrdr,
                    17 => ErrorCode::UsernameBannedFromOrdr,
                    23 => ErrorCode::ServerFailedPreparation,
                    24 => ErrorCode::BeatmapHasNoName,
                    25 => ErrorCode::ReplayMissingInputData,
                    26 => ErrorCode::ReplayIncompatibleMods,
                    29 => ErrorCode::ReplayAlreadyInQueue,
                    30 => ErrorCode::StarRatingTooHigh,
                    31 => ErrorCode::MapperIsBlacklisted,
                    32 => ErrorCode::BeatmapsetIsBlacklisted,
                    33 => ErrorCode::ReplayErroredRecently,
                    other => ErrorCode::Other(other),
                };

                Ok(code)
            }

            fn visit_u64<E: DeError>(self, v: u64) -> Result<Self::Value, E> {
                let code = u8::try_from(v).map_err(|_| {
                    DeError::invalid_value(Unexpected::Unsigned(v), &"a valid error code")
                })?;

                self.visit_u8(code)
            }
        }

        d.deserialize_u8(ErrorCodeVisitor)
    }
}
