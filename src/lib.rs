//! devlapse — periodic desktop screenshots for "day in the life of a developer" reels.
//!
//! The core (config, naming, storage, session, scheduler) is pure and fully unit-tested
//! with fakes. Real screen grabbing lives in `xcap_capturer`, behind the `capture` feature.

pub mod capture;
pub mod config;
pub mod naming;
pub mod scheduler;
pub mod session;
pub mod storage;

#[cfg(feature = "capture")]
pub mod xcap_capturer;
