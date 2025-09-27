use std::future::IntoFuture;

use serde::{Serialize, Serializer};

use crate::{model::UserPreset, request::Request, routing::Route, ClientError, OrdrClient};

use super::OrdrFuture;

#[derive(Serialize)]
struct GetUserPresetFields<'a> {
    key: &'a str,
    #[serde(serialize_with = "u64_as_str")]
    discord_id: u64,
}

#[expect(clippy::trivially_copy_pass_by_ref, reason = "required by serde")]
fn u64_as_str<S: Serializer>(n: &u64, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&n.to_string())
}

/// Get a [`UserPreset`].
#[must_use]
pub struct GetUserPreset<'a> {
    ordr: &'a OrdrClient,
    fields: GetUserPresetFields<'a>,
}

impl<'a> GetUserPreset<'a> {
    pub(crate) const fn new(ordr: &'a OrdrClient, key: &'a str, discord_id: u64) -> Self {
        Self {
            ordr,
            fields: GetUserPresetFields { key, discord_id },
        }
    }
}

impl IntoFuture for &mut GetUserPreset<'_> {
    type Output = Result<UserPreset, ClientError>;
    type IntoFuture = OrdrFuture<UserPreset>;

    fn into_future(self) -> Self::IntoFuture {
        match Request::builder(Route::UserPreset).query(&self.fields) {
            Ok(builder) => self.ordr.request(builder.build()),
            Err(err) => OrdrFuture::error(err),
        }
    }
}

impl IntoFuture for GetUserPreset<'_> {
    type Output = Result<UserPreset, ClientError>;
    type IntoFuture = OrdrFuture<UserPreset>;

    fn into_future(mut self) -> Self::IntoFuture {
        (&mut self).into_future()
    }
}
