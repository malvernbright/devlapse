use std::path::PathBuf;
use std::time::Duration;

use chrono::NaiveTime;
use devlapse::config::{parse_active_hours, parse_args, parse_duration, Config, ImageFormat, ParseOutcome};

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}
fn default_dir() -> PathBuf {
    PathBuf::from("/home/dev/Desktop/DevLapse")
}
fn run_cfg(list: &[&str]) -> Config {
    match parse_args(args(list), default_dir()).expect("should parse") {
        ParseOutcome::Run(c) => c,
        ParseOutcome::Help => panic!("unexpected help"),
    }
}
fn t(h: u32, m: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(h, m, 0).unwrap()
}

#[test]
fn durations_accept_seconds_minutes_hours_and_bare_numbers() {
    assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
    assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
    assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
    assert_eq!(parse_duration("45").unwrap(), Duration::from_secs(45));
}

#[test]
fn durations_reject_garbage_and_zero() {
    for bad in ["", "abc", "10x", "-5s", "0s", "0"] {
        assert!(parse_duration(bad).is_err(), "{bad:?} should be rejected");
    }
}

#[test]
fn defaults_are_sensible() {
    let c = run_cfg(&[]);
    assert_eq!(c.interval, Duration::from_secs(60));
    assert_eq!(c.output_dir, default_dir());
    assert_eq!(c.format, ImageFormat::Jpeg);
    assert_eq!(c.active_hours, None);
    assert_eq!(c.max_captures, None);
}

#[test]
fn all_flags_are_parsed() {
    let c = run_cfg(&["--interval", "10s", "--output", "/tmp/caps", "--format", "png",
                      "--active-hours", "09:00-18:30", "--max-captures", "5"]);
    assert_eq!(c.interval, Duration::from_secs(10));
    assert_eq!(c.output_dir, PathBuf::from("/tmp/caps"));
    assert_eq!(c.format, ImageFormat::Png);
    let ah = c.active_hours.unwrap();
    assert_eq!((ah.start, ah.end), (t(9, 0), t(18, 30)));
    assert_eq!(c.max_captures, Some(5));
}

#[test]
fn short_flags_work() {
    let c = run_cfg(&["-i", "2m", "-o", "/tmp/x"]);
    assert_eq!(c.interval, Duration::from_secs(120));
    assert_eq!(c.output_dir, PathBuf::from("/tmp/x"));
}

#[test]
fn help_flag_returns_help() {
    assert!(matches!(parse_args(args(&["--help"]), default_dir()), Ok(ParseOutcome::Help)));
    assert!(matches!(parse_args(args(&["-h"]), default_dir()), Ok(ParseOutcome::Help)));
}

#[test]
fn bad_input_is_an_error_not_a_panic() {
    for bad in [vec!["--interval"], vec!["--bogus"], vec!["--format", "gif"],
                vec!["--max-captures", "0"], vec!["--max-captures", "lots"]] {
        assert!(parse_args(args(&bad), default_dir()).is_err(), "{bad:?} should be rejected");
    }
}

#[test]
fn format_maps_to_extension() {
    assert_eq!(ImageFormat::Png.extension(), "png");
    assert_eq!(ImageFormat::Jpeg.extension(), "jpg");
}

#[test]
fn active_hours_daytime_window() {
    let ah = parse_active_hours("09:00-18:00").unwrap();
    assert!(!ah.contains(t(8, 59)));
    assert!(ah.contains(t(9, 0)));
    assert!(ah.contains(t(17, 59)));
    assert!(!ah.contains(t(18, 0)), "end is exclusive");
}

#[test]
fn active_hours_overnight_window_for_night_owls() {
    let ah = parse_active_hours("22:00-02:00").unwrap();
    assert!(ah.contains(t(23, 30)));
    assert!(ah.contains(t(1, 0)));
    assert!(!ah.contains(t(12, 0)));
}

#[test]
fn active_hours_reject_malformed() {
    for bad in ["9-5", "25:00-26:00", "09:00", "09:00-09:00", "aa:bb-cc:dd"] {
        assert!(parse_active_hours(bad).is_err(), "{bad:?} should be rejected");
    }
}
