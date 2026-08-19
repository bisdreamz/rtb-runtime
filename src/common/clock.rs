use crate::common::utils::epoch_timestamp;

/// A source of milliseconds, so time-dependent logic can be driven by something
/// other than the host clock.
///
/// This exists so components with time-based behaviour (rate histograms, decay
/// windows, TTL eviction) can be exercised deterministically or replayed far
/// faster than real time. Production code keeps using [`SystemClock`] and
/// behaves exactly as before.
pub trait Clock: Send + Sync {
    /// Milliseconds. Callers must only rely on differences between readings,
    /// not on the absolute value having any particular epoch.
    fn now_ms(&self) -> u64;
}

/// Wall-clock source. Identical to calling [`epoch_timestamp`] directly, and the
/// default everywhere so existing behaviour is unchanged.
///
/// Readings are NOT guaranteed monotonic: `SystemTime` can step backwards on
/// clock correction. Code that computes elapsed time with `saturating_sub` will
/// observe a stalled interval rather than an error. Use [`MonotonicClock`] where
/// that matters.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> u64 {
        epoch_timestamp()
    }
}

/// Monotonic source, measured from its own construction. Never steps backwards.
///
/// Its zero point is arbitrary, so readings are not comparable across instances
/// or with [`SystemClock`].
#[derive(Debug, Clone, Copy)]
pub struct MonotonicClock {
    base: std::time::Instant,
}

impl Default for MonotonicClock {
    fn default() -> Self {
        Self {
            base: std::time::Instant::now(),
        }
    }
}

impl Clock for MonotonicClock {
    fn now_ms(&self) -> u64 {
        self.base.elapsed().as_millis() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_clock_matches_epoch_timestamp() {
        let before = epoch_timestamp();
        let read = SystemClock.now_ms();
        let after = epoch_timestamp();
        assert!(read >= before && read <= after);
    }

    #[test]
    fn monotonic_clock_starts_at_zero_and_advances() {
        let c = MonotonicClock::default();
        assert!(c.now_ms() < 50);
        std::thread::sleep(std::time::Duration::from_millis(5));
        assert!(c.now_ms() >= 5);
    }
}
