use super::entry::MAX_SAFE_TICK_DURATION;
use crate::time::{Clock, Duration, Instant};

/// A structure which handles conversion from Instants to `u64` timestamps.
/// In paused mode it handles nanosecond resolution, in live mode millisecond resolution
#[derive(Debug)]
pub(crate) struct TimeSource {
    start_time: Instant,
    can_pause: bool,
}

impl TimeSource {
    pub(crate) fn new(clock: &Clock) -> Self {
        Self {
            start_time: clock.now(),
            can_pause: clock.can_pause(),
        }
    }

    pub(crate) fn deadline_to_tick(&self, t: Instant) -> u64 {
        if !self.can_pause {
            // Round up to the end of a ms
            self.instant_to_tick(t + Duration::from_nanos(999_999))
        } else {
            self.instant_to_tick(t)
        }
    }

    pub(crate) fn instant_to_tick(&self, t: Instant) -> u64 {
        let dur: Duration = t.saturating_duration_since(self.start_time);
        if !self.can_pause {
            // round up
            dur.as_millis().try_into().unwrap_or(MAX_SAFE_TICK_DURATION)
        } else {
            dur.as_nanos().try_into().unwrap_or(MAX_SAFE_TICK_DURATION)
        }
    }

    pub(crate) fn tick_to_duration(&self, t: u64) -> Duration {
        if !self.can_pause {
            Duration::from_millis(t)
        } else {
            Duration::from_nanos(t)
        }
    }

    pub(crate) fn now(&self, clock: &Clock) -> u64 {
        self.instant_to_tick(clock.now())
    }
}
