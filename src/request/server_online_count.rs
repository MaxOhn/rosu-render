use std::future::IntoFuture;

use crate::{model::ServerOnlineCount, request::Request, routing::Route, ClientError, OrdrClient};

use super::OrdrFuture;

// TODO: docs
pub struct GetServerOnlineCount<'a> {
    ordr: &'a OrdrClient,
}

impl<'a> GetServerOnlineCount<'a> {
    pub(crate) const fn new(ordr: &'a OrdrClient) -> Self {
        Self { ordr }
    }
}

impl IntoFuture for &mut GetServerOnlineCount<'_> {
    type Output = Result<ServerOnlineCount, ClientError>;
    type IntoFuture = OrdrFuture<ServerOnlineCount>;

    fn into_future(self) -> Self::IntoFuture {
        self.ordr
            .request(Request::from_route(Route::ServerOnlineCount))
    }
}

impl IntoFuture for GetServerOnlineCount<'_> {
    type Output = Result<ServerOnlineCount, ClientError>;
    type IntoFuture = OrdrFuture<ServerOnlineCount>;

    fn into_future(mut self) -> Self::IntoFuture {
        (&mut self).into_future()
    }
}
