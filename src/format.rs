//! Shared human-readable formatting helpers for CLI tools.

/// Format a byte count using the appropriate binary unit (B/KB/MB/GB/TB).
pub fn format_bytes(bytes: f64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.2} {}", UNITS[unit])
}

/// Format an age (in seconds) as a human-readable "time ago" string, using
/// the largest applicable unit (seconds, minutes, hours, or days). Negative
/// ages (e.g. clock skew) are clamped to zero.
pub fn format_age(age_secs: i64) -> String {
    let age_secs = age_secs.max(0);
    if age_secs < 60 {
        format!("{age_secs}s ago")
    } else if age_secs < 3600 {
        format!("{}m ago", age_secs / 60)
    } else if age_secs < 86400 {
        format!("{}h ago", age_secs / 3600)
    } else {
        format!("{}d ago", age_secs / 86400)
    }
}
