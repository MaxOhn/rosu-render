use std::sync::{atomic::AtomicBool, Arc};

use futures::{future::BoxFuture, FutureExt};
use hyper::Client as HyperClient;
use rust_socketio::{asynchronous::ClientBuilder as SocketBuilder, Payload};

use crate::{
    client::connector,
    error::Error,
    model::{RenderDone, RenderFail, RenderProgress, Verification},
};

use super::{ratelimiter::Ratelimiter, Ordr, OrdrRef};

macro_rules! wrap_fn {
    ($f:ident, $event:literal) => {
        move |payload: Payload, _| {
            match payload {
                Payload::String(payload) => match serde_json::from_str(&payload) {
                    Ok(progress) => return ($f)(progress),
                    Err(err) => error!(
                        %err,
                        %payload,
                        concat!("Failed to deserialize ", $event, " payload"),
                    ),
                },
                Payload::Binary(payload) => error!(
                    ?payload,
                    concat!($event, " received binary payload, expected string")
                ),
            }

            futures::future::ready(()).boxed()
        }
    };
}

const WS_URL: &str = "https://ordr-ws.issou.best";

/// A builder for [`Ordr`].
pub struct OrdrBuilder {
    socket_builder: SocketBuilder,
    verification: Option<Verification>,
    ratelimit: Option<RatelimitBuilder>,
    with_socket: Option<bool>,
}

impl OrdrBuilder {
    /// Create a new builder to create a [`Ordr`].
    pub fn new() -> Self {
        Self {
            socket_builder: SocketBuilder::new(WS_URL),
            verification: None,
            ratelimit: None,
            with_socket: None,
        }
    }

    //// Build an [`Ordr`].
    pub async fn build(self) -> Result<Ordr, Error> {
        let connector = connector::create();
        let http = HyperClient::builder().build(connector);

        let socket = if self.with_socket.unwrap_or(true) {
            // TODO: tls_config
            let socket = self.socket_builder.connect().await?;
            info!("Connected websocket");

            Some(socket)
        } else {
            None
        };

        let ratelimit = match (self.verification.as_ref(), self.ratelimit) {
            (None, None) => RatelimitBuilder::new(300_000, 1, 1), // One per 5 minutes
            (None, Some(ratelimit)) => {
                let ms_per_gain = ratelimit.interval / ratelimit.refill;

                if ms_per_gain < 300_000 {
                    RatelimitBuilder::new(300_000, 1, 1)
                } else {
                    RatelimitBuilder {
                        max: ratelimit.max.min(2),
                        ..ratelimit
                    }
                }
            }
            (Some(Verification::Key(_)), None) => todo!(),
            (
                Some(
                    Verification::DevModeSuccess
                    | Verification::DevModeFail
                    | Verification::DevModeWsFail,
                ),
                None,
            ) => RatelimitBuilder::new(1000, 1, 1), // One per 2 seconds
            (Some(_), Some(ratelimit)) => ratelimit,
        };

        Ok(Ordr {
            inner: Arc::new(OrdrRef {
                http,
                socket,
                ratelimiter: Ratelimiter::new(ratelimit),
                verification: self.verification,
                banned: Arc::new(AtomicBool::new(false)),
            }),
        })
    }

    /// Specify a [`Verification`]
    ///
    /// Refer to its documentation for more information.
    pub fn verification(self, verification: Verification) -> Self {
        Self {
            verification: Some(verification),
            ..self
        }
    }

    /// Specify a ratelimit that the client will uphold for the render endpoint.
    /// Other endpoints won't be affected, they have a pre-set ratelimit.
    ///
    /// - `interval_ms`: How many milliseconds until the next refill
    /// - `refill`: How many allowances are added per refill
    /// - `max`: What's the maximum amount of available allowances
    ///
    /// If no [`Verification`] is specified, the ratelimit will be clamped up to one
    /// per 5 minutes as per o!rdr rules.
    /// If a dev mode [`Verification`] is specified, the ratelimit defaults to one per second.
    ///
    /// # Panics
    ///
    /// Panics if `interval_ms` or `refill` are zero.
    pub fn render_ratelimit(self, interval_ms: u64, refill: u64, max: u64) -> Self {
        Self {
            ratelimit: Some(RatelimitBuilder::new(interval_ms, refill, max)),
            ..self
        }
    }

    /// Specify whether a connection to the websocket should be build.
    ///
    /// Defaults to `true`.
    pub fn with_websocket(self, with_websocket: bool) -> Self {
        Self {
            with_socket: Some(with_websocket),
            ..self
        }
    }

    /// Specify what happens when there is progress on a render.
    pub fn on_render_progress<F>(self, mut f: F) -> Self
    where
        F: FnMut(RenderProgress) -> BoxFuture<'static, ()> + 'static + Send + Sync,
    {
        let f = wrap_fn!(f, "on_render_progress");

        Self {
            socket_builder: self.socket_builder.on("render_progress_json", f),
            ..self
        }
    }

    /// Specify what happens when a render finishes.
    pub fn on_render_done<F>(self, mut f: F) -> Self
    where
        F: FnMut(RenderDone) -> BoxFuture<'static, ()> + 'static + Send + Sync,
    {
        let f = wrap_fn!(f, "on_render_done");

        Self {
            socket_builder: self.socket_builder.on("render_done_json", f),
            ..self
        }
    }

    /// Specify what happens when a render failed.
    pub fn on_render_failed<F>(self, mut f: F) -> Self
    where
        F: FnMut(RenderFail) -> BoxFuture<'static, ()> + 'static + Send + Sync,
    {
        let f = wrap_fn!(f, "on_render_failed");

        Self {
            socket_builder: self.socket_builder.on("render_failed_json", f),
            ..self
        }
    }
}

pub(super) struct RatelimitBuilder {
    pub interval: u64,
    pub refill: u64,
    pub max: u64,
}

impl RatelimitBuilder {
    fn new(interval: u64, refill: u64, max: u64) -> Self {
        Self {
            interval,
            refill,
            max,
        }
    }
}
