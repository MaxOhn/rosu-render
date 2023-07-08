use std::future::IntoFuture;

use crate::{model::RenderServers, request::Request, routing::Route, ClientError, OrdrClient};

use super::OrdrFuture;

/// Get [`RenderServers`].
pub struct GetServerList<'a> {
    ordr: &'a OrdrClient,
}

impl<'a> GetServerList<'a> {
    pub(crate) const fn new(ordr: &'a OrdrClient) -> Self {
        Self { ordr }
    }
}

impl IntoFuture for &mut GetServerList<'_> {
    type Output = Result<RenderServers, ClientError>;
    type IntoFuture = OrdrFuture<RenderServers>;

    fn into_future(self) -> Self::IntoFuture {
        self.ordr.request(Request::from_route(Route::ServerList))
    }
}

impl IntoFuture for GetServerList<'_> {
    type Output = Result<RenderServers, ClientError>;
    type IntoFuture = OrdrFuture<RenderServers>;

    fn into_future(mut self) -> Self::IntoFuture {
        (&mut self).into_future()
    }
}
