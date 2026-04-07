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

    if let Ok(mut f) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = f.write_all(line.as_bytes());
    }

    print!("{}", line);
}

pub fn set_enabled(enabled: bool) {
    LOGGER.lock().unwrap().enabled = enabled;
}

pub fn is_enabled() -> bool {
    LOGGER.lock().unwrap().enabled
}

pub fn get_path() -> PathBuf {
    let mut logger = LOGGER.lock().unwrap();
    logger.ensure_date();
    logger.log_path()
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
            if !name.starts_with("vtt-") || !name.ends_with(".log") {
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

    // Remove legacy vtt.log files
    fs::remove_file(log_dir.join("vtt.log")).ok();
    for i in 1..=3 {
        fs::remove_file(log_dir.join(format!("vtt.log.{}", i))).ok();
    }
}

/// Convenience macro for formatted logging
#[macro_export]
macro_rules! vtt_log {
    ($($arg:tt)*) => {
        $crate::logging::log(&format!($($arg)*))
    };
}
