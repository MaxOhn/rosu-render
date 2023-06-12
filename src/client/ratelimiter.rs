use std::{sync::Arc, time::Duration};

use leaky_bucket::RateLimiter;

pub(super) struct Ratelimiter {
    pub(super) general: Arc<RateLimiter>,
    pub(super) send_render: Arc<RateLimiter>,
}

pub(crate) enum RatelimiterKind {
    General,
    SendRender,
}

impl Ratelimiter {
    pub fn new() -> Self {
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
                // 1 per 5 minutes
                RateLimiter::builder()
                    .max(1)
                    .initial(1)
                    .refill(1)
                    .interval(Duration::from_secs(300))
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
