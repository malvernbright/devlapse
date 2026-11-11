use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

use chrono::{NaiveTime, Timelike};

pub const USAGE: &str = "\
devlapse — capture your desktop on a timer for dev-life reels

USAGE:
    devlapse [OPTIONS]

OPTIONS:
    -i, --interval <DUR>        Time between captures: 30s, 5m, 1h, or seconds  [default: 60s]
    -o, --output <DIR>          Where to save captures  [default: ~/Desktop/DevLapse]
    -f, --format <jpg|png>      Image format (jpg is ~10x smaller)  [default: jpg]
    -a, --active-hours <RANGE>  Only capture inside this window, e.g. 09:00-18:00 or 22:00-02:00
    -n, --max-captures <N>      Stop after N captures (handy for a quick test run)
    -h, --help                  Show this help

Captures are saved as <DIR>/YYYY-MM-DD/YYYY-MM-DD_HH-MM-SS_screenN.<ext>
Stop with Ctrl+C.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
}

impl ImageFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
        }
    }
}

/// Daily window during which captures happen. `end` is exclusive; overnight windows wrap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveHours {
    pub start: NaiveTime,
    pub end: NaiveTime,
}

impl ActiveHours {
    pub fn contains(&self, t: NaiveTime) -> bool {
        if self.start < self.end {
            t >= self.start && t < self.end
        } else {
            t >= self.start || t < self.end
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub interval: Duration,
    pub output_dir: PathBuf,
    pub format: ImageFormat,
    pub active_hours: Option<ActiveHours>,
    pub max_captures: Option<u64>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseOutcome {
    Run(Config),
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError(pub String);

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for ConfigError {}

fn err<T>(msg: impl Into<String>) -> Result<T, ConfigError> {
    Err(ConfigError(msg.into()))
}

/// `~/Desktop/DevLapse`, respecting OS-specific Desktop locations (e.g. OneDrive on Windows).
pub fn default_output_dir() -> PathBuf {
    dirs::desktop_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join("Desktop")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("DevLapse")
}

pub fn parse_duration(s: &str) -> Result<Duration, ConfigError> {
    let s = s.trim();
    let (num, mult) = match s.chars().last() {
        Some('s') => (&s[..s.len() - 1], 1),
        Some('m') => (&s[..s.len() - 1], 60),
        Some('h') => (&s[..s.len() - 1], 3600),
        Some(c) if c.is_ascii_digit() => (s, 1),
        _ => return err(format!("invalid duration {s:?} (try 30s, 5m, 1h)")),
    };
    let n: u64 = num.parse().map_err(|_| ConfigError(format!("invalid duration {s:?} (try 30s, 5m, 1h)")))?;
    if n == 0 {
        return err("interval must be at least 1 second");
    }
    Ok(Duration::from_secs(n * mult))
}

fn parse_hhmm(s: &str) -> Option<NaiveTime> {
    let (h, m) = s.split_once(':')?;
    if h.len() != 2 || m.len() != 2 {
        return None;
    }
    NaiveTime::from_hms_opt(h.parse().ok()?, m.parse().ok()?, 0)
}

pub fn parse_active_hours(s: &str) -> Result<ActiveHours, ConfigError> {
    let bad = || ConfigError(format!("invalid active hours {s:?} (expected HH:MM-HH:MM)"));
    let (a, b) = s.split_once('-').ok_or_else(bad)?;
    let (start, end) = (parse_hhmm(a.trim()).ok_or_else(bad)?, parse_hhmm(b.trim()).ok_or_else(bad)?);
    if start.num_seconds_from_midnight() == end.num_seconds_from_midnight() {
        return err("active hours start and end must differ");
    }
    Ok(ActiveHours { start, end })
}

pub fn parse_args<I: IntoIterator<Item = String>>(args: I, default_dir: PathBuf) -> Result<ParseOutcome, ConfigError> {
    let mut cfg = Config {
        interval: Duration::from_secs(60),
        output_dir: default_dir,
        format: ImageFormat::Jpeg,
        active_hours: None,
        max_captures: None,
    };
    let mut it = args.into_iter();
    while let Some(flag) = it.next() {
        let mut value = |name: &str| it.next().ok_or_else(|| ConfigError(format!("{name} needs a value")));
        match flag.as_str() {
            "-h" | "--help" => return Ok(ParseOutcome::Help),
            "-i" | "--interval" => cfg.interval = parse_duration(&value(&flag)?)?,
            "-o" | "--output" => cfg.output_dir = PathBuf::from(value(&flag)?),
            "-f" | "--format" => {
                cfg.format = match value(&flag)?.to_ascii_lowercase().as_str() {
                    "jpg" | "jpeg" => ImageFormat::Jpeg,
                    "png" => ImageFormat::Png,
                    other => return err(format!("unsupported format {other:?} (use jpg or png)")),
                }
            }
            "-a" | "--active-hours" => cfg.active_hours = Some(parse_active_hours(&value(&flag)?)?),
            "-n" | "--max-captures" => {
                let v = value(&flag)?;
                match v.parse::<u64>() {
                    Ok(n) if n > 0 => cfg.max_captures = Some(n),
                    _ => return err(format!("--max-captures must be a positive number, got {v:?}")),
                }
            }
            other => return err(format!("unknown option {other:?} (see --help)")),
        }
    }
    Ok(ParseOutcome::Run(cfg))
}
