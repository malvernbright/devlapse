mod common;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use common::{at, FakeCapturer, FakeClock};
use devlapse::scheduler::run;
use devlapse::session::Session;
use devlapse::storage::Storage;

const MIN: Duration = Duration::from_secs(60);

#[test]
fn runs_exactly_max_ticks_spaced_by_interval() {
    let dir = tempfile::tempdir().unwrap();
    let clock = FakeClock::new(at(9, 0, 0));
    let mut s = Session::new(FakeCapturer::new(1), Storage::new(dir.path().into()), "jpg", None);
    let stop = AtomicBool::new(false);
    let saved = RefCell::new(vec![]);

    let ticks = run(&mut s, &clock, MIN, &stop, Some(3), |r| saved.borrow_mut().extend(r.saved.clone()));

    assert_eq!(ticks, 3);
    assert_eq!(clock.elapsed(), 2 * MIN, "no trailing sleep after the last capture");
    let names: Vec<_> = saved.borrow().iter().map(|p| p.file_name().unwrap().to_str().unwrap().to_owned()).collect();
    assert_eq!(names, vec!["2026-11-11_09-00-00_screen1.jpg",
                           "2026-11-11_09-01-00_screen1.jpg",
                           "2026-11-11_09-02-00_screen1.jpg"]);
}

#[test]
fn stop_flag_ends_the_loop() {
    let dir = tempfile::tempdir().unwrap();
    let clock = FakeClock::new(at(9, 0, 0));
    let mut s = Session::new(FakeCapturer::new(1), Storage::new(dir.path().into()), "jpg", None);
    let stop = AtomicBool::new(false);
    let mut seen = 0;

    let ticks = run(&mut s, &clock, MIN, &stop, None, |_| {
        seen += 1;
        if seen == 2 { stop.store(true, Ordering::SeqCst); }
    });
    assert_eq!(ticks, 2);
}

#[test]
fn stop_is_noticed_mid_sleep_within_a_quarter_second() {
    let dir = tempfile::tempdir().unwrap();
    let clock = FakeClock::new(at(9, 0, 0));
    let mut s = Session::new(FakeCapturer::new(1), Storage::new(dir.path().into()), "jpg", None);
    let stop = AtomicBool::new(false);
    let hour = Duration::from_secs(3600);

    // Ctrl+C arrives right after the first capture, while the loop sleeps for an hour.
    let ticks = run(&mut s, &clock, hour, &stop, None, |_| stop.store(true, Ordering::SeqCst));
    assert_eq!(ticks, 1);
    assert!(clock.elapsed() <= Duration::from_millis(250), "slept {:?}", clock.elapsed());
}

#[test]
fn schedule_does_not_drift_when_capture_takes_time() {
    let dir = tempfile::tempdir().unwrap();
    let clock = FakeClock::new(at(9, 0, 0));
    let mut cap = FakeCapturer::new(1);
    cap.cost = Some((clock.clone(), Duration::from_secs(7)));
    let mut s = Session::new(cap, Storage::new(dir.path().into()), "jpg", None);
    let stop = AtomicBool::new(false);
    let starts = RefCell::new(vec![]);

    run(&mut s, &clock, MIN, &stop, Some(4), |_| starts.borrow_mut().push(clock.elapsed() - Duration::from_secs(7)));
    assert_eq!(*starts.borrow(), vec![Duration::ZERO, MIN, 2 * MIN, 3 * MIN]);
}

#[test]
fn slow_capture_skips_missed_slots_instead_of_bursting() {
    let dir = tempfile::tempdir().unwrap();
    let clock = FakeClock::new(at(9, 0, 0));
    let mut cap = FakeCapturer::new(1);
    cap.cost = Some((clock.clone(), Duration::from_secs(150))); // 2.5 intervals, e.g. laptop lid closed
    let mut s = Session::new(cap, Storage::new(dir.path().into()), "jpg", None);
    let stop = AtomicBool::new(false);
    let starts = RefCell::new(vec![]);

    run(&mut s, &clock, MIN, &stop, Some(3), |_| starts.borrow_mut().push(clock.elapsed() - Duration::from_secs(150)));
    assert_eq!(*starts.borrow(), vec![Duration::ZERO, 3 * MIN, 6 * MIN]);
}
