use std::{
    error::Error as StdError,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    str::from_utf8 as str_from_utf8,
};

use hyper::{
    body::{Bytes, Incoming},
    Response,
};
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
pub enum ClientError {
    #[error("Failed to build the request")]
    BuildingRequest {
        #[source]
        source: Box<dyn StdError + Send + Sync + 'static>,
    },
    #[error("Failed to chunk the response")]
    ChunkingResponse {
        #[source]
        source: hyper::Error,
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
        source: hyper_util::client::legacy::Error,
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
    ServiceUnavailable { response: Response<Incoming> },
    #[error("Skin was not found (received a 404)")]
    SkinDeleted { error: SkinDeleted },
}

impl ClientError {
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
pub struct ApiError {
    /// The response of the server.
    pub message: Box<str>,
    /// The reason of the ban (if provided by admins).
    pub reason: Option<Box<str>>,
    /// The error code of the creation of this render.
    #[serde(rename = "errorCode")]
    pub code: Option<ErrorCode>,
}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Some(ref code) = self.code {
            write!(f, "Error code {code}: ")?;
        }

        f.write_str(&self.message)?;

        if let Some(ref reason) = self.reason {
            write!(f, " (reason: {reason})")?;
        }

        Ok(())
    }
}

/// Error codes as defined by o!rdr
///
/// See <https://ordr.issou.best/docs/#section/Error-codes>
#[derive(Copy, Clone, Debug, ThisError, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[repr(u8)]
pub enum ErrorCode {
    #[error("Emergency stop (triggered manually)")]
    EmergencyStop,
    #[error("Replay download error (bad upload from the sender)")]
    ReplayParsingError,
    #[error("Replay download error (bad download from the server), can happen because of invalid characters")]
    ReplayDownloadError,
    #[error("All beatmap mirrors are unavailable")]
    MirrorsUnavailable,
    #[error("Replay file corrupted")]
    ReplayFileCorrupted,
    #[error("Invalid osu! gamemode (not 0 = std)")]
    InvalidGameMode,
    #[error("The replay has no input data")]
    ReplayWithoutInputData,
    #[error("Beatmap does not exist on osu! (probably because of custom difficulty or non-submitted map)")]
    BeatmapNotFound,
    #[error("Audio for the map is unavailable (because of copyright claim)")]
    BeatmapAudioUnavailable,
    #[error("Cannot connect to osu! api")]
    OsuApiConnection,
    #[error("The replay has the autoplay mod")]
    ReplayIsAutoplay,
    #[error("The replay username has invalid characters")]
    InvalidReplayUsername,
    #[error("The beatmap is longer than 15 minutes")]
    BeatmapTooLong,
    #[error("This player is banned from o!rdr")]
    PlayerBannedFromOrdr,
    #[error("Beatmap not found on all the beatmap mirrors")]
    MapNotFound,
    #[error("This IP is banned from o!rdr")]
    IpBannedFromOrdr,
    #[error("This username is banned from o!rdr")]
    UsernameBannedFromOrdr,
    #[error("Unknown error from the renderer")]
    UnknownRendererError,
    #[error("The renderer cannot download the map")]
    CannotDownloadMap,
    #[error("Beatmap version on the mirror is not the same as the replay")]
    InconsistentMapVersion,
    #[error("The replay is corrupted (danser cannot process it)")]
    ReplayFileCorrupted2,
    #[error("Server-side problem while finalizing the generated video")]
    FailedFinalizing,
    #[error("Server-side problem while preparing the render")]
    ServerFailedPreparation,
    #[error("The beatmap has no name")]
    BeatmapHasNoName,
    #[error("The replay is missing input data")]
    ReplayMissingInputData,
    #[error("The replay has incompatible mods")]
    ReplayIncompatibleMods,
    #[error(
        "Something with the renderer went wrong: it probably has an unstable internet connection \
        (multiple renders at the same time)"
    )]
    RendererIssue,
    #[error("The renderer cannot download the replay")]
    CannotDownloadReplay,
    #[error("The replay is already rendering or in queue")]
    ReplayAlreadyInQueue,
    #[error("The star rating is greater than 20")]
    StarRatingTooHigh,
    #[error("The mapper is blacklisted")]
    MapperIsBlacklisted,
    #[error("The beatmapset is blacklisted")]
    BeatmapsetIsBlacklisted,
    #[error("The replay has already errored less than an hour ago")]
    ReplayErroredRecently,
    #[error("invalid replay URL or can't download the replay (if replayURL is provided)")]
    InvalidReplayUrl,
    #[error("a required field is missing (the missing field is shown in the message)")]
    MissingField,
    #[error("your last replays have a too high error rate (cannot be triggered when you're a verified bot)")]
    ErrorRateTooHigh,
    #[error("the replay username is inappropriate")]
    InappropriateUsername,
    #[error("this skin does not exist")]
    SkinDoesNotExist,
    #[error("this custom skin does not exist or has been deleted")]
    CustomSkinDoesNotExist,
    #[error("o!rdr is not ready to take render jobs at the moment")]
    RenderJobsPaused,
    #[error("o!rdr is not ready to take render jobs from unauthenticated users at the moment (verified bots are not authenticated users)")]
    UnauthenticatedRenderJobsPaused,
    #[error("replay accuracy is too bad and you're not authenticated")]
    AccuracyTooLow,
    #[error("this score does not exist")]
    ScoreDoesNotExist,
    #[error("the replay for this score isn't available")]
    ReplayUnavailable,
    #[error("invalid osu! ruleset score ID")]
    InvalidRulesetId,
    #[error("Unknown error code {0}")]
    Other(u8),
}

impl ErrorCode {
    #[must_use]
    pub fn to_u8(self) -> u8 {
        match self {
            Self::EmergencyStop => 1,
            Self::ReplayParsingError => 2,
            Self::ReplayDownloadError => 3,
            Self::MirrorsUnavailable => 4,
            Self::ReplayFileCorrupted => 5,
            Self::InvalidGameMode => 6,
            Self::ReplayWithoutInputData => 7,
            Self::BeatmapNotFound => 8,
            Self::BeatmapAudioUnavailable => 9,
            Self::OsuApiConnection => 10,
            Self::ReplayIsAutoplay => 11,
            Self::InvalidReplayUsername => 12,
            Self::BeatmapTooLong => 13,
            Self::PlayerBannedFromOrdr => 14,
            Self::MapNotFound => 15,
            Self::IpBannedFromOrdr => 16,
            Self::UsernameBannedFromOrdr => 17,
            Self::UnknownRendererError => 18,
            Self::CannotDownloadMap => 19,
            Self::InconsistentMapVersion => 20,
            Self::ReplayFileCorrupted2 => 21,
            Self::FailedFinalizing => 22,
            Self::ServerFailedPreparation => 23,
            Self::BeatmapHasNoName => 24,
            Self::ReplayMissingInputData => 25,
            Self::ReplayIncompatibleMods => 26,
            Self::RendererIssue => 27,
            Self::CannotDownloadReplay => 28,
            Self::ReplayAlreadyInQueue => 29,
            Self::StarRatingTooHigh => 30,
            Self::MapperIsBlacklisted => 31,
            Self::BeatmapsetIsBlacklisted => 32,
            Self::ReplayErroredRecently => 33,
            Self::InvalidReplayUrl => 34,
            Self::MissingField => 35,
            Self::ErrorRateTooHigh => 36,
            Self::InappropriateUsername => 37,
            Self::SkinDoesNotExist => 38,
            Self::CustomSkinDoesNotExist => 39,
            Self::RenderJobsPaused => 40,
            Self::UnauthenticatedRenderJobsPaused => 41,
            Self::AccuracyTooLow => 42,
            Self::ScoreDoesNotExist => 43,
            Self::ReplayUnavailable => 44,
            Self::InvalidRulesetId => 45,
            Self::Other(code) => code,
        }
    }
}

impl From<u8> for ErrorCode {
    fn from(code: u8) -> Self {
        match code {
            1 => Self::EmergencyStop,
            2 => Self::ReplayParsingError,
            5 => Self::ReplayFileCorrupted,
            6 => Self::InvalidGameMode,
            7 => Self::ReplayWithoutInputData,
            8 => Self::BeatmapNotFound,
            9 => Self::BeatmapAudioUnavailable,
            10 => Self::OsuApiConnection,
            11 => Self::ReplayIsAutoplay,
            12 => Self::InvalidReplayUsername,
            13 => Self::BeatmapTooLong,
            14 => Self::PlayerBannedFromOrdr,
            16 => Self::IpBannedFromOrdr,
            17 => Self::UsernameBannedFromOrdr,
            23 => Self::ServerFailedPreparation,
            24 => Self::BeatmapHasNoName,
            25 => Self::ReplayMissingInputData,
            26 => Self::ReplayIncompatibleMods,
            29 => Self::ReplayAlreadyInQueue,
            30 => Self::StarRatingTooHigh,
            31 => Self::MapperIsBlacklisted,
            32 => Self::BeatmapsetIsBlacklisted,
            33 => Self::ReplayErroredRecently,
            34 => Self::InvalidReplayUrl,
            35 => Self::MissingField,
            36 => Self::ErrorRateTooHigh,
            37 => Self::InappropriateUsername,
            38 => Self::SkinDoesNotExist,
            39 => Self::CustomSkinDoesNotExist,
            40 => Self::RenderJobsPaused,
            41 => Self::UnauthenticatedRenderJobsPaused,
            42 => Self::AccuracyTooLow,
            43 => Self::ScoreDoesNotExist,
            44 => Self::ReplayUnavailable,
            45 => Self::InvalidRulesetId,
            other => Self::Other(other),
        }
    }
}

impl<'de> Deserialize<'de> for ErrorCode {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct ErrorCodeVisitor;

        impl Visitor<'_> for ErrorCodeVisitor {
            type Value = ErrorCode;

            fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
                f.write_str("u8")
            }

            fn visit_u8<E: DeError>(self, v: u8) -> Result<Self::Value, E> {
                Ok(ErrorCode::from(v))
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
