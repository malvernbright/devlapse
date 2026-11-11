use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use chrono::NaiveDateTime;

use crate::capture::ScreenCapturer;
use crate::session::{Session, TickReport};

/// How long a single sleep slice is; bounds Ctrl+C latency.
pub const STOP_POLL: Duration = Duration::from_millis(250);

pub trait Clock {
    /// Local wall-clock time, used for file names and active hours.
    fn wall(&self) -> NaiveDateTime;
    /// Monotonic time since an arbitrary origin, used for scheduling.
    fn monotonic(&self) -> Duration;
    fn sleep(&self, d: Duration);
}

pub struct SystemClock {
    origin: Instant,
}

impl SystemClock {
    pub fn new() -> Self {
        Self { origin: Instant::now() }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn wall(&self) -> NaiveDateTime {
        chrono::Local::now().naive_local()
    }
    fn monotonic(&self) -> Duration {
        self.origin.elapsed()
    }
    fn sleep(&self, d: Duration) {
        std::thread::sleep(d)
    }
}

/// Runs captures on a fixed grid (t0, t0+i, t0+2i…) so slow captures don't cause drift.
/// If we fall behind (slow capture, laptop asleep), missed slots are skipped, never burst.
/// Returns the number of ticks performed.
pub fn run<C, K, F>(
    session: &mut Session<C>,
    clock: &K,
    interval: Duration,
    stop: &AtomicBool,
    max_ticks: Option<u64>,
    mut on_tick: F,
) -> u64
where
    C: ScreenCapturer,
    K: Clock,
    F: FnMut(&TickReport),
{
    let origin = clock.monotonic();
    let step = interval.as_nanos().max(1);
    let mut ticks = 0u64;
    let mut slot = 0u128;

    while !stop.load(Ordering::SeqCst) {
        let report = session.tick(clock.wall());
        ticks += 1;
        on_tick(&report);
        if max_ticks.map_or(false, |m| ticks >= m) {
            break;
        }

        let elapsed = (clock.monotonic() - origin).as_nanos();
        slot = (slot + 1).max(elapsed / step + 1);
        let target = slot * step;
        let mut remaining = Duration::from_nanos((target - elapsed) as u64);

        while remaining > Duration::ZERO && !stop.load(Ordering::SeqCst) {
            let chunk = remaining.min(STOP_POLL);
            clock.sleep(chunk);
            remaining -= chunk;
        }
    }
    ticks
}
