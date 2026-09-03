//! `--doctor`: does the running app match the one on disk? (TASK-VTT153)
//!
//! Three deployment failures happened in one morning, all the same shape:
//! something reported success while the old code kept running.
//!
//! 1. The `.deb` shipped a binary two weeks older than the source, through four
//!    green build stages and a sudo prompt. Now gated at build time
//!    (TASK-VTT152), but only for this repo's own script.
//! 2. `apt install` refuses an unchanged version number and exits 0, so the
//!    install script printed "Installed." over a no-op.
//! 3. After a correct `dpkg -i`, the running process still held the replaced
//!    inode — `/proc/<pid>/exe` read `/usr/bin/vtt-linux (deleted)`. Because
//!    that process had been launched by hand rather than by systemd,
//!    `MainPID` was 0, so `systemctl restart` restarted an empty unit while the
//!    singleton lock kept the replacement out.
//!
//! What distinguished the builds throughout was a log line about capture rate.
//! That was luck. This module makes the question answerable on purpose.

use std::path::{Path, PathBuf};

/// What `/proc/<pid>/exe` says about the file a process is running.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExeState {
    /// The process is running the file still at this path.
    Current(PathBuf),
    /// The file was replaced or removed underneath the process. The process
    /// keeps executing the old inode until it restarts — this is the state
    /// that makes a successful install look like a failed one.
    Replaced(PathBuf),
}

/// Parse the target of a `/proc/<pid>/exe` symlink.
///
/// Linux appends a literal " (deleted)" when the inode the link points at has
/// been unlinked, which is exactly what a package upgrade does to a running
/// binary. Pure so the parsing is testable without a live process.
pub fn parse_exe_link(target: &str) -> ExeState {
    match target.strip_suffix(" (deleted)") {
        Some(path) => ExeState::Replaced(PathBuf::from(path)),
        None => ExeState::Current(PathBuf::from(target)),
    }
}

/// One line of the report: a check, its finding, and whether it is a problem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub label: String,
    pub value: String,
    pub is_problem: bool,
}

impl Finding {
    pub fn ok(label: &str, value: impl Into<String>) -> Self {
        Self {
            label: label.to_string(),
            value: value.into(),
            is_problem: false,
        }
    }

    pub fn problem(label: &str, value: impl Into<String>) -> Self {
        Self {
            label: label.to_string(),
            value: value.into(),
            is_problem: true,
        }
    }
}

/// Turn one process's exe state into a finding, given the path the package
/// manager installs to. Pure: the caller does the `/proc` reading.
pub fn diagnose_running(pid: u32, state: &ExeState, installed: &Path) -> Finding {
    match state {
        ExeState::Replaced(path) => Finding::problem(
            "running binary",
            format!(
                "pid {pid} is running a DELETED copy of {} — the file was replaced \
                 underneath it, so this process is still the OLD build. Restart it: \
                 `kill {pid}` then `systemctl --user restart vtt.service`",
                path.display()
            ),
        ),
        ExeState::Current(path) if path == installed => {
            Finding::ok("running binary", format!("pid {pid} at {}", path.display()))
        }
        ExeState::Current(path) => Finding::problem(
            "running binary",
            format!(
                "pid {pid} is running {}, not the installed {} — two builds are \
                 present and the wrong one may be answering the hotkey",
                path.display(),
                installed.display()
            ),
        ),
    }
}

/// Render findings as the report the operator reads. Returns the text and
/// whether anything was wrong, so the caller can set an exit code.
pub fn format_report(findings: &[Finding]) -> (String, bool) {
    let width = findings.iter().map(|f| f.label.len()).max().unwrap_or(0);
    let mut out = String::from("voice-to-text doctor\n\n");
    let mut any_problem = false;
    for f in findings {
        let mark = if f.is_problem {
            any_problem = true;
            "PROBLEM"
        } else {
            "ok     "
        };
        out.push_str(&format!(
            "  [{mark}] {:width$}  {}\n",
            f.label,
            f.value,
            width = width
        ));
    }
    out.push('\n');
    out.push_str(if any_problem {
        "One or more checks failed. The most common cause is a running process \
         that predates an install.\n"
    } else {
        "Everything checks out.\n"
    });
    (out, any_problem)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_replaced_binary_is_detected_from_the_deleted_suffix() {
        // The exact string Linux produced during the TASK-VTT150 deployment.
        assert_eq!(
            parse_exe_link("/usr/bin/vtt-linux (deleted)"),
            ExeState::Replaced(PathBuf::from("/usr/bin/vtt-linux"))
        );
    }

    #[test]
    fn an_ordinary_link_is_current() {
        assert_eq!(
            parse_exe_link("/usr/bin/vtt-linux"),
            ExeState::Current(PathBuf::from("/usr/bin/vtt-linux"))
        );
    }

    #[test]
    fn a_path_containing_deleted_is_not_mistaken_for_a_replaced_one() {
        // Only the trailing marker counts; a directory legitimately named
        // "deleted" must not trip the check.
        assert_eq!(
            parse_exe_link("/home/u/deleted/vtt-linux"),
            ExeState::Current(PathBuf::from("/home/u/deleted/vtt-linux"))
        );
    }

    #[test]
    fn the_replaced_case_names_the_pid_and_the_fix() {
        let f = diagnose_running(
            3247235,
            &ExeState::Replaced(PathBuf::from("/usr/bin/vtt-linux")),
            Path::new("/usr/bin/vtt-linux"),
        );
        assert!(f.is_problem);
        assert!(f.value.contains("3247235"), "must name the pid to kill");
        assert!(f.value.contains("kill 3247235"), "must give the command");
        assert!(f.value.contains("OLD build"), "must say what is wrong");
    }

    #[test]
    fn running_the_installed_path_is_fine() {
        let f = diagnose_running(
            42,
            &ExeState::Current(PathBuf::from("/usr/bin/vtt-linux")),
            Path::new("/usr/bin/vtt-linux"),
        );
        assert!(!f.is_problem);
    }

    #[test]
    fn running_a_different_build_is_flagged() {
        // The case where a hand-launched ./target/release build is answering
        // the hotkey while the packaged one is what the operator just updated.
        let f = diagnose_running(
            99,
            &ExeState::Current(PathBuf::from("/home/u/repo/target/release/vtt-linux")),
            Path::new("/usr/bin/vtt-linux"),
        );
        assert!(f.is_problem);
        assert!(f.value.contains("two builds are present"));
    }

    #[test]
    fn the_report_sets_a_failure_flag_when_anything_is_wrong() {
        let (text, bad) = format_report(&[
            Finding::ok("version", "2.3.11"),
            Finding::problem("running binary", "stale"),
        ]);
        assert!(
            bad,
            "a problem must be reported to the caller for its exit code"
        );
        assert!(text.contains("PROBLEM"));
        assert!(text.contains("One or more checks failed"));
    }

    #[test]
    fn a_clean_report_says_so_and_flags_nothing() {
        let (text, bad) = format_report(&[Finding::ok("version", "2.3.11")]);
        assert!(!bad);
        assert!(text.contains("Everything checks out"));
        assert!(!text.contains("PROBLEM"));
    }

    #[test]
    fn an_empty_report_does_not_panic_on_the_label_width() {
        let (_, bad) = format_report(&[]);
        assert!(!bad);
    }
}
