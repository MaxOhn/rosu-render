use std::{
    num::NonZeroU64,
    time::{Duration, Instant},
};

/// Keeps track of successive reconnect attempts
/// and adds a delay based on exponential backoff.
pub(super) struct Reconnect {
    backoff_ms: Option<NonZeroU64>,
    last_attempt: Instant,
}

impl Reconnect {
    const MAX_BACKOFF_MS: u64 = 10_000;
    const RESET_INTERVAL: Duration = Duration::from_secs(60);

    pub(super) fn delay(&mut self) -> Option<Duration> {
        let backoff_ms = self.backoff_ms?;
        let now = Instant::now();

        if self.last_attempt + Self::RESET_INTERVAL > now {
            self.last_attempt = now;

            Some(Duration::from_millis(backoff_ms.get()))
        } else {
            self.backoff_ms = None;
            self.last_attempt = now;

            None
        }
    }

    /// Exponential backoff ms: 100 - 200 - 400 - 800 - 1600 - 3200 - 6400 - 10000
    pub(super) fn backoff(&mut self) {
        self.backoff_ms = match self.backoff_ms {
            Some(backoff_ms) => NonZeroU64::new((backoff_ms.get() * 2).min(Self::MAX_BACKOFF_MS)),
            None => NonZeroU64::new(100),
        };
    }
}

impl Default for Reconnect {
    fn default() -> Self {
        Self {
            backoff_ms: None,
            last_attempt: Instant::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::Reconnect;

    #[test]
    fn test_reconnect() {
        let mut reconnect = Reconnect::default();
        assert_eq!(reconnect.delay(), None);

        reconnect.backoff();
        assert_eq!(reconnect.delay(), Some(Duration::from_millis(100)));

        reconnect.backoff();
        assert_eq!(reconnect.delay(), Some(Duration::from_millis(200)));

        reconnect.backoff();
        assert_eq!(reconnect.delay(), Some(Duration::from_millis(400)));

        reconnect.backoff();
        assert_eq!(reconnect.delay(), Some(Duration::from_millis(800)));

        reconnect.backoff();
        assert_eq!(reconnect.delay(), Some(Duration::from_millis(1600)));

        reconnect.backoff();
        assert_eq!(reconnect.delay(), Some(Duration::from_millis(3200)));

        reconnect.backoff();
        assert_eq!(reconnect.delay(), Some(Duration::from_millis(6400)));

        reconnect.backoff();
        assert_eq!(reconnect.delay(), Some(Duration::from_millis(10000)));

        reconnect.backoff();
        assert_eq!(reconnect.delay(), Some(Duration::from_millis(10000)));

        reconnect.last_attempt = Instant::now() - Reconnect::RESET_INTERVAL;
        assert_eq!(reconnect.delay(), None);

        reconnect.backoff();
        assert_eq!(reconnect.delay(), Some(Duration::from_millis(100)));

        reconnect.backoff();
        assert_eq!(reconnect.delay(), Some(Duration::from_millis(200)));
    }
}
