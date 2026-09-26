use std::fmt::{Display, Formatter, Result as FmtResult};

use hyper::Method;

use crate::client::RatelimiterKind;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Route {
    DynLink,
    Render,
    RenderList,
    ServerList,
    ServerOnlineCount,
    SkinList,
    SkinCustom,
    UserPreset,
}

impl Route {
    pub fn method(self) -> Method {
        match self {
            Self::Render => Method::POST,
            Self::DynLink
            | Self::RenderList
            | Self::ServerList
            | Self::ServerOnlineCount
            | Self::SkinList
            | Self::SkinCustom
            | Self::UserPreset => Method::GET,
        }
    }

    pub fn ratelimiter(self) -> RatelimiterKind {
        match self {
            Route::Render => RatelimiterKind::SendRender,
            Route::DynLink
            | Route::RenderList
            | Route::ServerList
            | Route::ServerOnlineCount
            | Route::SkinList
            | Route::SkinCustom
            | Route::UserPreset => RatelimiterKind::General,
        }
    }
}

impl Display for Route {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            // `dynlink` lives outside the `/ordr/` base path, so it is absolute
            // to the API host.
            Self::DynLink => f.write_str("/dynlink/ordr/gen"),
            Self::Render | Self::RenderList => f.write_str("renders"),
            Self::ServerList => f.write_str("servers"),
            Self::ServerOnlineCount => f.write_str("servers/onlinecount"),
            Self::SkinList => f.write_str("skins"),
            Self::SkinCustom => f.write_str("skins/custom"),
            Self::UserPreset => f.write_str("presets/bot"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints() {
        for (route, path) in [
            (Route::DynLink, "/dynlink/ordr/gen"),
            (Route::Render, "renders"),
            (Route::RenderList, "renders"),
            (Route::ServerList, "servers"),
            (Route::ServerOnlineCount, "servers/onlinecount"),
            (Route::SkinList, "skins"),
            (Route::SkinCustom, "skins/custom"),
            (Route::UserPreset, "presets/bot"),
        ] {
            assert_eq!(route.to_string(), path);
            assert_eq!(
                route.method(),
                if route == Route::Render {
                    Method::POST
                } else {
                    Method::GET
                },
            );
            assert_eq!(
                route == Route::Render,
                matches!(route.ratelimiter(), RatelimiterKind::SendRender),
            );
        }
    }
}
