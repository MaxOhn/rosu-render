use std::future::IntoFuture;

use serde::Serialize;

use crate::{error::Error, model::RenderList, routing::Route, Ordr};

use super::{OrdrFuture, Request};

#[derive(Serialize)]
struct GetRenderListFields<'a> {
    #[serde(rename = "pageSize")]
    page_size: Option<u32>,
    page: Option<u32>,
    #[serde(rename = "ordrUsername")]
    ordr_username: Option<&'a str>,
    #[serde(rename = "replayUsername")]
    replay_username: Option<&'a str>,
    #[serde(rename = "renderID")]
    render_id: Option<u32>,
    #[serde(rename = "nobots")]
    no_bots: Option<bool>,
    link: Option<&'a str>,
    #[serde(rename = "beatmapsetid")]
    mapset_id: Option<u32>,
}

// TODO: docs
pub struct GetRenderList<'a> {
    ordr: &'a Ordr,
    fields: GetRenderListFields<'a>,
}

impl<'a> GetRenderList<'a> {
    pub(crate) const fn new(ordr: &'a Ordr) -> Self {
        Self {
            ordr,
            fields: GetRenderListFields {
                page_size: None,
                page: None,
                ordr_username: None,
                replay_username: None,
                render_id: None,
                no_bots: None,
                link: None,
                mapset_id: None,
            },
        }
    }

    /// The number of renders the query will return you in the page. If not specified, 50 is the default.
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

    /// Search by o!rdr username, can be used at the same time as replay_username.
    pub fn ordr_username(&mut self, ordr_username: &'a str) -> &mut Self {
        self.fields.ordr_username = Some(ordr_username);

        self
    }

    /// Search by replay username, can be used at the same time as ordr_username.
    pub fn replay_username(&mut self, replay_username: &'a str) -> &mut Self {
        self.fields.replay_username = Some(replay_username);

        self
    }

    /// The render ID of a render.
    pub fn render_id(&mut self, render_id: u32) -> &mut Self {
        self.fields.render_id = Some(render_id);

        self
    }

    /// Hide bots from the returned render query.
    pub fn no_bots(&mut self, no_bots: bool) -> &mut Self {
        self.fields.no_bots = Some(no_bots);

        self
    }

    /// The path of a shortlink (for example `pov8n` for `https://link.issou.best/pov8n`)
    pub fn link(&mut self, link: &'a str) -> &mut Self {
        self.fields.link = Some(link);

        self
    }

    /// Get renders with this specific beatmapset ID
    pub fn mapset_id(&mut self, mapset_id: u32) -> &mut Self {
        self.fields.mapset_id = Some(mapset_id);

        self
    }
}

impl IntoFuture for &mut GetRenderList<'_> {
    type Output = Result<RenderList, Error>;
    type IntoFuture = OrdrFuture<RenderList>;

    fn into_future(self) -> Self::IntoFuture {
        match Request::builder(Route::RenderList).query(&self.fields) {
            Ok(builder) => self.ordr.request(builder.build()),
            Err(err) => OrdrFuture::error(err),
        }
    }
}

impl IntoFuture for GetRenderList<'_> {
    type Output = Result<RenderList, Error>;
    type IntoFuture = OrdrFuture<RenderList>;

    fn into_future(mut self) -> Self::IntoFuture {
        (&mut self).into_future()
    }
}
