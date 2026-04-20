use chrono::Local;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

const MAX_LOG_DAYS: i64 = 7;

struct Logger {
    log_dir: PathBuf,
    current_date: String,
    enabled: bool,
}

static LOGGER: LazyLock<Mutex<Logger>> = LazyLock::new(|| {
    Mutex::new(Logger {
        log_dir: PathBuf::new(),
        current_date: String::new(),
        enabled: true,
    })
});

impl Logger {
    fn log_path(&self) -> PathBuf {
        self.log_dir.join(format!("vtt-{}.log", self.current_date))
    }

    fn ensure_date(&mut self) {
        let today = Local::now().format("%Y-%m-%d").to_string();
        if today != self.current_date {
            self.current_date = today;
        }
    }
}

pub fn init(log_dir: &Path) {
    fs::create_dir_all(log_dir).ok();
    {
        let mut logger = LOGGER.lock().unwrap();
        logger.log_dir = log_dir.to_path_buf();
        logger.ensure_date();
    }
    purge_old_logs(log_dir);
}

pub fn log(msg: &str) {
    let mut logger = LOGGER.lock().unwrap();
    if !logger.enabled || logger.log_dir.as_os_str().is_empty() {
        return;
    }

    logger.ensure_date();
    let path = logger.log_path();
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let line = format!("[{}] {}\n", timestamp, msg);

    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(line.as_bytes());
    }

    print!("{}", line);
}

pub fn set_enabled(enabled: bool) {
    LOGGER.lock().unwrap().enabled = enabled;
}

pub fn get_dir() -> PathBuf {
    LOGGER.lock().unwrap().log_dir.clone()
}

fn purge_old_logs(log_dir: &Path) {
    let cutoff = Local::now() - chrono::Duration::days(MAX_LOG_DAYS);
    let cutoff_ts = cutoff.timestamp();

    if let Ok(entries) = fs::read_dir(log_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !is_daily_log_filename(&name) {
                continue;
            }
            if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    let ts = modified
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0);
                    if ts < cutoff_ts {
                        fs::remove_file(entry.path()).ok();
                    }
                }
            }
        }
    }

    remove_legacy_vtt_logs(log_dir);
}

/// Delete the pre-2.0 single-file log + its rotation siblings. Pre-2.0 used
/// one rolling `vtt.log` / `vtt.log.1` / `vtt.log.2` / `vtt.log.3` — 2.0 moved
/// to per-day files (vtt-YYYY-MM-DD.log). After an upgrade the legacy files
/// hang around otherwise, confusing the Logs submenu filter.
fn remove_legacy_vtt_logs(log_dir: &Path) {
    fs::remove_file(log_dir.join("vtt.log")).ok();
    for i in 1..=3 {
        fs::remove_file(log_dir.join(format!("vtt.log.{}", i))).ok();
    }
}

/// Pure: does the filename look like one of our daily log files
/// (`vtt-YYYY-MM-DD.log`)? Used by purge_old_logs to decide what to touch.
fn is_daily_log_filename(name: &str) -> bool {
    name.starts_with("vtt-") && name.ends_with(".log")
}

/// Convenience macro for formatted logging
#[macro_export]
macro_rules! vtt_log {
    ($($arg:tt)*) => {
        $crate::logging::log(&format!($($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_daily_log_filename_accepts_real_format() {
        assert!(is_daily_log_filename("vtt-2026-04-20.log"));
        assert!(is_daily_log_filename("vtt-2025-12-31.log"));
    }

    #[test]
    fn is_daily_log_filename_rejects_legacy_and_unrelated() {
        assert!(!is_daily_log_filename("vtt.log"));
        assert!(!is_daily_log_filename("vtt.log.1"));
        assert!(!is_daily_log_filename("vtt-linux.pid"));
        assert!(!is_daily_log_filename("settings.conf"));
        assert!(!is_daily_log_filename("vtt-linux.lock"));
        assert!(!is_daily_log_filename(""));
    }

    #[test]
    fn is_daily_log_filename_accepts_liberal_middle() {
        // Intentionally permissive — we key on prefix+suffix so any date or
        // marker in the middle is fine. This matches the current purge logic.
        assert!(is_daily_log_filename("vtt-anything.log"));
        assert!(is_daily_log_filename("vtt-.log"));
    }

    #[test]
    fn remove_legacy_vtt_logs_deletes_all_rotation_siblings() {
        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        let root = dir.path();

        // Create the four legacy files the pre-2.0 logger could produce.
        fs::write(root.join("vtt.log"), b"legacy current").unwrap();
        fs::write(root.join("vtt.log.1"), b"legacy rotated 1").unwrap();
        fs::write(root.join("vtt.log.2"), b"legacy rotated 2").unwrap();
        fs::write(root.join("vtt.log.3"), b"legacy rotated 3").unwrap();

        // Also create a daily file and an unrelated file that must survive.
        fs::write(root.join("vtt-2026-04-20.log"), b"daily").unwrap();
        fs::write(root.join("settings.conf"), b"unrelated").unwrap();

        remove_legacy_vtt_logs(root);

        assert!(!root.join("vtt.log").exists());
        assert!(!root.join("vtt.log.1").exists());
        assert!(!root.join("vtt.log.2").exists());
        assert!(!root.join("vtt.log.3").exists());
        assert!(
            root.join("vtt-2026-04-20.log").exists(),
            "daily log must survive legacy cleanup"
        );
        assert!(
            root.join("settings.conf").exists(),
            "unrelated files must survive"
        );
    }

    #[test]
    fn remove_legacy_vtt_logs_missing_files_is_safe_noop() {
        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        // Empty directory — none of the legacy files exist. Should not panic
        // or error; the .ok() swallows ENOENT.
        remove_legacy_vtt_logs(dir.path());
        // Sanity: directory is still empty and exists.
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
    }
}
