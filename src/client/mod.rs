mod builder;
mod connector;
mod ratelimiter;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use hyper::{
    header::{CONTENT_LENGTH, CONTENT_TYPE, USER_AGENT},
    http::HeaderValue,
    Body, Client as HyperClient, Method, Request as HyperRequest,
};
use rust_socketio::asynchronous::Client as SocketIoClient;

pub use self::builder::OrdrBuilder;
pub(crate) use self::ratelimiter::RatelimiterKind;
use self::{connector::Connector, ratelimiter::Ratelimiter};

use crate::{
    error::Error,
    model::{RenderSkinOption, Verification},
    request::{
        GetRenderList, GetServerList, GetServerOnlineCount, GetSkinCustom, GetSkinList, OrdrFuture,
        Request, SendRender,
    },
};

const BASE_URL: &str = "https://apis.issou.best/ordr/";
const ROSU_RENDER_USER_AGENT: &str = concat!("rosu-render (", env!("CARGO_PKG_VERSION"), ")");

/// Client to access the o!rdr API and websocket.
///
/// Cheap to clone as the underlying http client and websocket will be re-used.
#[derive(Clone)]
pub struct Ordr {
    inner: Arc<OrdrRef>,
}

struct OrdrRef {
    pub(super) http: HyperClient<Connector>,
    pub(super) socket: Option<SocketIoClient>,
    pub(super) ratelimiter: Ratelimiter,
    // don't perform further requests when we're banned
    pub(super) banned: Arc<AtomicBool>,
    pub(super) verification: Option<Verification>,
}

impl Ordr {
    /// Create a new builder to create [`Ordr`].
    pub fn builder() -> OrdrBuilder {
        OrdrBuilder::new()
    }

    /// Disconnects the websocket. This should be called before dropping the [`Ordr`].
    pub async fn disconnect(self) -> Result<(), Error> {
        if let Some(ref socket) = self.inner.socket {
            socket.disconnect().await.map_err(Error::from)?;
        }

        Ok(())
    }

    /// Get info of a custom skin.
    ///
    /// You must provide the ID of the custom skin.
    pub const fn custom_skin_info(&self, id: u32) -> GetSkinCustom<'_> {
        GetSkinCustom::new(self, id)
    }

    /// Send a render request to o!rdr via replay file.
    pub const fn render_with_replay_file<'a>(
        &'a self,
        replay_file: &'a [u8],
        username: &'a str,
        skin: &'a RenderSkinOption<'a>,
    ) -> SendRender<'a> {
        SendRender::with_file(self, replay_file, username, skin)
    }

    /// Send a render request to o!rdr via replay url.
    pub const fn render_with_replay_url<'a>(
        &'a self,
        url: &'a str,
        username: &'a str,
        skin: &'a RenderSkinOption<'a>,
    ) -> SendRender<'a> {
        SendRender::with_url(self, url, username, skin)
    }

    /// Get a paginated list of all renders.
    pub const fn render_list(&self) -> GetRenderList<'_> {
        GetRenderList::new(self)
    }

    /// Get a list of available servers.
    pub const fn server_list(&self) -> GetServerList<'_> {
        GetServerList::new(self)
    }

    /// Get the amount of online servers.
    pub const fn server_online_count(&self) -> GetServerOnlineCount<'_> {
        GetServerOnlineCount::new(self)
    }

    /// Get a paginated list of all available skins.
    pub const fn skin_list(&self) -> GetSkinList<'_> {
        GetSkinList::new(self)
    }

    pub(crate) fn verification(&self) -> Option<&Verification> {
        self.inner.verification.as_ref()
    }

    pub(crate) fn request<T>(&self, req: Request) -> OrdrFuture<T> {
        self.try_request::<T>(req).unwrap_or_else(OrdrFuture::error)
    }

    fn try_request<T>(&self, req: Request) -> Result<OrdrFuture<T>, Error> {
        if self.inner.banned.load(Ordering::Relaxed) {
            return Err(Error::Unauthorized);
        }

        let Request {
            form,
            method,
            path,
            ratelimiter,
        } = req;

        let mut url = String::with_capacity(BASE_URL.len() + path.len());
        url.push_str(BASE_URL);
        url.push_str(&path);
        debug!(?url);

        debug_assert!(method != Method::POST || form.is_some());

        let mut builder = HyperRequest::builder().method(method).uri(&url);

        if let Some(headers) = builder.headers_mut() {
            if let Some(ref form) = form {
                headers.insert(CONTENT_LENGTH, HeaderValue::from(form.len()));

                if let Ok(content_type) = HeaderValue::try_from(form.content_type()) {
                    headers.insert(CONTENT_TYPE, content_type);
                }
            }

            headers.insert(USER_AGENT, HeaderValue::from_static(ROSU_RENDER_USER_AGENT));
        }

        let try_req = if let Some(form) = form {
            builder.body(Body::from(form.build()))
        } else {
            builder.body(Body::empty())
        };

        let req = try_req.map_err(|source| Error::BuildingRequest {
            source: Box::new(source),
        })?;

        let inner = self.inner.http.request(req);

        Ok(OrdrFuture::new(
            Box::pin(inner),
            Arc::clone(&self.inner.banned),
            self.inner.ratelimiter.get(ratelimiter).acquire_owned(1),
        ))
    }
}
