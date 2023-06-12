use std::future::IntoFuture;

use crate::{error::Error, request::Request, routing::Route, Ordr};

use super::OrdrFuture;

// TODO: docs
pub struct GetServerOnlineCount<'a> {
    ordr: &'a Ordr,
}

impl<'a> GetServerOnlineCount<'a> {
    pub(crate) const fn new(ordr: &'a Ordr) -> Self {
        Self { ordr }
    }
}

impl IntoFuture for &mut GetServerOnlineCount<'_> {
    type Output = Result<u32, Error>;
    type IntoFuture = OrdrFuture<u32>;

    fn into_future(self) -> Self::IntoFuture {
        self.ordr
            .request(Request::from_route(Route::ServerOnlineCount))
    }
}

impl IntoFuture for GetServerOnlineCount<'_> {
    type Output = Result<u32, Error>;
    type IntoFuture = OrdrFuture<u32>;

    fn into_future(mut self) -> Self::IntoFuture {
        (&mut self).into_future()
    }
}
