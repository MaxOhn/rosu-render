use std::future::IntoFuture;

use crate::{error::Error, model::RenderServers, request::Request, routing::Route, Ordr};

use super::OrdrFuture;

// TODO: docs
pub struct GetServerList<'a> {
    ordr: &'a Ordr,
}

impl<'a> GetServerList<'a> {
    pub(crate) const fn new(ordr: &'a Ordr) -> Self {
        Self { ordr }
    }
}

impl IntoFuture for &mut GetServerList<'_> {
    type Output = Result<RenderServers, Error>;
    type IntoFuture = OrdrFuture<RenderServers>;

    fn into_future(self) -> Self::IntoFuture {
        self.ordr.request(Request::from_route(Route::ServerList))
    }
}

impl IntoFuture for GetServerList<'_> {
    type Output = Result<RenderServers, Error>;
    type IntoFuture = OrdrFuture<RenderServers>;

    fn into_future(mut self) -> Self::IntoFuture {
        (&mut self).into_future()
    }
}
