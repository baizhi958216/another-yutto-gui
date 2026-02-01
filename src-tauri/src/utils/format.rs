/// Format bytes to human-readable string (e.g., "75.70 MiB")
/// Uses binary units (KiB, MiB, GiB, TiB) with 1024 base
pub fn format_bytes(bytes: i64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let bytes_f = bytes.abs() as f64;
    let base: f64 = 1024.0;
    let exp = (bytes_f.ln() / base.ln()).floor() as usize;
    let exp = exp.min(UNITS.len() - 1);

    let value = bytes_f / base.powi(exp as i32);
    let sign = if bytes < 0 { "-" } else { "" };

    format!("{}{:.2} {}", sign, value, UNITS[exp])
}

/// Format speed (bytes/sec to human-readable, e.g., "56.30 MiB/s")
pub fn format_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec.is_nan() || bytes_per_sec.is_infinite() || bytes_per_sec < 0.0 {
        return "0 B/s".to_string();
    }

    format!("{}/s", format_bytes(bytes_per_sec as i64))
}

/// Format ETA (seconds to human-readable, e.g., "02:35" or "1:23:45")
/// Returns "--:--" for invalid values
pub fn format_eta(seconds: f64) -> String {
    if seconds.is_nan() || seconds.is_infinite() || seconds < 0.0 {
        return "--:--".to_string();
    }

    let total_secs = seconds.round() as i64;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{:02}:{:02}", minutes, secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.00 KiB");
        assert_eq!(format_bytes(1048576), "1.00 MiB");
        assert_eq!(format_bytes(81604378), "77.82 MiB");
        assert_eq!(format_bytes(1073741824), "1.00 GiB");
    }

    #[test]
    fn test_format_speed() {
        assert_eq!(format_speed(0.0), "0 B/s");
        assert_eq!(format_speed(524288.0), "512.00 KiB/s");
        assert_eq!(format_speed(5242880.0), "5.00 MiB/s");
        assert_eq!(format_speed(f64::NAN), "0 B/s");
        assert_eq!(format_speed(f64::INFINITY), "0 B/s");
        assert_eq!(format_speed(-100.0), "0 B/s");
    }

    #[test]
    fn test_format_eta() {
        assert_eq!(format_eta(0.0), "00:00");
        assert_eq!(format_eta(65.0), "01:05");
        assert_eq!(format_eta(3665.0), "01:01:05");
        assert_eq!(format_eta(17.5), "00:18");
        assert_eq!(format_eta(125.5), "02:06");
        assert_eq!(format_eta(f64::NAN), "--:--");
        assert_eq!(format_eta(f64::INFINITY), "--:--");
        assert_eq!(format_eta(-10.0), "--:--");
    }
}
