//! Test doubles shared by the integration tests.
#![allow(dead_code)]
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use chrono::{NaiveDate, NaiveDateTime};
use devlapse::capture::{Frame, ScreenCapturer};
use devlapse::scheduler::Clock;

pub fn at(h: u32, m: u32, s: u32) -> NaiveDateTime {
    NaiveDate::from_ymd_opt(2026, 11, 11).unwrap().and_hms_opt(h, m, s).unwrap()
}

/// Deterministic clock: time only moves when someone sleeps or calls `advance`.
#[derive(Clone)]
pub struct FakeClock {
    start_wall: NaiveDateTime,
    elapsed: Rc<Cell<Duration>>,
}

impl FakeClock {
    pub fn new(start_wall: NaiveDateTime) -> Self {
        Self { start_wall, elapsed: Rc::new(Cell::new(Duration::ZERO)) }
    }
    pub fn advance(&self, d: Duration) {
        self.elapsed.set(self.elapsed.get() + d);
    }
    pub fn elapsed(&self) -> Duration {
        self.elapsed.get()
    }
}

impl Clock for FakeClock {
    fn wall(&self) -> NaiveDateTime {
        self.start_wall + chrono::Duration::from_std(self.elapsed.get()).unwrap()
    }
    fn monotonic(&self) -> Duration {
        self.elapsed.get()
    }
    fn sleep(&self, d: Duration) {
        self.advance(d);
    }
}

/// Capturer that returns `monitors` fake frames, optionally failing or taking time.
pub struct FakeCapturer {
    pub monitors: usize,
    pub calls: u32,
    pub fail_on_call: Option<u32>,
    pub cost: Option<(FakeClock, Duration)>,
}

impl FakeCapturer {
    pub fn new(monitors: usize) -> Self {
        Self { monitors, calls: 0, fail_on_call: None, cost: None }
    }
}

impl ScreenCapturer for FakeCapturer {
    fn capture(&mut self) -> Result<Vec<Frame>, String> {
        self.calls += 1;
        if let Some((clock, d)) = &self.cost {
            clock.advance(*d);
        }
        if self.fail_on_call == Some(self.calls) {
            return Err("display unavailable".into());
        }
        Ok((0..self.monitors)
            .map(|i| Frame { monitor: i, bytes: format!("frame{}-screen{}", self.calls, i + 1).into_bytes() })
            .collect())
    }
}
