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

/// Daily log filenames, newest first.
///
/// Shared by both tray implementations (TASK-VTT098) so the Logs menu is the
/// same list on every platform rather than two drifting copies. Filenames carry
/// an ISO date, so a reversed lexicographic sort is newest-first.
/// `Err` carries a human-readable reason for the menu to display.
pub fn list_log_filenames() -> Result<Vec<String>, String> {
    let entries = fs::read_dir(get_dir()).map_err(|e| e.to_string())?;
    let mut files: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            is_daily_log_filename(&name).then_some(name)
        })
        .collect();
    files.sort_unstable();
    files.reverse();
    Ok(files)
}

/// Menu label for a daily log file: "Today (04-20)", "Yesterday (04-19)", or
/// the bare ISO date. Pure so it is testable without touching the filesystem.
pub fn format_log_label(filename: &str, today: &str, yesterday: &str) -> String {
    let date = filename
        .strip_prefix("vtt-")
        .and_then(|s| s.strip_suffix(".log"))
        .unwrap_or(filename);

    if date == today && date.len() >= 7 {
        format!("Today ({})", &date[5..])
    } else if date == yesterday && date.len() >= 7 {
        format!("Yesterday ({})", &date[5..])
    } else {
        date.to_string()
    }
}

/// Today's and yesterday's ISO dates, the two `format_log_label` compares against.
pub fn today_and_yesterday() -> (String, String) {
    let now = Local::now();
    (
        now.format("%Y-%m-%d").to_string(),
        (now - chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string(),
    )
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
    fn format_log_label_today_gives_friendly_label() {
        let s = format_log_label("vtt-2026-04-20.log", "2026-04-20", "2026-04-19");
        assert_eq!(s, "Today (04-20)");
    }

    #[test]
    fn format_log_label_yesterday_gives_friendly_label() {
        let s = format_log_label("vtt-2026-04-19.log", "2026-04-20", "2026-04-19");
        assert_eq!(s, "Yesterday (04-19)");
    }

    #[test]
    fn format_log_label_older_returns_full_iso_date() {
        let s = format_log_label("vtt-2026-01-15.log", "2026-04-20", "2026-04-19");
        assert_eq!(s, "2026-01-15");
    }

    #[test]
    fn format_log_label_malformed_filename_returns_filename() {
        let s = format_log_label("not-a-log-file.txt", "2026-04-20", "2026-04-19");
        assert_eq!(s, "not-a-log-file.txt");
    }

    #[test]
    fn format_log_label_wrong_prefix_returns_filename() {
        let s = format_log_label("vtt2026-04-20.log", "2026-04-20", "2026-04-19");
        assert_eq!(
            s, "vtt2026-04-20.log",
            "filename without proper vtt- prefix falls through"
        );
    }

    #[test]
    fn format_log_label_short_dates_dont_panic() {
        // Sanity check the bounds guard — a "today" of "2026" (len 4) must not panic
        // from string slicing at [5..].
        let s = format_log_label("vtt-2026.log", "2026", "2025");
        assert_eq!(
            s, "2026",
            "short dates bypass the friendly label and fall through"
        );
    }

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
    fn list_log_filenames_returns_only_daily_logs_newest_first() {
        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        let root = dir.path();

        fs::write(root.join("vtt-2026-01-15.log"), b"old").unwrap();
        fs::write(root.join("vtt-2026-04-20.log"), b"newest").unwrap();
        fs::write(root.join("vtt-2026-04-19.log"), b"middle").unwrap();
        // Neither of these is a daily log and neither may appear in the menu.
        fs::write(root.join("vtt.log"), b"legacy").unwrap();
        fs::write(root.join("settings.conf"), b"unrelated").unwrap();

        // list_log_filenames reads the process-wide log dir, so point it here.
        init(root);

        let files = list_log_filenames().unwrap();
        assert_eq!(
            files,
            vec![
                "vtt-2026-04-20.log".to_string(),
                "vtt-2026-04-19.log".to_string(),
                "vtt-2026-01-15.log".to_string(),
            ],
            "daily logs only, newest first — the order both tray menus render"
        );
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
