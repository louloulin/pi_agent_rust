//! Pure wall-clock snapshot and formatting primitives for Pi.

use chrono::{DateTime, FixedOffset, Local, SecondsFormat, Utc};

/// Wall-clock reading rendered for the model or another client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeSnapshot {
    pub utc: String,
    pub local: String,
    pub offset_seconds: i32,
    pub offset: String,
    pub unix_seconds: i64,
    pub unix_millis: i64,
    pub weekday: String,
    pub iso_week: u32,
    pub timezone: Option<String>,
}

impl TimeSnapshot {
    #[must_use]
    pub fn from_instants(utc: DateTime<Utc>, local: DateTime<FixedOffset>, timezone: Option<String>) -> Self {
        let offset_seconds = local.offset().local_minus_utc();
        let sign = if offset_seconds < 0 { '-' } else { '+' };
        let abs = offset_seconds.unsigned_abs();
        Self {
            utc: utc.to_rfc3339_opts(SecondsFormat::Secs, true),
            local: local.to_rfc3339_opts(SecondsFormat::Secs, false),
            offset_seconds,
            offset: format!("{sign}{:02}:{:02}", abs / 3600, (abs % 3600) / 60),
            unix_seconds: utc.timestamp(),
            unix_millis: utc.timestamp_millis(),
            weekday: local.format("%A").to_string(),
            iso_week: local.format("%V").to_string().parse().unwrap_or(0),
            timezone,
        }
    }

    #[must_use]
    pub fn now() -> Self {
        let utc = Utc::now();
        let local: DateTime<FixedOffset> = Local::now().fixed_offset();
        let timezone = std::env::var("TZ").ok().map(|value| value.trim().to_string()).filter(|value| !value.is_empty());
        Self::from_instants(utc, local, timezone)
    }

    #[must_use]
    pub fn render_text(&self) -> String {
        let timezone = self.timezone.as_deref().map_or_else(String::new, |tz| format!(" [{tz}]"));
        format!("UTC: {}\nLocal: {} (UTC{}){timezone}\nUnix: {}\nWeekday: {}\nISO week: {}", self.utc, self.local, self.offset, self.unix_seconds, self.weekday, self.iso_week)
    }

    #[must_use]
    pub fn details(&self) -> serde_json::Value {
        serde_json::json!({"utc": self.utc, "local": self.local, "offset": self.offset, "offsetSeconds": self.offset_seconds, "unixSeconds": self.unix_seconds, "unixMillis": self.unix_millis, "weekday": self.weekday, "isoWeek": self.iso_week, "timezone": self.timezone})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn pinned_snapshot_has_stable_wire_fields() {
        let utc = Utc.with_ymd_and_hms(2026, 9, 1, 23, 15, 42).unwrap();
        let offset = FixedOffset::west_opt(4 * 3600).unwrap();
        let snapshot = TimeSnapshot::from_instants(utc, utc.with_timezone(&offset), Some("America/New_York".into()));
        assert_eq!(snapshot.utc, "2026-09-01T23:15:42Z");
        assert_eq!(snapshot.local, "2026-09-01T19:15:42-04:00");
        assert_eq!(snapshot.offset, "-04:00");
        assert_eq!(snapshot.iso_week, 36);
        assert_eq!(snapshot.details()["offsetSeconds"], -14_400);
    }
}
