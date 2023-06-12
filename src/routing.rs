use std::fmt::{Display, Formatter, Result as FmtResult};

use hyper::Method;

use crate::client::RatelimiterKind;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Route {
    Render,
    RenderList,
    ServerList,
    ServerOnlineCount,
    SkinList,
    SkinCustom,
}

impl Route {
    pub fn method(self) -> Method {
        match self {
            Self::Render => Method::POST,
            Self::RenderList
            | Self::ServerList
            | Self::ServerOnlineCount
            | Self::SkinList
            | Self::SkinCustom => Method::GET,
        }
    }

    pub fn ratelimiter(self) -> RatelimiterKind {
        match self {
            Route::Render => RatelimiterKind::SendRender,
            Route::RenderList
            | Route::ServerList
            | Route::ServerOnlineCount
            | Route::SkinList
            | Route::SkinCustom => RatelimiterKind::General,
        }
    }
}

impl Display for Route {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Render | Self::RenderList => f.write_str("renders"),
            Self::ServerList => f.write_str("servers"),
            Self::ServerOnlineCount => f.write_str("servers/onlinecount"),
            Self::SkinList => f.write_str("skins"),
            Self::SkinCustom => f.write_str("skins/custom"),
        }
    }
}
