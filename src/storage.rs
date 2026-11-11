use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use chrono::NaiveDateTime;

use crate::naming::{day_folder_name, file_name};

pub struct Storage {
    root: PathBuf,
}

impl Storage {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Writes `bytes` to `<root>/<day>/<timestamped name>`.
    /// Never overwrites (adds `-1`, `-2`…) and writes via temp file + rename so a crash
    /// or power loss can't leave a half-written image that breaks a video render.
    pub fn save(&self, ts: NaiveDateTime, monitor: usize, ext: &str, bytes: &[u8]) -> io::Result<PathBuf> {
        let dir = self.root.join(day_folder_name(ts));
        fs::create_dir_all(&dir)?;

        let base = file_name(ts, monitor, ext);
        let stem = base.trim_end_matches(&format!(".{ext}")).to_owned();
        let mut target = dir.join(&base);
        let mut n = 1;
        while target.exists() {
            target = dir.join(format!("{stem}-{n}.{ext}"));
            n += 1;
        }

        let tmp = target.with_extension(format!("{ext}.part"));
        let result = (|| {
            let mut f = fs::File::create(&tmp)?;
            f.write_all(bytes)?;
            f.sync_all()?;
            fs::rename(&tmp, &target)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&tmp);
        }
        result.map(|_| target)
    }
}
