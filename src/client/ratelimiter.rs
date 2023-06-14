use std::{sync::Arc, time::Duration};

use leaky_bucket::RateLimiter;

use super::builder::RatelimitBuilder;

pub(super) struct Ratelimiter {
    pub(super) general: Arc<RateLimiter>,
    pub(super) send_render: Arc<RateLimiter>,
}

pub(crate) enum RatelimiterKind {
    General,
    SendRender,
}

impl Ratelimiter {
    pub fn new(builder: RatelimitBuilder) -> Self {
        let RatelimitBuilder {
            interval,
            refill,
            max,
        } = builder;

        info!("o!rdr ratelimit: Refill {refill} every {interval}ms, up to {max}");

        Self {
            general: Arc::new(
                // 10 per minute
                RateLimiter::builder()
                    .max(10)
                    .initial(10)
                    .refill(1)
                    .interval(Duration::from_secs(6))
                    .build(),
            ),
            send_render: Arc::new(
                RateLimiter::builder()
                    .max(max as usize)
                    .initial(max as usize)
                    .refill(refill as usize)
                    .interval(Duration::from_millis(interval))
                    .build(),
            ),
        }
    }

    pub fn get(&self, kind: RatelimiterKind) -> Arc<RateLimiter> {
        match kind {
            RatelimiterKind::General => Arc::clone(&self.general),
            RatelimiterKind::SendRender => Arc::clone(&self.send_render),
        }
    }
}
