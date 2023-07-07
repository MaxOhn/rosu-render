use std::sync::{atomic::AtomicBool, Arc};

use hyper::Client as HyperClient;

use crate::{client::connector, model::Verification};

use super::{ratelimiter::Ratelimiter, OrdrClient, OrdrRef};

/// A builder for [`OrdrClient`].
#[derive(Default)]
pub struct OrdrClientBuilder {
    verification: Option<Verification>,
    ratelimit: Option<RatelimitBuilder>,
}

impl OrdrClientBuilder {
    /// Create a new builder to create a [`OrdrClient`].
    pub fn new() -> Self {
        Self::default()
    }

    //// Build an [`OrdrClient`].
    pub fn build(self) -> OrdrClient {
        let connector = connector::create();
        let http = HyperClient::builder().build(connector);

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
            (Some(Verification::Key(_)), None) => RatelimitBuilder::new(10_000, 1, 1), // One per 10 seconds
            (
                Some(
                    Verification::DevModeSuccess
                    | Verification::DevModeFail
                    | Verification::DevModeWsFail,
                ),
                None,
            ) => RatelimitBuilder::new(1000, 1, 1), // One per second
            (Some(_), Some(ratelimit)) => ratelimit,
        };

        OrdrClient {
            inner: Arc::new(OrdrRef {
                http,
                ratelimiter: Ratelimiter::new(ratelimit),
                verification: self.verification,
                banned: Arc::new(AtomicBool::new(false)),
            }),
        }
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
    /// If a verification key is specified, the ratelimit defaults to one per 10 seconds.
    ///
    /// # Panics
    ///
    /// Panics if `interval_ms` or `refill` are zero.
    ///
    /// # Example
    /// ```
    /// use rosu_render::OrdrClient;
    ///
    /// // Applying a ratelimit of 1 refill every 5 seconds, up to 2 charges
    /// // which means 2 requests per 10 seconds.
    /// let client = OrdrClient::builder()
    ///     .render_ratelimit(5000, 1, 2)
    ///     .build();
    /// ```
    pub fn render_ratelimit(self, interval_ms: u64, refill: u64, max: u64) -> Self {
        Self {
            ratelimit: Some(RatelimitBuilder::new(interval_ms, refill, max)),
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
