//! Local wall-clock time, formatted the few ways this app writes it: the daily
//! log file's date, a log line's timestamp, and an archived recording's id and
//! `recorded_at`.
//!
//! The calendar arithmetic and the formatting are pure and tested on every
//! platform. The OS is asked one thing, the local UTC offset at an instant, so
//! daylight saving and a time zone change while the app runs stay the OS's to
//! get right.

use std::time::{SystemTime, UNIX_EPOCH};

pub const SECS_PER_DAY: i64 = 86_400;
const MS_PER_DAY: i64 = SECS_PER_DAY * 1_000;

/// The proleptic Gregorian `(year, month, day)` for a count of days since
/// 1970-01-01.
///
/// Howard Hinnant's `civil_from_days`
/// (<https://howardhinnant.github.io/date_algorithms.html>): count from
/// 0000-03-01 instead, so each 400-year era repeats exactly and the leap day
/// falls last in its year.
pub fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097); // day of era, [0, 146096]
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of March-based year, [0, 365]
    let mp = (5 * doy + 2) / 153; // March-based month, [0, 11]
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

/// Days since 1970-01-01 for a proleptic Gregorian date: the inverse of
/// [`civil_from_days`], from the same source.
pub fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let (month, day) = (i64::from(month), i64::from(day));
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400); // [0, 399]
    let mp = if month > 2 { month - 3 } else { month + 9 }; // [0, 11]
    let doy = (153 * mp + 2) / 5 + day - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146_097 + doe - 719_468
}

/// An RFC 3339 UTC offset: `+01:00`, `-05:30`, `+00:00`. Leftover seconds,
/// which no zone in use today has, are dropped.
pub fn format_offset(offset_secs: i32) -> String {
    let sign = if offset_secs < 0 { '-' } else { '+' };
    let minutes = offset_secs.unsigned_abs() / 60;
    format!("{sign}{:02}:{:02}", minutes / 60, minutes % 60)
}

fn format_date(year: i64, month: u32, day: u32) -> String {
    format!("{year:04}-{month:02}-{day:02}")
}

/// A local wall-clock reading to the millisecond, with the UTC offset it was
/// read at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalTime {
    year: i64,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    millis: u32,
    offset_secs: i32,
}

impl LocalTime {
    /// The wall clock `offset_secs` east of UTC at `unix_ms` milliseconds
    /// since the epoch.
    pub fn from_unix_millis(unix_ms: i64, offset_secs: i32) -> Self {
        let local_ms = unix_ms + i64::from(offset_secs) * 1_000;
        let (year, month, day) = civil_from_days(local_ms.div_euclid(MS_PER_DAY));
        let ms_of_day = local_ms.rem_euclid(MS_PER_DAY);
        let secs_of_day = (ms_of_day / 1_000) as u32;
        LocalTime {
            year,
            month,
            day,
            hour: secs_of_day / 3_600,
            minute: secs_of_day / 60 % 60,
            second: secs_of_day % 60,
            millis: (ms_of_day % 1_000) as u32,
            offset_secs,
        }
    }

    /// `2026-11-18`: the daily log file's date and the archive's dated
    /// directory.
    pub fn date(&self) -> String {
        format_date(self.year, self.month, self.day)
    }

    /// The calendar day before [`date`](Self::date), in the same form.
    pub fn yesterday(&self) -> String {
        let (y, m, d) = civil_from_days(days_from_civil(self.year, self.month, self.day) - 1);
        format_date(y, m, d)
    }

    /// `2026-11-18 05:30:31`: a log line's timestamp.
    pub fn log_timestamp(&self) -> String {
        format!(
            "{} {:02}:{:02}:{:02}",
            self.date(),
            self.hour,
            self.minute,
            self.second
        )
    }

    /// `20261118T053031_639`: an archived recording's id, the shape
    /// `archive::is_our_recording` accepts.
    pub fn archive_id(&self) -> String {
        format!(
            "{:04}{:02}{:02}T{:02}{:02}{:02}_{:03}",
            self.year, self.month, self.day, self.hour, self.minute, self.second, self.millis
        )
    }

    /// `2026-11-18T05:30:31.639+00:00`: RFC 3339 with milliseconds and the
    /// numeric local offset, for an archive sidecar's `recorded_at`.
    pub fn rfc3339(&self) -> String {
        format!(
            "{}T{:02}:{:02}:{:02}.{:03}{}",
            self.date(),
            self.hour,
            self.minute,
            self.second,
            self.millis,
            format_offset(self.offset_secs)
        )
    }
}

/// Milliseconds since the Unix epoch, negative if the clock reads before 1970.
fn unix_millis_now() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_millis() as i64,
        Err(e) => -(e.duration().as_millis() as i64),
    }
}

/// Seconds since the Unix epoch. No time zone is involved.
pub fn unix_seconds() -> i64 {
    unix_millis_now().div_euclid(1_000)
}

/// The local wall clock now.
pub fn now() -> LocalTime {
    let unix_ms = unix_millis_now();
    LocalTime::from_unix_millis(unix_ms, local_offset_secs(unix_ms.div_euclid(1_000)))
}

/// The UTC offset a broken-down local time implies: its wall-clock reading
/// taken as if it were UTC, less the instant it was made from.
#[cfg(any(windows, test))]
fn offset_from_tm(unix_secs: i64, tm: &libc::tm) -> i64 {
    let days = days_from_civil(
        i64::from(tm.tm_year) + 1_900,
        (tm.tm_mon + 1) as u32,
        tm.tm_mday as u32,
    );
    let wall = days * SECS_PER_DAY
        + i64::from(tm.tm_hour) * 3_600
        + i64::from(tm.tm_min) * 60
        + i64::from(tm.tm_sec);
    wall - unix_secs
}

// POSIX <time.h>: `void tzset(void);`. The libc crate binds it on Windows
// only.
#[cfg(unix)]
extern "C" {
    fn tzset();
}

/// Seconds east of UTC at `unix_secs`, from the OS; 0 if it cannot say.
///
/// `tzset` runs first, as POSIX advises before `localtime_r`: glibc's
/// `localtime_r` loads the zone once per process, while `tzset` re-checks `TZ`
/// and `/etc/localtime`, so a time zone change reaches the running app.
///
/// Both read `TZ` with `getenv`, which races a `setenv` on another thread.
/// Nothing in this crate sets environment variables.
#[cfg(unix)]
fn local_offset_secs(unix_secs: i64) -> i32 {
    let t = unix_secs as libc::time_t;
    // SAFETY: libc::tm is integers and a nullable pointer; all zeroes is valid.
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    // SAFETY: tzset takes no arguments. localtime_r writes only into `tm`, and
    // both pointers are to locals that outlive the call.
    let r = unsafe {
        tzset();
        libc::localtime_r(&t, &mut tm)
    };
    if r.is_null() {
        0
    } else {
        tm.tm_gmtoff as i32
    }
}

/// Seconds east of UTC at `unix_secs`, from the C runtime; 0 if it cannot say.
///
/// The CRT's `tm` has no `tm_gmtoff`, so the offset is the gap between the
/// local reading and the instant. `_tzset` runs first so a time zone change
/// in Windows Settings reaches the running app.
#[cfg(windows)]
fn local_offset_secs(unix_secs: i64) -> i32 {
    let t = unix_secs as libc::time_t;
    // SAFETY: libc::tm is integers only on Windows; all zeroes is valid.
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    // SAFETY: _tzset takes no arguments. _localtime64_s writes only into `tm`,
    // and both pointers are to locals that outlive the call.
    let err = unsafe {
        libc::tzset();
        libc::localtime_s(&mut tm, &t)
    };
    if err == 0 {
        offset_from_tm(unix_secs, &tm) as i32
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2026-11-18T05:30:31.639Z (`date -u -d @1794979831`).
    const SAMPLE_MS: i64 = 1_794_979_831_639;

    fn at(unix_ms: i64, offset_secs: i32) -> LocalTime {
        LocalTime::from_unix_millis(unix_ms, offset_secs)
    }

    /// Noon UTC on a civil date, so no offset under test moves the date.
    fn noon(year: i64, month: u32, day: u32) -> LocalTime {
        at(
            days_from_civil(year, month, day) * MS_PER_DAY + MS_PER_DAY / 2,
            0,
        )
    }

    #[test]
    fn civil_from_days_gives_known_dates() {
        // Day numbers from `date -u -d <date> +%s` divided by 86400.
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(11_016), (2000, 2, 29));
        assert_eq!(civil_from_days(19_782), (2024, 2, 29));
        assert_eq!(civil_from_days(20_775), (2026, 11, 18));
        assert_eq!(civil_from_days(10_956), (1999, 12, 31));
        assert_eq!(civil_from_days(-1), (1969, 12, 31));
    }

    #[test]
    fn days_from_civil_inverts_civil_from_days() {
        // Every day from 1900-01-01 to 2199-12-31 survives the round trip.
        for days in -25_567..84_006 {
            let (y, m, d) = civil_from_days(days);
            assert_eq!(days_from_civil(y, m, d), days, "{y}-{m}-{d}");
        }
    }

    #[test]
    fn every_field_is_zero_padded() {
        // 2026-01-02T03:04:05.006Z
        let ms = days_from_civil(2026, 1, 2) * MS_PER_DAY + 3_600_000 * 3 + 60_000 * 4 + 5_006;
        let t = at(ms, 0);
        assert_eq!(t.date(), "2026-01-02");
        assert_eq!(t.log_timestamp(), "2026-01-02 03:04:05");
        assert_eq!(t.archive_id(), "20260102T030405_006");
        assert_eq!(t.rfc3339(), "2026-01-02T03:04:05.006+00:00");
    }

    #[test]
    fn rfc3339_keeps_milliseconds_and_a_numeric_utc_offset() {
        assert_eq!(at(SAMPLE_MS, 0).rfc3339(), "2026-11-18T05:30:31.639+00:00");
    }

    #[test]
    fn format_offset_signs_and_pads_whole_and_part_hours() {
        assert_eq!(format_offset(0), "+00:00");
        assert_eq!(format_offset(3_600), "+01:00");
        assert_eq!(format_offset(-18_000), "-05:00");
        assert_eq!(format_offset(19_800), "+05:30");
        assert_eq!(format_offset(-19_800), "-05:30");
        assert_eq!(format_offset(-12_600), "-03:30");
        assert_eq!(format_offset(20_700), "+05:45");
        assert_eq!(format_offset(50_400), "+14:00");
        assert_eq!(format_offset(-43_200), "-12:00");
    }

    #[test]
    fn the_offset_moves_the_wall_clock() {
        // Oracles: `TZ=Asia/Kolkata date -d @1794979831 +%FT%T%:z` and likewise
        // for America/St_Johns and Asia/Kathmandu.
        assert_eq!(
            at(SAMPLE_MS, 19_800).rfc3339(),
            "2026-11-18T11:00:31.639+05:30"
        );
        assert_eq!(
            at(SAMPLE_MS, -12_600).rfc3339(),
            "2026-11-18T02:00:31.639-03:30"
        );
        assert_eq!(
            at(SAMPLE_MS, 20_700).rfc3339(),
            "2026-11-18T11:15:31.639+05:45"
        );
    }

    #[test]
    fn the_offset_can_move_the_date_across_a_year() {
        // 2026-01-01T03:00:00Z is still New Year's Eve at -05:00.
        let t = at(1_767_236_400_000, -18_000);
        assert_eq!(t.date(), "2025-12-31");
        assert_eq!(t.log_timestamp(), "2025-12-31 22:00:00");
        // 2026-12-31T23:30:00Z is already New Year's Day at +01:00.
        let t = at(1_798_759_800_000, 3_600);
        assert_eq!(t.date(), "2027-01-01");
        assert_eq!(t.archive_id(), "20270101T003000_000");
    }

    #[test]
    fn yesterday_crosses_month_and_year_boundaries() {
        assert_eq!(noon(2026, 11, 18).yesterday(), "2026-11-17");
        assert_eq!(noon(2026, 5, 1).yesterday(), "2026-04-30");
        assert_eq!(noon(2026, 3, 1).yesterday(), "2026-02-28");
        assert_eq!(noon(2024, 3, 1).yesterday(), "2024-02-29");
        assert_eq!(noon(2000, 3, 1).yesterday(), "2000-02-29");
        assert_eq!(noon(1900, 3, 1).yesterday(), "1900-02-28");
        assert_eq!(noon(2027, 1, 1).yesterday(), "2026-12-31");
    }

    #[test]
    fn a_pre_epoch_instant_floors_rather_than_truncates() {
        assert_eq!(at(-1, 0).rfc3339(), "1969-12-31T23:59:59.999+00:00");
    }

    #[test]
    fn offset_from_tm_reads_the_wall_clock_back() {
        // SAFETY: libc::tm is integers plus, on Unix, a nullable pointer; all
        // zeroes is a valid value.
        let mut tm: libc::tm = unsafe { std::mem::zeroed() };
        tm.tm_year = 2026 - 1900;
        tm.tm_mon = 10; // November
        tm.tm_mday = 18;
        tm.tm_hour = 11;
        tm.tm_min = 0;
        tm.tm_sec = 31;
        assert_eq!(offset_from_tm(1_794_979_831, &tm), 19_800);
        tm.tm_hour = 2;
        assert_eq!(offset_from_tm(1_794_979_831, &tm), -12_600);
        // Behind UTC across midnight: 2026-01-01T03:00:00Z read as the 31st.
        tm.tm_year = 2025 - 1900;
        tm.tm_mon = 11;
        tm.tm_mday = 31;
        tm.tm_hour = 22;
        tm.tm_min = 0;
        tm.tm_sec = 0;
        assert_eq!(offset_from_tm(1_767_236_400, &tm), -18_000);
    }

    /// The Windows path derives the offset from the broken-down time because
    /// its `tm` has no `tm_gmtoff`. Here the OS gives both, so check the
    /// derivation against the OS's own answer, today and half a year ago (the
    /// other side of any daylight-saving change).
    #[cfg(unix)]
    #[test]
    fn offset_from_tm_agrees_with_the_os_gmtoff() {
        let now = unix_seconds();
        for probe in [now, now - 182 * SECS_PER_DAY, 0, 1_794_979_831] {
            let t = probe as libc::time_t;
            // SAFETY: as above, all zeroes is a valid libc::tm.
            let mut tm: libc::tm = unsafe { std::mem::zeroed() };
            // SAFETY: localtime_r writes only into `tm`; both pointers are to
            // locals that outlive the call.
            let r = unsafe { libc::localtime_r(&t, &mut tm) };
            assert!(!r.is_null(), "localtime_r failed for {probe}");
            assert_eq!(offset_from_tm(probe, &tm), tm.tm_gmtoff as i64, "{probe}");
        }
    }

    #[test]
    fn now_reads_the_same_instant_as_unix_seconds() {
        let before = unix_seconds();
        let t = now();
        let after = unix_seconds();
        let wall = days_from_civil(t.year, t.month, t.day) * SECS_PER_DAY
            + i64::from(t.hour) * 3_600
            + i64::from(t.minute) * 60
            + i64::from(t.second);
        let utc = wall - i64::from(t.offset_secs);
        assert!(
            before <= utc && utc <= after,
            "{before} <= {utc} <= {after}"
        );
        assert!(t.offset_secs.abs() <= 14 * 3_600, "{}", t.offset_secs);
        assert!(t.millis < 1_000);
    }
}
