// SPDX-License-Identifier: AGPL-3.0-or-later

use std::time::{SystemTime, UNIX_EPOCH};

const FLUXER_EPOCH: u64 = 1_420_070_400_000;

pub fn snowflake_to_timestamp_ms(snowflake: &str) -> Option<u64> {
    let id: u64 = snowflake.parse().ok()?;
    let timestamp_ms = (id >> 22) + FLUXER_EPOCH;
    Some(timestamp_ms)
}

pub fn format_unix_timestamp(unix_seconds: u64) -> String {
    let offset = time::OffsetDateTime::from_unix_timestamp(unix_seconds as i64).ok();
    match offset {
        Some(dt) => {
            let format = time::format_description::well_known::Rfc3339;
            dt.format(&format)
                .unwrap_or_else(|_| unix_seconds.to_string())
        }
        None => unix_seconds.to_string(),
    }
}

pub fn format_unix_timestamp_ms(unix_ms: u64) -> String {
    format_unix_timestamp(unix_ms / 1000)
}

pub fn relative_time(unix_seconds: u64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before Unix epoch")
        .as_secs();
    let diff = now.saturating_sub(unix_seconds);
    if diff < 60 {
        return "just now".to_owned();
    }
    if diff < 3600 {
        let minutes = diff / 60;
        return format!(
            "{} minute{} ago",
            minutes,
            if minutes == 1 { "" } else { "s" }
        );
    }
    if diff < 86400 {
        let hours = diff / 3600;
        return format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" });
    }
    let days = diff / 86400;
    if days < 30 {
        return format!("{} day{} ago", days, if days == 1 { "" } else { "s" });
    }
    let months = days / 30;
    if months < 12 {
        return format!("{} month{} ago", months, if months == 1 { "" } else { "s" });
    }
    let years = months / 12;
    format!("{} year{} ago", years, if years == 1 { "" } else { "s" })
}

pub fn format_admin_timestamp(iso: &str) -> String {
    let dt = time::OffsetDateTime::parse(iso, &time::format_description::well_known::Rfc3339)
        .or_else(|_| {
            time::OffsetDateTime::parse(
                iso,
                &time::format_description::well_known::Iso8601::DEFAULT,
            )
        })
        .ok();
    match dt {
        Some(dt) => {
            let dt = dt.to_offset(time::UtcOffset::UTC);
            let month = match dt.month() {
                time::Month::January => "Jan",
                time::Month::February => "Feb",
                time::Month::March => "Mar",
                time::Month::April => "Apr",
                time::Month::May => "May",
                time::Month::June => "Jun",
                time::Month::July => "Jul",
                time::Month::August => "Aug",
                time::Month::September => "Sep",
                time::Month::October => "Oct",
                time::Month::November => "Nov",
                time::Month::December => "Dec",
            };
            let day = dt.day();
            let year = dt.year();
            let hour_12 = match dt.hour() {
                0 => 12,
                h if h > 12 => h - 12,
                h => h,
            };
            let minute = dt.minute();
            let ampm = if dt.hour() < 12 { "AM" } else { "PM" };
            format!("{month} {day}, {year}, {hour_12}:{minute:02} {ampm} UTC")
        }
        None => iso.to_owned(),
    }
}

pub fn snowflake_creation_date(snowflake: &str) -> String {
    match snowflake_to_timestamp_ms(snowflake) {
        Some(ms) => format_unix_timestamp_ms(ms),
        None => "Unknown".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snowflake_to_timestamp_ms_known_value() {
        assert_eq!(snowflake_to_timestamp_ms("0"), Some(FLUXER_EPOCH));
    }

    #[test]
    fn snowflake_to_timestamp_ms_real_id() {
        let snowflake = (1u64 << 22).to_string();
        assert_eq!(
            snowflake_to_timestamp_ms(&snowflake),
            Some(FLUXER_EPOCH + 1)
        );
    }

    #[test]
    fn snowflake_to_timestamp_ms_invalid() {
        assert_eq!(snowflake_to_timestamp_ms(""), None);
        assert_eq!(snowflake_to_timestamp_ms("abc"), None);
        assert_eq!(snowflake_to_timestamp_ms("-1"), None);
    }

    #[test]
    fn format_unix_timestamp_iso8601() {
        let result = format_unix_timestamp(1_704_067_200);
        assert!(
            result.starts_with("2024-01-01"),
            "expected ISO date starting with 2024-01-01, got: {result}"
        );
    }

    #[test]
    fn format_unix_timestamp_ms_converts() {
        let result = format_unix_timestamp_ms(1_704_067_200_000);
        assert!(
            result.starts_with("2024-01-01"),
            "expected ISO date, got: {result}"
        );
    }

    #[test]
    fn relative_time_just_now() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert_eq!(relative_time(now), "just now");
    }

    #[test]
    fn relative_time_minutes() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert_eq!(relative_time(now - 120), "2 minutes ago");
        assert_eq!(relative_time(now - 60), "1 minute ago");
    }

    #[test]
    fn relative_time_hours() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert_eq!(relative_time(now - 3600), "1 hour ago");
        assert_eq!(relative_time(now - 7200), "2 hours ago");
    }

    #[test]
    fn relative_time_days() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert_eq!(relative_time(now - 86400), "1 day ago");
        assert_eq!(relative_time(now - 86400 * 5), "5 days ago");
    }

    #[test]
    fn snowflake_creation_date_invalid() {
        assert_eq!(snowflake_creation_date("abc"), "Unknown");
    }

    #[test]
    fn snowflake_creation_date_valid() {
        let result = snowflake_creation_date("0");
        assert!(
            result.starts_with("2015-01-01"),
            "expected 2015-01-01, got: {result}"
        );
    }
}
