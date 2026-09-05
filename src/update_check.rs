//! GitHub Releases update check (TASK-VTT095). Informational only — this
//! never downloads or installs anything; the tray item it feeds just links
//! to the release page. Runs once at startup on a background thread so a
//! slow or offline network never delays the app becoming ready, and any
//! failure (network, rate limit, malformed response) is silently swallowed
//! — an update check must never be the reason the app looks broken.

use serde_json::Value;

const RELEASES_API: &str =
    "https://api.github.com/repos/powell-clark/voice-to-text/releases/latest";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct UpdateInfo {
    pub version: String,
    pub url: String,
}

/// Parsed as major.minor.patch integers. A tag that doesn't parse this way
/// returns `None` — "cannot compare" — rather than guessing; a malformed
/// release tag must never trigger a false "update available".
fn parse_version(s: &str) -> Option<(u64, u64, u64)> {
    let s = s.strip_prefix('v').unwrap_or(s);
    let mut parts = s.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    Some((major, minor, patch))
}

/// True when `latest` is a genuinely newer version than `current`. Pure —
/// no network — so the comparison itself is unit-testable without a live
/// GitHub API call.
fn is_newer(current: &str, latest: &str) -> bool {
    match (parse_version(current), parse_version(latest)) {
        (Some(c), Some(l)) => l > c,
        _ => false,
    }
}

/// Query GitHub Releases for the latest tag. Returns `None` on any failure
/// (network, parse, rate limit) or when already up to date.
pub fn check_for_update() -> Option<UpdateInfo> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(concat!("voice-to-text/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .ok()?;
    let resp = client
        .get(RELEASES_API)
        .send()
        .ok()?
        .error_for_status()
        .ok()?;
    let body: Value = serde_json::from_str(&resp.text().ok()?).ok()?;
    let tag = body.get("tag_name")?.as_str()?;
    let url = body.get("html_url")?.as_str()?;
    if is_newer(CURRENT_VERSION, tag) {
        Some(UpdateInfo {
            version: tag.to_string(),
            url: url.to_string(),
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_version_reads_major_minor_patch() {
        assert_eq!(parse_version("2.4.0"), Some((2, 4, 0)));
    }

    #[test]
    fn parse_version_strips_a_leading_v() {
        assert_eq!(parse_version("v2.4.0"), Some((2, 4, 0)));
    }

    #[test]
    fn parse_version_rejects_malformed_tags_rather_than_guessing() {
        assert_eq!(parse_version("not-a-version"), None);
        assert_eq!(parse_version("2.4"), None);
        assert_eq!(parse_version(""), None);
        assert_eq!(parse_version("v2.x.0"), None);
    }

    #[test]
    fn is_newer_true_for_a_genuinely_newer_patch() {
        assert!(is_newer("2.4.0", "v2.4.1"));
    }

    #[test]
    fn is_newer_true_for_a_newer_minor_or_major() {
        assert!(is_newer("2.4.0", "v2.5.0"));
        assert!(is_newer("2.4.0", "v3.0.0"));
    }

    #[test]
    fn is_newer_false_for_the_same_version() {
        assert!(!is_newer("2.4.0", "v2.4.0"));
    }

    #[test]
    fn is_newer_false_for_an_older_version() {
        assert!(!is_newer("2.4.0", "v2.3.9"));
    }

    #[test]
    fn is_newer_false_when_either_side_fails_to_parse() {
        // Never guess: an incomparable tag must not trigger an update prompt.
        assert!(!is_newer("2.4.0", "not-a-version"));
        assert!(!is_newer("not-a-version", "v2.4.0"));
    }
}
