use std::future::IntoFuture;

use serde::Serialize;

use crate::{error::Error, model::SkinList, request::Request, routing::Route, Ordr};

use super::OrdrFuture;

#[derive(Serialize)]
struct GetSkinListFields<'a> {
    #[serde(rename = "pageSize")]
    page_size: Option<u32>,
    page: Option<u32>,
    search: Option<&'a str>,
}

// TODO: docs
pub struct GetSkinList<'a> {
    ordr: &'a Ordr,
    fields: GetSkinListFields<'a>,
}

impl<'a> GetSkinList<'a> {
    pub(crate) const fn new(ordr: &'a Ordr) -> Self {
        Self {
            ordr,
            fields: GetSkinListFields {
                page_size: None,
                page: None,
                search: None,
            },
        }
    }

    /// The number of skins the API will return you in the page. If not specified, 100 is the default.
    pub fn page_size(&mut self, page_size: u32) -> &mut Self {
        self.fields.page_size = Some(page_size);
        self.fields.page.get_or_insert(1);

        self
    }

    /// The page.
    pub fn page(&mut self, page: u32) -> &mut Self {
        self.fields.page = Some(page);

        self
    }

    /// Get the skins that matches the most your string.
    pub fn search(&mut self, search: &'a str) -> &mut Self {
        self.fields.search = Some(search);

        self
    }
}

impl IntoFuture for &mut GetSkinList<'_> {
    type Output = Result<SkinList, Error>;
    type IntoFuture = OrdrFuture<SkinList>;

    fn into_future(self) -> Self::IntoFuture {
        match Request::builder(Route::SkinList).query(&self.fields) {
            Ok(builder) => self.ordr.request(builder.build()),
            Err(err) => OrdrFuture::error(err),
        }
    }
}

impl IntoFuture for GetSkinList<'_> {
    type Output = Result<SkinList, Error>;
    type IntoFuture = OrdrFuture<SkinList>;

    fn into_future(mut self) -> Self::IntoFuture {
        (&mut self).into_future()
    }
}
