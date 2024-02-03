use std::fmt::{Formatter, Result as FmtResult};

use serde::{
    de::{Error as DeError, Unexpected, Visitor},
    Deserializer,
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

struct OffsetDateTimeVisitor;

impl<'de> Visitor<'de> for OffsetDateTimeVisitor {
    type Value = OffsetDateTime;

    fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("an OffsetDateTime")
    }

    fn visit_u64<E: DeError>(self, timestamp_ms: u64) -> Result<Self::Value, E> {
        let timestamp_ns = i128::from(timestamp_ms) * 1_000_000;

        OffsetDateTime::from_unix_timestamp_nanos(timestamp_ns).map_err(|_| {
            DeError::invalid_value(
                Unexpected::Unsigned(timestamp_ms),
                &"a valid unix timestamp in milliseconds",
            )
        })
    }

    fn visit_str<E: DeError>(self, datetime: &str) -> Result<Self::Value, E> {
        OffsetDateTime::parse(datetime, &Rfc3339).map_err(|_| {
            DeError::invalid_value(
                Unexpected::Str(datetime),
                &"an RFC3339-formatted `OffsetDateTime`",
            )
        })
    }
}

pub(crate) fn deserialize_datetime<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<OffsetDateTime, D::Error> {
    d.deserialize_any(OffsetDateTimeVisitor)
}
