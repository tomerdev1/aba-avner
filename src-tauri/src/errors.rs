use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DedupeError {
    #[error("Invalid input directory: {0}")]
    InvalidInputDir(String),
    #[error("Invalid output directory: {0}")]
    InvalidOutputDir(String),
    #[error("Invalid selection: {0}")]
    InvalidSelection(String),
    #[error("Invalid similarity threshold: {0}. Supported range is 0..=64")]
    InvalidSimilarityThreshold(u32),
    #[error("Input and output directories conflict: {0}")]
    InputOutputConflict(String),
    #[error("Dedupe run cancelled")]
    Cancelled,
    #[error("Failed to read directory: {0}")]
    ReadDir(String),
    #[error("Failed to load image: {0}")]
    LoadImage(String),
    #[error("Failed to copy image: {0}")]
    CopyImage(String),
}

pub type Result<T> = std::result::Result<T, DedupeError>;

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TauriError {
    pub code: &'static str,
    pub message: String,
}

impl TauriError {
    pub const UNKNOWN_CODE: &'static str = "unknown";
    pub const RUN_IN_PROGRESS_CODE: &'static str = "run_in_progress";

    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn unknown(message: impl Into<String>) -> Self {
        Self::new(Self::UNKNOWN_CODE, message)
    }
}

impl DedupeError {
    pub fn code(&self) -> &'static str {
        match self {
            DedupeError::InvalidInputDir(_) => "invalid_input_dir",
            DedupeError::InvalidOutputDir(_) => "invalid_output_dir",
            DedupeError::InvalidSelection(_) => "invalid_selection",
            DedupeError::InvalidSimilarityThreshold(_) => "invalid_similarity_threshold",
            DedupeError::InputOutputConflict(_) => "input_output_conflict",
            DedupeError::Cancelled => "cancelled",
            DedupeError::ReadDir(_) => "read_dir_failed",
            DedupeError::LoadImage(_) => "load_image_failed",
            DedupeError::CopyImage(_) => "copy_image_failed",
        }
    }
}

impl From<DedupeError> for TauriError {
    fn from(error: DedupeError) -> Self {
        Self::new(error.code(), error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{DedupeError, TauriError};
    use serde_json::json;

    #[test]
    fn dedupe_errors_map_to_stable_tauri_codes() {
        let cases = [
            (
                DedupeError::InvalidInputDir("/input".to_string()),
                "invalid_input_dir",
                "Invalid input directory: /input",
            ),
            (
                DedupeError::InvalidOutputDir("/output".to_string()),
                "invalid_output_dir",
                "Invalid output directory: /output",
            ),
            (
                DedupeError::InvalidSelection("group g1 points to /bad.jpg".to_string()),
                "invalid_selection",
                "Invalid selection: group g1 points to /bad.jpg",
            ),
            (
                DedupeError::InvalidSimilarityThreshold(65),
                "invalid_similarity_threshold",
                "Invalid similarity threshold: 65. Supported range is 0..=64",
            ),
            (
                DedupeError::InputOutputConflict("/input <-> /output".to_string()),
                "input_output_conflict",
                "Input and output directories conflict: /input <-> /output",
            ),
            (DedupeError::Cancelled, "cancelled", "Dedupe run cancelled"),
            (
                DedupeError::ReadDir("boom".to_string()),
                "read_dir_failed",
                "Failed to read directory: boom",
            ),
            (
                DedupeError::LoadImage("bad.png".to_string()),
                "load_image_failed",
                "Failed to load image: bad.png",
            ),
            (
                DedupeError::CopyImage("copy failed".to_string()),
                "copy_image_failed",
                "Failed to copy image: copy failed",
            ),
        ];

        for (source, expected_code, expected_message) in cases {
            let mapped = TauriError::from(source);
            assert_eq!(mapped.code, expected_code);
            assert_eq!(mapped.message, expected_message);
        }
    }

    #[test]
    fn tauri_error_serializes_to_frontend_contract() {
        let payload = serde_json::to_value(TauriError {
            code: "invalid_input_dir",
            message: "Invalid input directory: /input".to_string(),
        })
        .unwrap();

        assert_eq!(
            payload,
            json!({
                "code": "invalid_input_dir",
                "message": "Invalid input directory: /input"
            })
        );
    }

    #[test]
    fn unknown_tauri_error_preserves_message() {
        let mapped = TauriError::unknown("disk full");

        assert_eq!(mapped.code, TauriError::UNKNOWN_CODE);
        assert_eq!(mapped.message, "disk full");
    }

    #[test]
    fn dedupe_error_exposes_stable_codes() {
        assert_eq!(DedupeError::Cancelled.code(), "cancelled");
        assert_eq!(
            DedupeError::CopyImage("copy failed".to_string()).code(),
            "copy_image_failed"
        );
    }
}
