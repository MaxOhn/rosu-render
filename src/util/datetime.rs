use std::fmt::{Formatter, Result as FmtResult};

use serde::{
    de::{Error as DeError, Unexpected, Visitor},
    Deserializer,
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

struct OffsetDateTimeVisitor;

impl Visitor<'_> for OffsetDateTimeVisitor {
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

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Deserialize)]
    struct Wrapper {
        #[serde(deserialize_with = "deserialize_datetime")]
        date: OffsetDateTime,
    }

    fn expected() -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp_nanos(1_700_000_000_123_i128 * 1_000_000).unwrap()
    }

    #[test]
    fn from_milliseconds() {
        let wrapper: Wrapper = serde_json::from_str(r#"{"date": 1700000000123}"#).unwrap();

        assert_eq!(wrapper.date, expected());
    }

    #[test]
    fn from_rfc3339() {
        let wrapper: Wrapper =
            serde_json::from_str(r#"{"date": "2023-11-14T22:13:20.123Z"}"#).unwrap();

        assert_eq!(wrapper.date, expected());
    }

    #[test]
    fn invalid() {
        assert!(serde_json::from_str::<Wrapper>(r#"{"date": "not a date"}"#).is_err());
    }
}
