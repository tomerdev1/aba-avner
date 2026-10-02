use crate::errors::DedupeError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

pub const MAX_SIMILARITY_THRESHOLD: u32 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppLocale {
    En,
    He,
}

impl AppLocale {
    pub fn from_user_input(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "he" | "he-il" => Self::He,
            _ => Self::En,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunDedupeRequest {
    pub input_dir: PathBuf,
    pub output_dir: PathBuf,
    pub similarity_threshold: u32,
    pub min_image_size_bytes: u64,
    pub filter_existing_output: bool,
    pub locale: AppLocale,
}

impl RunDedupeRequest {
    pub fn new(
        input_dir: impl Into<String>,
        output_dir: impl Into<String>,
        similarity_threshold: u32,
        min_image_size_bytes: u64,
        filter_existing_output: bool,
    ) -> Result<Self, DedupeError> {
        Self::new_with_locale(
            input_dir,
            output_dir,
            similarity_threshold,
            min_image_size_bytes,
            filter_existing_output,
            "en",
        )
    }

    pub fn new_with_locale(
        input_dir: impl Into<String>,
        output_dir: impl Into<String>,
        similarity_threshold: u32,
        min_image_size_bytes: u64,
        filter_existing_output: bool,
        locale: impl AsRef<str>,
    ) -> Result<Self, DedupeError> {
        let input_dir = input_dir.into();
        let output_dir = output_dir.into();

        Ok(Self {
            input_dir: validate_directory_path(&input_dir, DedupeError::InvalidInputDir)?,
            output_dir: validate_directory_path(&output_dir, DedupeError::InvalidOutputDir)?,
            similarity_threshold: validate_similarity_threshold(similarity_threshold)?,
            min_image_size_bytes,
            filter_existing_output,
            locale: AppLocale::from_user_input(locale.as_ref()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunDedupeResult {
    pub total_images: usize,
    pub unique_images: usize,
    pub duplicate_images: usize,
    pub report: DedupeReport,
    pub output_dir: PathBuf,
    pub warnings: Vec<RunWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrepareReviewResult {
    pub groups: Vec<DuplicateReviewGroup>,
    pub warnings: Vec<RunWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportSelectionsRequest {
    pub output_dir: PathBuf,
    pub similarity_threshold: u32,
    pub filter_existing_output: bool,
    pub export_similar_image_groups: bool,
    pub locale: AppLocale,
    pub groups: Vec<DuplicateReviewGroup>,
    pub selections: HashMap<String, PathBuf>,
}

impl ExportSelectionsRequest {
    pub fn new_with_locale(
        output_dir: impl Into<String>,
        similarity_threshold: u32,
        filter_existing_output: bool,
        export_similar_image_groups: bool,
        locale: impl AsRef<str>,
        groups: Vec<DuplicateReviewGroup>,
        selections: HashMap<String, String>,
    ) -> Result<Self, DedupeError> {
        Ok(Self {
            output_dir: validate_directory_path(&output_dir.into(), DedupeError::InvalidOutputDir)?,
            similarity_threshold: validate_similarity_threshold(similarity_threshold)?,
            filter_existing_output,
            export_similar_image_groups,
            locale: AppLocale::from_user_input(locale.as_ref()),
            groups,
            selections: selections
                .into_iter()
                .map(|(group_id, path)| {
                    validate_image_path(&path).map(|normalized| (group_id, normalized))
                })
                .collect::<Result<HashMap<_, _>, _>>()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateReviewGroup {
    pub group_id: String,
    pub images: Vec<String>,
    pub suggested: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DedupeReport {
    pub storage_saved_bytes: u64,
    pub storage_saved_human: String,
    pub summary: DedupeSummary,
    pub folder_breakdown: Vec<FolderBreakdownEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DedupeSummary {
    pub total_images: usize,
    pub unique_images: usize,
    pub duplicate_images: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderBreakdownEntry {
    pub path: String,
    pub duplicates: usize,
    pub wasted_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunWarning {
    pub code: &'static str,
    pub path: PathBuf,
    pub detail: Option<String>,
}

impl RunWarning {
    pub const FILE_ISSUE_CODE: &'static str = "file_issue";
    pub const OUTPUT_DIR_SCAN_FAILED_CODE: &'static str = "output_dir_scan_failed";
    pub const SIMILAR_IMAGE_IN_OUTPUT_CODE: &'static str = "similar_image_in_output";

    pub fn file_issue(path: PathBuf, detail: impl Into<String>) -> Self {
        Self {
            code: Self::FILE_ISSUE_CODE,
            path,
            detail: Some(detail.into()),
        }
    }

    pub fn output_dir_scan_failed(path: PathBuf, detail: impl Into<String>) -> Self {
        Self {
            code: Self::OUTPUT_DIR_SCAN_FAILED_CODE,
            path,
            detail: Some(detail.into()),
        }
    }

    pub fn similar_image_in_output(path: PathBuf) -> Self {
        Self {
            code: Self::SIMILAR_IMAGE_IN_OUTPUT_CODE,
            path,
            detail: None,
        }
    }

    pub fn to_message(&self) -> String {
        match self.code {
            Self::FILE_ISSUE_CODE | Self::OUTPUT_DIR_SCAN_FAILED_CODE => format!(
                "{}: {}",
                self.path.display(),
                self.detail.as_deref().unwrap_or_default()
            ),
            Self::SIMILAR_IMAGE_IN_OUTPUT_CODE => format!(
                "Skipped {} because a similar image already exists in the output folder.",
                self.path.display()
            ),
            _ => format!(
                "{}: {}",
                self.path.display(),
                self.detail.as_deref().unwrap_or_default()
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgressUpdate {
    pub stage: String,
    pub current: usize,
    pub total: usize,
    pub message: String,
}

impl ProgressUpdate {
    pub fn new(
        stage: impl Into<String>,
        current: usize,
        total: usize,
        message: impl Into<String>,
    ) -> Self {
        Self {
            stage: stage.into(),
            current,
            total,
            message: message.into(),
        }
    }
}

fn validate_directory_path(
    value: &str,
    error: impl FnOnce(String) -> DedupeError,
) -> Result<PathBuf, DedupeError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(error(value.to_string()));
    }

    Ok(normalize_user_supplied_path(trimmed))
}

fn validate_similarity_threshold(value: u32) -> Result<u32, DedupeError> {
    if value > MAX_SIMILARITY_THRESHOLD {
        return Err(DedupeError::InvalidSimilarityThreshold(value));
    }

    Ok(value)
}

fn validate_image_path(value: &str) -> Result<PathBuf, DedupeError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(DedupeError::InvalidSelection(
            "selection path cannot be empty".to_string(),
        ));
    }

    Ok(normalize_user_supplied_path(trimmed))
}

fn normalize_user_supplied_path(value: &str) -> PathBuf {
    normalize_path(Path::new(value))
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if matches!(
                    normalized.components().next_back(),
                    Some(Component::Normal(_))
                ) {
                    normalized.pop();
                } else {
                    normalized.push(component.as_os_str());
                }
            }
            other => normalized.push(other.as_os_str()),
        }
    }

    normalized
}

#[cfg(test)]
mod tests {
    use super::{
        AppLocale, DuplicateReviewGroup, ExportSelectionsRequest, RunDedupeRequest, RunWarning,
    };
    use crate::errors::DedupeError;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn request_new_trims_input_and_output_directories() {
        let request = RunDedupeRequest::new(" /input ", " /output ", 10, 4096, true).unwrap();

        assert_eq!(request.input_dir, PathBuf::from("/input"));
        assert_eq!(request.output_dir, PathBuf::from("/output"));
        assert_eq!(request.similarity_threshold, 10);
        assert_eq!(request.min_image_size_bytes, 4096);
        assert!(request.filter_existing_output);
        assert_eq!(request.locale, AppLocale::En);
    }

    #[test]
    fn request_new_normalizes_dot_segments() {
        let request = RunDedupeRequest::new(
            "/input/./nested/../images",
            "./output/../deduped",
            10,
            0,
            true,
        )
        .unwrap();

        assert_eq!(request.input_dir, PathBuf::from("/input/images"));
        assert_eq!(request.output_dir, PathBuf::from("deduped"));
    }

    #[test]
    fn request_new_with_locale_normalizes_supported_locale_values() {
        let request =
            RunDedupeRequest::new_with_locale("/input", "/output", 10, 0, true, "he-IL").unwrap();

        assert_eq!(request.locale, AppLocale::He);
    }

    #[test]
    fn request_new_rejects_blank_input_dir() {
        let error = RunDedupeRequest::new("   ", "/output", 10, 0, true).unwrap_err();

        match error {
            DedupeError::InvalidInputDir(path) => assert_eq!(path, "   "),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn request_new_rejects_blank_output_dir() {
        let error = RunDedupeRequest::new("/input", "", 10, 0, true).unwrap_err();

        match error {
            DedupeError::InvalidOutputDir(path) => assert_eq!(path, ""),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn request_new_accepts_similarity_threshold_upper_bound() {
        let request = RunDedupeRequest::new("/input", "/output", 64, 0, true).unwrap();

        assert_eq!(request.similarity_threshold, 64);
    }

    #[test]
    fn request_new_rejects_similarity_threshold_above_hash_width() {
        let error = RunDedupeRequest::new("/input", "/output", 65, 0, true).unwrap_err();

        match error {
            DedupeError::InvalidSimilarityThreshold(threshold) => assert_eq!(threshold, 65),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn run_warning_formats_existing_output_skip_message() {
        let warning = RunWarning::similar_image_in_output(PathBuf::from("/images/a.png"));

        assert_eq!(
            warning.to_message(),
            "Skipped /images/a.png because a similar image already exists in the output folder."
        );
    }

    #[test]
    fn run_warning_formats_path_and_detail_message() {
        let warning = RunWarning::file_issue(PathBuf::from("/images/a.png"), "permission denied");

        assert_eq!(warning.to_message(), "/images/a.png: permission denied");
    }

    #[test]
    fn export_request_normalizes_output_and_selection_paths() {
        let request = ExportSelectionsRequest::new_with_locale(
            "./output/../deduped",
            10,
            true,
            false,
            "he",
            vec![DuplicateReviewGroup {
                group_id: "g1".to_string(),
                images: vec!["/input/a.jpg".to_string(), "/input/b.jpg".to_string()],
                suggested: "/input/a.jpg".to_string(),
            }],
            HashMap::from([("g1".to_string(), "./picked/../picked/b.jpg".to_string())]),
        )
        .unwrap();

        assert_eq!(request.output_dir, PathBuf::from("deduped"));
        assert_eq!(
            request.selections,
            HashMap::from([("g1".to_string(), PathBuf::from("picked/b.jpg"))])
        );
        assert!(!request.export_similar_image_groups);
        assert_eq!(request.locale, AppLocale::He);
    }

    #[test]
    fn export_request_rejects_empty_selection_path() {
        let error = ExportSelectionsRequest::new_with_locale(
            "/output",
            10,
            true,
            true,
            "en",
            vec![],
            HashMap::from([("g1".to_string(), "   ".to_string())]),
        )
        .unwrap_err();

        match error {
            DedupeError::InvalidSelection(message) => {
                assert_eq!(message, "selection path cannot be empty");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
