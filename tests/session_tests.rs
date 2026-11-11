mod common;
use common::{at, FakeCapturer};
use devlapse::config::parse_active_hours;
use devlapse::session::Session;
use devlapse::storage::Storage;

#[test]
fn tick_saves_one_file_per_monitor() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Session::new(FakeCapturer::new(2), Storage::new(dir.path().into()), "jpg", None);
    let r = s.tick(at(10, 0, 0));
    assert_eq!(r.saved.len(), 2);
    assert!(r.errors.is_empty());
    assert!(!r.skipped_inactive);
    assert!(r.saved.iter().all(|p| p.exists()));
}

#[test]
fn capture_failure_is_reported_and_next_tick_recovers() {
    let dir = tempfile::tempdir().unwrap();
    let mut cap = FakeCapturer::new(1);
    cap.fail_on_call = Some(1);
    let mut s = Session::new(cap, Storage::new(dir.path().into()), "jpg", None);

    let first = s.tick(at(10, 0, 0));
    assert!(first.saved.is_empty());
    assert_eq!(first.errors.len(), 1);
    assert!(first.errors[0].contains("display unavailable"));

    assert_eq!(s.tick(at(10, 1, 0)).saved.len(), 1);
}

#[test]
fn outside_active_hours_nothing_is_captured() {
    let dir = tempfile::tempdir().unwrap();
    let hours = Some(parse_active_hours("09:00-18:00").unwrap());
    let mut s = Session::new(FakeCapturer::new(1), Storage::new(dir.path().into()), "jpg", hours);

    let r = s.tick(at(20, 0, 0));
    assert!(r.skipped_inactive);
    assert!(r.saved.is_empty());
    assert_eq!(s.capturer().calls, 0, "screen must not even be grabbed off-hours");

    assert_eq!(s.tick(at(9, 30, 0)).saved.len(), 1);
}

#[test]
fn storage_failure_is_reported_per_frame() {
    let dir = tempfile::tempdir().unwrap();
    let blocker = dir.path().join("file");
    std::fs::write(&blocker, b"").unwrap();
    let mut s = Session::new(FakeCapturer::new(2), Storage::new(blocker.join("x")), "jpg", None);
    let r = s.tick(at(10, 0, 0));
    assert!(r.saved.is_empty());
    assert_eq!(r.errors.len(), 2);
}
