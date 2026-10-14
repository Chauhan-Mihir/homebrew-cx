//! Stamps the version into the binary.
//!
//! A trimmed copy of `crates/build-info/build.rs` in the toolkit: the
//! pipeline sets `CX_VERSION` from the release tag and that value is used
//! verbatim, otherwise the package version is used. Keeping the same shape
//! means the rehearsal covers the real version-stamping path, not a
//! simplified one.

use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    println!("cargo:rerun-if-env-changed=CX_VERSION");

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (date, time) = utc(secs);

    let version = std::env::var("CX_VERSION")
        .ok()
        .map(|v| v.trim().trim_start_matches('v').to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| std::env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION"));

    assert!(
        version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+')),
        "CX_VERSION has characters outside [0-9A-Za-z.+-]: {version}"
    );

    println!("cargo:rustc-env=CX_BUILD_VERSION={version}");
    println!("cargo:rustc-env=CX_BUILD_TIME={date}T{time}Z");
    println!(
        "cargo:rustc-env=CX_BUILD_TARGET={}",
        std::env::var("TARGET").unwrap_or_else(|_| "unknown".into())
    );
}

/// `(yyyy-mm-dd, hh:mm:ss)` in UTC for Unix seconds (civil-from-days,
/// proleptic Gregorian calendar). Copied from the toolkit so the crate
/// stays dependency-free.
fn utc(secs: u64) -> (String, String) {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    (
        format!("{year:04}-{month:02}-{day:02}"),
        format!("{:02}:{:02}:{:02}", rem / 3_600, rem % 3_600 / 60, rem % 60),
    )
}
