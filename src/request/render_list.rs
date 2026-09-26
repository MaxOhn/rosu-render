use std::future::IntoFuture;

use serde::Serialize;

use crate::{model::RenderList, routing::Route, ClientError, OrdrClient};

use super::{OrdrFuture, Request, RequestBuilder};

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

/// Get a [`RenderList`].
#[must_use]
pub struct GetRenderList<'a> {
    ordr: &'a OrdrClient,
    fields: GetRenderListFields<'a>,
}

impl<'a> GetRenderList<'a> {
    pub(crate) const fn new(ordr: &'a OrdrClient) -> Self {
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

    /// Search by o!rdr username, can be used at the same time as [`replay_username`].
    ///
    /// [`replay_username`]: GetRenderList::replay_username
    pub fn ordr_username(&mut self, ordr_username: &'a str) -> &mut Self {
        self.fields.ordr_username = Some(ordr_username);

        self
    }

    /// Search by replay username, can be used at the same time as [`ordr_username`].
    ///
    /// [`ordr_username`]: GetRenderList::ordr_username
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

    fn request(&self) -> Result<Request, ClientError> {
        Request::builder(Route::RenderList)
            .query(&self.fields)
            .map(RequestBuilder::build)
    }
}

impl IntoFuture for &mut GetRenderList<'_> {
    type Output = Result<RenderList, ClientError>;
    type IntoFuture = OrdrFuture<RenderList>;

    fn into_future(self) -> Self::IntoFuture {
        match self.request() {
            Ok(request) => self.ordr.request(request),
            Err(err) => OrdrFuture::error(err),
        }
    }
}

impl IntoFuture for GetRenderList<'_> {
    type Output = Result<RenderList, ClientError>;
    type IntoFuture = OrdrFuture<RenderList>;

    fn into_future(mut self) -> Self::IntoFuture {
        (&mut self).into_future()
    }
}

#[cfg(test)]
mod tests {
    use hyper::Method;

    use crate::OrdrClient;

    use super::GetRenderList;

    fn path(list: &GetRenderList<'_>) -> String {
        let request = list
            .request()
            .unwrap_or_else(|err| panic!("query serialization failed: {err}"));

        assert_eq!(request.method, Method::GET);

        request.path
    }

    #[test]
    fn paging_and_render_id() {
        let client = OrdrClient::builder().build();

        let mut list = client.render_list();
        list.page_size(25).page(2).render_id(42);

        assert_eq!(path(&list), "renders?pageSize=25&page=2&renderID=42");
    }

    #[test]
    fn filters() {
        let client = OrdrClient::builder().build();

        let mut list = client.render_list();
        list.no_bots(true).link("pov8n").mapset_id(7);

        assert_eq!(path(&list), "renders?nobots=true&link=pov8n&beatmapsetid=7");
    }
}
