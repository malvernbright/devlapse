use std::path::PathBuf;

use chrono::NaiveDateTime;

use crate::capture::ScreenCapturer;
use crate::config::ActiveHours;
use crate::storage::Storage;

#[derive(Debug, Default)]
pub struct TickReport {
    pub saved: Vec<PathBuf>,
    pub skipped_inactive: bool,
    pub errors: Vec<String>,
}

/// One capture-and-save step. Never panics: failures are reported so the
/// long-running loop survives a locked screen, a full disk, a monitor unplug, etc.
pub struct Session<C> {
    capturer: C,
    storage: Storage,
    ext: &'static str,
    active_hours: Option<ActiveHours>,
}

impl<C: ScreenCapturer> Session<C> {
    pub fn new(capturer: C, storage: Storage, ext: &'static str, active_hours: Option<ActiveHours>) -> Self {
        Self { capturer, storage, ext, active_hours }
    }

    pub fn capturer(&self) -> &C {
        &self.capturer
    }

    pub fn tick(&mut self, now: NaiveDateTime) -> TickReport {
        let mut report = TickReport::default();
        if let Some(h) = self.active_hours {
            if !h.contains(now.time()) {
                report.skipped_inactive = true;
                return report;
            }
        }
        match self.capturer.capture() {
            Err(e) => report.errors.push(format!("capture failed: {e}")),
            Ok(frames) => {
                for f in frames {
                    match self.storage.save(now, f.monitor, self.ext, &f.bytes) {
                        Ok(p) => report.saved.push(p),
                        Err(e) => report.errors.push(format!("save failed (screen{}): {e}", f.monitor + 1)),
                    }
                }
            }
        }
        report
    }
}
