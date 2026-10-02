use crate::application::{
    DedupeReport, DuplicateReviewGroup, ExportSelectionsRequest, PrepareReviewResult,
    ProgressUpdate, RunDedupeRequest, RunDedupeResult, RunWarning,
};
use crate::errors::DedupeError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DedupeConfig {
    pub input_dir: String,
    pub output_dir: String,
    pub similarity_threshold: u32,
    pub min_image_size_bytes: u64,
    pub filter_existing_output: bool,
    #[serde(default)]
    pub run_id: String,
    #[serde(default = "default_locale")]
    pub locale: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSelectionsConfig {
    pub output_dir: String,
    pub similarity_threshold: u32,
    pub filter_existing_output: bool,
    #[serde(default = "default_export_similar_image_groups")]
    pub export_similar_image_groups: bool,
    pub groups: Vec<DuplicateReviewGroup>,
    #[serde(default)]
    pub selections: HashMap<String, String>,
    #[serde(default)]
    pub run_id: String,
    #[serde(default = "default_locale")]
    pub locale: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub run_id: String,
    pub stage: String,
    pub current: usize,
    pub total: usize,
    pub message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DedupeResult {
    pub total_images: usize,
    pub unique_images: usize,
    pub duplicate_images: usize,
    pub report: DedupeReport,
    pub output_dir: String,
    pub warnings: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warning_details: Vec<WarningDetail>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewGroupsResult {
    pub groups: Vec<DuplicateReviewGroup>,
    pub warnings: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warning_details: Vec<WarningDetail>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WarningDetail {
    pub code: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl TryFrom<DedupeConfig> for RunDedupeRequest {
    type Error = DedupeError;

    fn try_from(config: DedupeConfig) -> Result<Self, Self::Error> {
        RunDedupeRequest::new_with_locale(
            config.input_dir,
            config.output_dir,
            config.similarity_threshold,
            config.min_image_size_bytes,
            config.filter_existing_output,
            config.locale,
        )
    }
}

impl TryFrom<ExportSelectionsConfig> for ExportSelectionsRequest {
    type Error = DedupeError;

    fn try_from(config: ExportSelectionsConfig) -> Result<Self, Self::Error> {
        ExportSelectionsRequest::new_with_locale(
            config.output_dir,
            config.similarity_threshold,
            config.filter_existing_output,
            config.export_similar_image_groups,
            config.locale,
            config.groups,
            config.selections,
        )
    }
}

fn default_locale() -> String {
    "en".to_string()
}

fn default_export_similar_image_groups() -> bool {
    true
}

impl From<RunDedupeResult> for DedupeResult {
    fn from(result: RunDedupeResult) -> Self {
        let warning_details: Vec<WarningDetail> = result
            .warnings
            .into_iter()
            .map(WarningDetail::from)
            .collect();
        let warnings = warning_details
            .iter()
            .map(WarningDetail::to_message)
            .collect();

        Self {
            total_images: result.total_images,
            unique_images: result.unique_images,
            duplicate_images: result.duplicate_images,
            report: result.report,
            output_dir: result.output_dir.display().to_string(),
            warnings,
            warning_details,
        }
    }
}

impl From<PrepareReviewResult> for ReviewGroupsResult {
    fn from(result: PrepareReviewResult) -> Self {
        let warning_details: Vec<WarningDetail> = result
            .warnings
            .into_iter()
            .map(WarningDetail::from)
            .collect();
        let warnings = warning_details
            .iter()
            .map(WarningDetail::to_message)
            .collect();

        Self {
            groups: result.groups,
            warnings,
            warning_details,
        }
    }
}

impl WarningDetail {
    fn to_message(&self) -> String {
        match self.code.as_str() {
            RunWarning::SIMILAR_IMAGE_IN_OUTPUT_CODE => format!(
                "Skipped {} because a similar image already exists in the output folder.",
                self.path
            ),
            _ => format!(
                "{}: {}",
                self.path,
                self.detail.as_deref().unwrap_or_default()
            ),
        }
    }
}

impl From<RunWarning> for WarningDetail {
    fn from(warning: RunWarning) -> Self {
        Self {
            code: warning.code.to_string(),
            path: warning.path.display().to_string(),
            detail: warning.detail,
        }
    }
}

impl ProgressEvent {
    pub fn from_update(run_id: impl Into<String>, update: ProgressUpdate) -> Self {
        Self {
            run_id: run_id.into(),
            stage: update.stage,
            current: update.current,
            total: update.total,
            message: update.message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DedupeConfig, DedupeResult, ExportSelectionsConfig, ProgressEvent, ReviewGroupsResult,
        WarningDetail,
    };
    use crate::application::{
        DuplicateReviewGroup, PrepareReviewResult, ProgressUpdate, RunDedupeRequest,
        RunDedupeResult, RunWarning,
    };
    use serde_json::json;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn dedupe_config_deserializes_from_current_tauri_payload_shape() {
        let config: DedupeConfig = serde_json::from_value(json!({
            "inputDir": "/input",
            "outputDir": "/output",
            "similarityThreshold": 10,
            "minImageSizeBytes": 4096,
            "filterExistingOutput": true,
            "runId": "run-123",
            "locale": "he"
        }))
        .unwrap();

        assert_eq!(config.input_dir, "/input");
        assert_eq!(config.output_dir, "/output");
        assert_eq!(config.similarity_threshold, 10);
        assert_eq!(config.min_image_size_bytes, 4096);
        assert!(config.filter_existing_output);
        assert_eq!(config.run_id, "run-123");
        assert_eq!(config.locale, "he");
    }

    #[test]
    fn progress_event_serializes_to_current_frontend_contract() {
        let payload = serde_json::to_value(ProgressEvent {
            run_id: "run-123".to_string(),
            stage: "hash".to_string(),
            current: 3,
            total: 5,
            message: "Hashing images".to_string(),
        })
        .unwrap();

        assert_eq!(
            payload,
            json!({
                "runId": "run-123",
                "stage": "hash",
                "current": 3,
                "total": 5,
                "message": "Hashing images"
            })
        );
    }

    #[test]
    fn application_progress_maps_to_transport_progress_event() {
        let payload = ProgressEvent::from_update(
            "run-123",
            ProgressUpdate::new("copy", 2, 3, "Copy complete"),
        );

        assert_eq!(payload.run_id, "run-123");
        assert_eq!(payload.stage, "copy");
        assert_eq!(payload.current, 2);
        assert_eq!(payload.total, 3);
        assert_eq!(payload.message, "Copy complete");
    }

    #[test]
    fn dedupe_result_serializes_to_current_frontend_contract() {
        let payload = serde_json::to_value(DedupeResult {
            total_images: 8,
            unique_images: 5,
            duplicate_images: 3,
            report: crate::application::DedupeReport {
                storage_saved_bytes: 123_456_789,
                storage_saved_human: "117.7 MB".to_string(),
                summary: crate::application::DedupeSummary {
                    total_images: 8,
                    unique_images: 5,
                    duplicate_images: 3,
                },
                folder_breakdown: vec![crate::application::FolderBreakdownEntry {
                    path: "/photos/whatsapp".to_string(),
                    duplicates: 2,
                    wasted_bytes: 50_000_000,
                }],
            },
            output_dir: "/output".to_string(),
            warnings: vec!["warning 1".to_string(), "warning 2".to_string()],
            warning_details: vec![
                WarningDetail {
                    code: "file_issue".to_string(),
                    path: "/input/a.png".to_string(),
                    detail: Some("failed".to_string()),
                },
                WarningDetail {
                    code: "similar_image_in_output".to_string(),
                    path: "/input/b.png".to_string(),
                    detail: None,
                },
            ],
        })
        .unwrap();

        assert_eq!(
            payload,
            json!({
                "totalImages": 8,
                "uniqueImages": 5,
                "duplicateImages": 3,
                "report": {
                    "storageSavedBytes": 123456789,
                    "storageSavedHuman": "117.7 MB",
                    "summary": {
                        "totalImages": 8,
                        "uniqueImages": 5,
                        "duplicateImages": 3
                    },
                    "folderBreakdown": [
                        {
                            "path": "/photos/whatsapp",
                            "duplicates": 2,
                            "wastedBytes": 50000000
                        }
                    ]
                },
                "outputDir": "/output",
                "warnings": ["warning 1", "warning 2"],
                "warningDetails": [
                    {
                        "code": "file_issue",
                        "path": "/input/a.png",
                        "detail": "failed"
                    },
                    {
                        "code": "similar_image_in_output",
                        "path": "/input/b.png"
                    }
                ]
            })
        );
    }

    #[test]
    fn transport_config_maps_to_validated_application_request() {
        let request = RunDedupeRequest::try_from(DedupeConfig {
            input_dir: " /input ".to_string(),
            output_dir: " /output ".to_string(),
            similarity_threshold: 10,
            min_image_size_bytes: 4096,
            filter_existing_output: false,
            run_id: String::new(),
            locale: "he".to_string(),
        })
        .unwrap();

        assert_eq!(request.input_dir, PathBuf::from("/input"));
        assert_eq!(request.output_dir, PathBuf::from("/output"));
        assert_eq!(request.similarity_threshold, 10);
        assert_eq!(request.min_image_size_bytes, 4096);
        assert!(!request.filter_existing_output);
        assert_eq!(request.locale, crate::application::AppLocale::He);
    }

    #[test]
    fn transport_config_rejects_similarity_threshold_above_hash_width() {
        let error = RunDedupeRequest::try_from(DedupeConfig {
            input_dir: "/input".to_string(),
            output_dir: "/output".to_string(),
            similarity_threshold: 65,
            min_image_size_bytes: 0,
            filter_existing_output: false,
            run_id: String::new(),
            locale: "en".to_string(),
        })
        .unwrap_err();

        match error {
            crate::errors::DedupeError::InvalidSimilarityThreshold(threshold) => {
                assert_eq!(threshold, 65)
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn application_result_maps_to_transport_result() {
        let payload = DedupeResult::from(RunDedupeResult {
            total_images: 8,
            unique_images: 5,
            duplicate_images: 3,
            report: crate::application::DedupeReport {
                storage_saved_bytes: 123_456_789,
                storage_saved_human: "117.7 MB".to_string(),
                summary: crate::application::DedupeSummary {
                    total_images: 8,
                    unique_images: 5,
                    duplicate_images: 3,
                },
                folder_breakdown: vec![crate::application::FolderBreakdownEntry {
                    path: "/photos/whatsapp".to_string(),
                    duplicates: 2,
                    wasted_bytes: 50_000_000,
                }],
            },
            output_dir: PathBuf::from("/output"),
            warnings: vec![
                RunWarning::file_issue(PathBuf::from("/input/a.png"), "failed"),
                RunWarning::similar_image_in_output(PathBuf::from("/input/b.png")),
            ],
        });

        assert_eq!(payload.output_dir, "/output");
        assert_eq!(payload.total_images, 8);
        assert_eq!(payload.unique_images, 5);
        assert_eq!(payload.duplicate_images, 3);
        assert_eq!(payload.report.storage_saved_bytes, 123_456_789);
        assert_eq!(payload.report.storage_saved_human, "117.7 MB");
        assert_eq!(payload.report.summary.duplicate_images, 3);
        assert_eq!(payload.report.folder_breakdown.len(), 1);
        assert_eq!(payload.warnings.len(), 2);
        assert_eq!(payload.warning_details.len(), 2);
        assert_eq!(
            payload.warnings,
            vec![
                "/input/a.png: failed".to_string(),
                "Skipped /input/b.png because a similar image already exists in the output folder."
                    .to_string(),
            ]
        );
    }

    #[test]
    fn export_selections_config_deserializes_from_frontend_payload_shape() {
        let config: ExportSelectionsConfig = serde_json::from_value(json!({
            "outputDir": "/output",
            "similarityThreshold": 10,
            "filterExistingOutput": true,
            "exportSimilarImageGroups": false,
            "groups": [{
                "groupId": "g1",
                "images": ["/input/a.jpg", "/input/b.jpg"],
                "suggested": "/input/a.jpg"
            }],
            "selections": {
                "g1": "/input/b.jpg"
            },
            "runId": "run-456",
            "locale": "en"
        }))
        .unwrap();

        assert_eq!(config.output_dir, "/output");
        assert!(!config.export_similar_image_groups);
        assert_eq!(config.groups.len(), 1);
        assert_eq!(
            config.selections,
            HashMap::from([("g1".to_string(), "/input/b.jpg".to_string())])
        );
        assert_eq!(config.run_id, "run-456");
    }

    #[test]
    fn export_selections_config_defaults_similar_group_export_to_true() {
        let config: ExportSelectionsConfig = serde_json::from_value(json!({
            "outputDir": "/output",
            "similarityThreshold": 10,
            "filterExistingOutput": true,
            "groups": [],
            "runId": "run-456",
            "locale": "en"
        }))
        .unwrap();

        assert!(config.export_similar_image_groups);
    }

    #[test]
    fn review_groups_result_serializes_groups_and_warnings() {
        let payload = serde_json::to_value(ReviewGroupsResult::from(PrepareReviewResult {
            groups: vec![DuplicateReviewGroup {
                group_id: "g1".to_string(),
                images: vec!["/input/a.jpg".to_string(), "/input/b.jpg".to_string()],
                suggested: "/input/a.jpg".to_string(),
            }],
            warnings: vec![RunWarning::file_issue(
                PathBuf::from("/input/broken.jpg"),
                "failed to read".to_string(),
            )],
        }))
        .unwrap();

        assert_eq!(
            payload,
            json!({
                "groups": [{
                    "groupId": "g1",
                    "images": ["/input/a.jpg", "/input/b.jpg"],
                    "suggested": "/input/a.jpg"
                }],
                "warnings": ["/input/broken.jpg: failed to read"],
                "warningDetails": [{
                    "code": "file_issue",
                    "path": "/input/broken.jpg",
                    "detail": "failed to read"
                }]
            })
        );
    }
}
