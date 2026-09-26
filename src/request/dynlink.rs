use std::future::IntoFuture;

use serde::Serialize;

use crate::{model::DynLink, request::Request, routing::Route, ClientError, OrdrClient};

use super::OrdrFuture;

#[derive(Serialize)]
struct GetDynLinkFields {
    id: u32,
}

/// Generate a temporary video download link for a rendered video.
#[must_use]
pub struct GetDynLink<'a> {
    ordr: &'a OrdrClient,
    fields: GetDynLinkFields,
}

impl<'a> GetDynLink<'a> {
    pub(crate) const fn new(ordr: &'a OrdrClient, id: u32) -> Self {
        Self {
            ordr,
            fields: GetDynLinkFields { id },
        }
    }
}

impl IntoFuture for &mut GetDynLink<'_> {
    type Output = Result<DynLink, ClientError>;
    type IntoFuture = OrdrFuture<DynLink>;

    fn into_future(self) -> Self::IntoFuture {
        match Request::builder(Route::DynLink).query(&self.fields) {
            Ok(builder) => self.ordr.request(builder.build()),
            Err(err) => OrdrFuture::error(err),
        }
    }
}

impl IntoFuture for GetDynLink<'_> {
    type Output = Result<DynLink, ClientError>;
    type IntoFuture = OrdrFuture<DynLink>;

    fn into_future(mut self) -> Self::IntoFuture {
        (&mut self).into_future()
    }
}
