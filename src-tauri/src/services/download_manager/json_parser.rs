use serde::{Deserialize, Serialize};

/// Yutto JSON progress output structure
/// Corresponds to the JSON format output by yutto with --json-output flag
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct YuttoProgressJson {
    #[serde(rename = "type")]
    pub msg_type: String,        // "progress"
    pub downloaded: i64,          // bytes downloaded
    pub total: i64,               // total bytes
    pub speed: f64,               // bytes per second
    pub progress: f64,            // percentage (0-100)
    pub eta: f64,                 // seconds remaining
    pub files: Option<i32>,       // number of files
}

/// Parsed progress data extracted from JSON or regex
#[derive(Debug, Clone)]
pub struct ParsedProgress {
    pub progress: f64,
    pub speed_bytes_per_sec: f64,
    pub downloaded_bytes: i64,
    pub total_bytes: i64,
    pub eta_seconds: Option<f64>,
    pub files_count: Option<i32>,
}

/// Parse a line of output, attempting JSON first
/// Returns Some(ParsedProgress) if JSON parsing succeeds, None otherwise
pub fn parse_progress_line(line: &str) -> Option<ParsedProgress> {
    // Try JSON parsing first
    if let Ok(json_progress) = serde_json::from_str::<YuttoProgressJson>(line) {
        // Only process "progress" type messages
        if json_progress.msg_type == "progress" {
            return Some(ParsedProgress {
                progress: json_progress.progress,
                speed_bytes_per_sec: json_progress.speed,
                downloaded_bytes: json_progress.downloaded,
                total_bytes: json_progress.total,
                eta_seconds: Some(json_progress.eta),
                files_count: json_progress.files,
            });
        }
    }

    // Return None if JSON parsing fails or message type is not "progress"
    // Caller will fallback to regex parsing
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_json() {
        let json_line = r#"{"type":"progress","downloaded":1048576,"total":10485760,"speed":524288.0,"progress":10.0,"eta":17.5,"files":2}"#;

        let result = parse_progress_line(json_line);
        assert!(result.is_some());

        let parsed = result.unwrap();
        assert_eq!(parsed.progress, 10.0);
        assert_eq!(parsed.speed_bytes_per_sec, 524288.0);
        assert_eq!(parsed.downloaded_bytes, 1048576);
        assert_eq!(parsed.total_bytes, 10485760);
        assert_eq!(parsed.eta_seconds, Some(17.5));
        assert_eq!(parsed.files_count, Some(2));
    }

    #[test]
    fn test_parse_json_without_files() {
        let json_line = r#"{"type":"progress","downloaded":2097152,"total":10485760,"speed":1048576.0,"progress":20.0,"eta":8.0}"#;

        let result = parse_progress_line(json_line);
        assert!(result.is_some());

        let parsed = result.unwrap();
        assert_eq!(parsed.progress, 20.0);
        assert_eq!(parsed.files_count, None);
    }

    #[test]
    fn test_parse_non_progress_type() {
        let json_line = r#"{"type":"info","message":"Starting download"}"#;

        let result = parse_progress_line(json_line);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_invalid_json() {
        let invalid_line = "This is not JSON";

        let result = parse_progress_line(invalid_line);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_malformed_json() {
        let malformed_line = r#"{"type":"progress","downloaded":1048576"#;

        let result = parse_progress_line(malformed_line);
        assert!(result.is_none());
    }
}
