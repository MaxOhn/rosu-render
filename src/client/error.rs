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

macro_rules! define_error_code {
    (
        $( #[ $meta:meta ] )*
        $vis:vis enum $name:ident {
            $(
                #[ $variant_meta:meta ]
                $variant:ident = $discriminant:literal,
            )*
        }
    ) => {
        $( #[$meta] )*
        $vis enum $name {
            $(
                #[$variant_meta]
                $variant,
            )*
            #[error("Unknown error code {0}")]
            Other(u8),
        }

        impl $name {
            #[must_use]
            pub fn to_u8(self) -> u8 {
                match self {
                    $( Self::$variant => $discriminant, )*
                    Self::Other(code) => code,
                }
            }
        }

        impl From<u8> for ErrorCode {
            fn from(code: u8) -> Self {
                match code {
                    $( $discriminant => Self::$variant, )*
                    other => Self::Other(other),
                }
            }
        }
    };
}

define_error_code! {
    /// Error codes as defined by o!rdr
    ///
    /// See <https://ordr.issou.best/docs/#section/Error-codes>
    #[derive(Copy, Clone, Debug, ThisError, PartialEq, Eq, Hash)]
    #[non_exhaustive]
    #[repr(u8)]
    pub enum ErrorCode {
        #[error("Emergency stop (triggered manually)")]
        EmergencyStop = 1,
        #[error("Replay download error (bad upload from the sender)")]
        ReplayParsingError = 2,
        #[error("Replay download error (bad download from the server), can happen because of invalid characters")]
        ReplayDownloadError = 3,
        #[error("All beatmap mirrors are unavailable")]
        MirrorsUnavailable = 4,
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
        #[error("Beatmap not found on all the beatmap mirrors")]
        MapNotFound = 15,
        #[error("This IP is banned from o!rdr")]
        IpBannedFromOrdr = 16,
        #[error("This username is banned from o!rdr")]
        UsernameBannedFromOrdr = 17,
        #[error("Unknown error from the renderer")]
        UnknownRendererError = 18,
        #[error("The renderer cannot download the map")]
        CannotDownloadMap = 19,
        #[error("Beatmap version on the mirror is not the same as the replay")]
        InconsistentMapVersion = 20,
        #[error("The replay is corrupted (danser cannot process it)")]
        ReplayFileCorrupted2 = 21,
        #[error("Server-side problem while finalizing the generated video")]
        FailedFinalizing = 22,
        #[error("Server-side problem while preparing the render")]
        ServerFailedPreparation = 23,
        #[error("The beatmap has no name")]
        BeatmapHasNoName = 24,
        #[error("The replay is missing input data")]
        ReplayMissingInputData = 25,
        #[error("The replay has incompatible mods")]
        ReplayIncompatibleMods = 26,
        #[error("Something with the renderer went wrong: it probably has an unstable internet connection (multiple renders at the same time)")]
        RendererIssue = 27,
        #[error("The renderer cannot download the replay")]
        CannotDownloadReplay = 28,
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
        #[error("invalid replay URL or can't download the replay (if replayURL is provided)")]
        InvalidReplayUrl = 34,
        #[error("a required field is missing (the missing field is shown in the message)")]
        MissingField = 35,
        #[error("your last replays have a too high error rate (cannot be triggered when you're a verified bot)")]
        ErrorRateTooHigh = 36,
        #[error("the replay username is inappropriate")]
        InappropriateUsername = 37,
        #[error("this skin does not exist")]
        SkinDoesNotExist = 38,
        #[error("this custom skin does not exist or has been deleted")]
        CustomSkinDoesNotExist = 39,
        #[error("o!rdr is not ready to take render jobs at the moment")]
        RenderJobsPaused = 40,
        #[error("o!rdr is not ready to take render jobs from unauthenticated users at the moment (verified bots are not authenticated users)")]
        UnauthenticatedRenderJobsPaused = 41,
        #[error("replay accuracy is too bad and you're not authenticated")]
        AccuracyTooLow = 42,
        #[error("this score does not exist")]
        ScoreDoesNotExist = 43,
        #[error("the replay for this score isn't available")]
        ReplayUnavailable = 44,
        #[error("invalid osu! ruleset score ID")]
        InvalidRulesetId = 45,
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
