mod dynlink;
mod future;
mod render;
mod render_list;
mod requestable;
mod server_list;
mod server_online_count;
mod skin_custom;
mod skin_list;
mod user_preset;

use form_urlencoded::Serializer as FormSerializer;
use hyper::Method;
use serde::Serialize;
use serde_urlencoded::Serializer as UrlSerializer;

use crate::{client::RatelimiterKind, routing::Route, util::multipart::Form, ClientError};

pub(crate) use self::requestable::Requestable;

pub use self::{
    dynlink::GetDynLink, future::OrdrFuture, render::CommissionRender, render_list::GetRenderList,
    server_list::GetServerList, server_online_count::GetServerOnlineCount,
    skin_custom::GetSkinCustom, skin_list::GetSkinList, user_preset::GetUserPreset,
};

pub(crate) struct Request {
    pub(crate) form: Option<Form>,
    pub(crate) method: Method,
    pub(crate) path: String,
    pub(crate) ratelimiter: RatelimiterKind,
}

impl Request {
    pub fn builder(route: Route) -> RequestBuilder {
        RequestBuilder::new(route)
    }

    pub fn from_route(route: Route) -> Self {
        Self {
            form: None,
            method: route.method(),
            path: route.to_string(),
            ratelimiter: route.ratelimiter(),
        }
    }
}

pub(crate) struct RequestBuilder(Request);

impl RequestBuilder {
    pub fn new(route: Route) -> Self {
        Self(Request::from_route(route))
    }

    pub fn build(self) -> Request {
        self.0
    }

    pub fn form(mut self, form: Form) -> Self {
        self.0.form = Some(form);

        self
    }

    /// Add a query to the end of the path
    pub fn query(mut self, query: impl Serialize) -> Result<Self, ClientError> {
        fn form_serializer(path: &mut String) -> FormSerializer<'_, &mut String> {
            if !path.contains('?') {
                path.push('?');
            }

            FormSerializer::for_suffix(path, path.len())
        }

        let mut form_serializer = form_serializer(&mut self.0.path);
        let url_serializer = UrlSerializer::new(&mut form_serializer);
        query.serialize(url_serializer).map_err(ClientError::from)?;

        Ok(self)
    }
}
