//! Build script.
//!
//! * Embeds the Jinja templates (`minijinja_embed`); `typed-fetch.ts` is a
//!   static asset loaded with `include_str!` instead.
//! * Embeds build metadata (timestamp + optional git revision) as the
//!   `BUILD_INFO` environment variable, replacing the old `stamp.txt`.

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    println!("cargo:rerun-if-changed=src/tools/generate_paths/templates");
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");

    minijinja_embed::embed_templates!("src/tools/generate_paths/templates", &[".jinja"]);

    println!("cargo:rustc-env=BUILD_INFO={}", build_info());
}

/// `YYYY-MM-DD HH:MM:SS` in UTC, honouring `SOURCE_DATE_EPOCH` for
/// reproducible builds, optionally followed by the short git revision.
fn build_info() -> String {
    let mut timestamp = format_timestamp(timestamp_secs());

    if let Some(rev) = git_revision() {
        timestamp.push_str(&format!(" (git {rev})"));
    }

    timestamp
}

fn timestamp_secs() -> i64 {
    if let Ok(value) = std::env::var("SOURCE_DATE_EPOCH")
        && let Ok(secs) = value.parse::<i64>()
    {
        return secs;
    }

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}

/// Formats a Unix timestamp as `YYYY-MM-DD HH:MM:SS` (UTC) without pulling in a
/// date/time crate.
fn format_timestamp(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let seconds_of_day = secs.rem_euclid(86_400);
    let (hour, minute, second) = (
        seconds_of_day / 3_600,
        (seconds_of_day % 3_600) / 60,
        seconds_of_day % 60,
    );
    let (year, month, day) = civil_from_days(days);

    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}")
}

/// Howard Hinnant's `civil_from_days` algorithm (days since 1970-01-01 → date).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

fn git_revision() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let rev = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if rev.is_empty() { None } else { Some(rev) }
}
