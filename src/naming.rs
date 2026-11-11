use chrono::NaiveDateTime;

/// Per-day folder, e.g. `2026-11-11`. One folder per day = one reel per day.
pub fn day_folder_name(ts: NaiveDateTime) -> String {
    ts.format("%Y-%m-%d").to_string()
}

/// e.g. `2026-11-11_09-05-07_screen1.jpg`. Zero-padded, so lexical order == time order
/// (ffmpeg's `-pattern_type glob` relies on this). No colons: they're illegal on Windows.
pub fn file_name(ts: NaiveDateTime, monitor: usize, ext: &str) -> String {
    format!("{}_screen{}.{}", ts.format("%Y-%m-%d_%H-%M-%S"), monitor + 1, ext)
}
