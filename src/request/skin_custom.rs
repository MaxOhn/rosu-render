use std::future::IntoFuture;

use serde::Serialize;

use crate::{error::Error, model::SkinInfo, request::Request, routing::Route, Ordr};

use super::OrdrFuture;

#[derive(Serialize)]
struct GetSkinCustomFields {
    id: u32,
}

// TODO: docs
pub struct GetSkinCustom<'a> {
    ordr: &'a Ordr,
    fields: GetSkinCustomFields,
}

impl<'a> GetSkinCustom<'a> {
    pub(crate) const fn new(ordr: &'a Ordr, id: u32) -> Self {
        Self {
            ordr,
            fields: GetSkinCustomFields { id },
        }
    }
}

impl IntoFuture for &mut GetSkinCustom<'_> {
    type Output = Result<SkinInfo, Error>;
    type IntoFuture = OrdrFuture<SkinInfo>;

    fn into_future(self) -> Self::IntoFuture {
        match Request::builder(Route::SkinCustom).query(&self.fields) {
            Ok(builder) => self.ordr.request(builder.build()),
            Err(err) => OrdrFuture::error(err),
        }
    }
}

impl IntoFuture for GetSkinCustom<'_> {
    type Output = Result<SkinInfo, Error>;
    type IntoFuture = OrdrFuture<SkinInfo>;

    fn into_future(mut self) -> Self::IntoFuture {
        (&mut self).into_future()
    }
}
