use aba_avner::application::{
    run_dedupe_cancellable, run_dedupe_with_filter, AppLocale, CancellationToken, FileIssue,
    FileSizeReader, ImageHasher, ImageScanner, OutputDirectoryFilter, OutputFilterResult,
    OutputWriter, ProgressPublisher, ProgressUpdate, RunDedupeRequest, RunWarning, WarningLog,
};
use aba_avner::errors::{DedupeError, Result};
use aba_avner::hashing::compute_dhash;
use aba_avner::infrastructure::{
    run_dedupe_with_filesystem, run_dedupe_with_filesystem_and_cancellation,
    AtomicCancellationToken,
};
use aba_avner::presentation::contracts::DedupeConfig;
use aba_avner::scan::collect_image_paths;
use image::{DynamicImage, ImageBuffer, Rgba};
use serde::Deserialize;
use std::fs::write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tempfile::tempdir;

struct NoopPublisher;

impl ProgressPublisher for NoopPublisher {
    fn emit(&mut self, _update: ProgressUpdate) {}
}

#[test]
fn dedupe_config_converts_to_validated_request() {
    let request = RunDedupeRequest::try_from(DedupeConfig {
        input_dir: " /input ".to_string(),
        output_dir: " /output ".to_string(),
        similarity_threshold: 10,
        min_image_size_bytes: 4096,
        filter_existing_output: true,
        run_id: String::new(),
        locale: "en".to_string(),
    })
    .unwrap();

    assert_eq!(request.input_dir, PathBuf::from("/input"));
    assert_eq!(request.output_dir, PathBuf::from("/output"));
    assert_eq!(request.similarity_threshold, 10);
    assert_eq!(request.min_image_size_bytes, 4096);
    assert!(request.filter_existing_output);
}

#[test]
fn blank_input_dir_is_rejected_during_request_validation() {
    let error = RunDedupeRequest::try_from(DedupeConfig {
        input_dir: "   ".to_string(),
        output_dir: "/output".to_string(),
        similarity_threshold: 10,
        min_image_size_bytes: 0,
        filter_existing_output: true,
        run_id: String::new(),
        locale: "en".to_string(),
    })
    .unwrap_err();

    match error {
        DedupeError::InvalidInputDir(path) => assert_eq!(path, "   "),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn blank_output_dir_is_rejected_during_request_validation() {
    let error = RunDedupeRequest::try_from(DedupeConfig {
        input_dir: "/input".to_string(),
        output_dir: "".to_string(),
        similarity_threshold: 10,
        min_image_size_bytes: 0,
        filter_existing_output: true,
        run_id: String::new(),
        locale: "en".to_string(),
    })
    .unwrap_err();

    match error {
        DedupeError::InvalidOutputDir(path) => assert_eq!(path, ""),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn oversized_similarity_threshold_is_rejected_during_request_validation() {
    let error = RunDedupeRequest::try_from(DedupeConfig {
        input_dir: "/input".to_string(),
        output_dir: "/output".to_string(),
        similarity_threshold: 65,
        min_image_size_bytes: 0,
        filter_existing_output: true,
        run_id: String::new(),
        locale: "en".to_string(),
    })
    .unwrap_err();

    match error {
        DedupeError::InvalidSimilarityThreshold(threshold) => assert_eq!(threshold, 65),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn filesystem_run_rejects_same_input_and_output_directory() {
    let temp = tempdir().unwrap();
    let dir = temp.path().join("images");
    std::fs::create_dir_all(&dir).unwrap();

    let mut publisher = NoopPublisher;
    let error = run_dedupe_with_filesystem(
        request(dir.display().to_string(), dir.display().to_string(), 0, 0),
        &mut publisher,
    )
    .unwrap_err();

    match error {
        DedupeError::InputOutputConflict(detail) => {
            assert!(detail.contains("same directory"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn filesystem_run_rejects_output_nested_under_input_directory() {
    let temp = tempdir().unwrap();
    let input_dir = temp.path().join("images");
    let output_dir = input_dir.join("deduped");
    std::fs::create_dir_all(&input_dir).unwrap();
    create_pattern_image(&input_dir.join("a.bmp"), 64, 64, 0);

    let mut publisher = NoopPublisher;
    let error = run_dedupe_with_filesystem(
        request(
            input_dir.display().to_string(),
            output_dir.display().to_string(),
            0,
            0,
        ),
        &mut publisher,
    )
    .unwrap_err();

    match error {
        DedupeError::InputOutputConflict(detail) => {
            assert!(detail.contains("deduped"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn progress_publisher_trait_can_be_implemented_by_non_closure_publishers() {
    struct RecordingPublisher {
        events: Vec<ProgressUpdate>,
    }

    impl ProgressPublisher for RecordingPublisher {
        fn emit(&mut self, update: ProgressUpdate) {
            self.events.push(update);
        }
    }

    let temp = tempdir().unwrap();
    let input_dir = temp.path().join("input");
    let output_dir = temp.path().join("output");
    std::fs::create_dir_all(&input_dir).unwrap();
    create_pattern_image(&input_dir.join("a.bmp"), 64, 64, 0);

    let mut publisher = RecordingPublisher { events: Vec::new() };
    let result = run_dedupe_with_filesystem(
        request(
            input_dir.display().to_string(),
            output_dir.display().to_string(),
            0,
            0,
        ),
        &mut publisher,
    )
    .unwrap();

    assert_eq!(result.total_images, 1);
    assert!(!publisher.events.is_empty());
}

#[test]
fn use_case_can_run_with_mock_scanner_and_writer() {
    struct MockScanner {
        input_root: PathBuf,
        input_paths: Vec<PathBuf>,
    }

    impl ImageScanner for MockScanner {
        fn collect_image_paths(&self, root: &Path) -> Result<Vec<PathBuf>> {
            if root == self.input_root.as_path() {
                return Ok(self.input_paths.clone());
            }

            Err(DedupeError::ReadDir(root.display().to_string()))
        }
    }

    struct MockWriter;

    impl OutputWriter for MockWriter {
        fn ensure_output_dir(&self, _path: &Path) -> Result<()> {
            Ok(())
        }

        fn copy_unique_images<F>(
            &self,
            paths: &[PathBuf],
            _output_dir: &Path,
            _cancellation: &dyn aba_avner::application::CancellationToken,
            mut on_progress: F,
        ) -> Result<Vec<PathBuf>>
        where
            F: FnMut(usize),
        {
            for idx in 0..paths.len() {
                on_progress(idx + 1);
            }
            Ok(paths.to_vec())
        }

        fn copy_non_unique_groups(
            &self,
            _groups: &[Vec<aba_avner::domain::ImageEntry>],
            _selected_paths: &[PathBuf],
            _output_dir: &Path,
            _locale: AppLocale,
            _export_similar_image_groups: bool,
            _cancellation: &dyn aba_avner::application::CancellationToken,
        ) -> Result<()> {
            Ok(())
        }
    }

    struct MockHasher;

    impl ImageHasher for MockHasher {
        fn load_and_hash(&self, path: &Path) -> std::result::Result<u64, FileIssue> {
            let hash = if path.to_string_lossy().contains("duplicate")
                || path.to_string_lossy().contains("existing")
            {
                11
            } else {
                99
            };

            Ok(hash)
        }
    }

    struct MockSizeReader;

    impl FileSizeReader for MockSizeReader {
        fn file_size_bytes(&self, _path: &Path) -> std::result::Result<u64, FileIssue> {
            Ok(10_000)
        }
    }

    struct MockOutputFilter;

    impl OutputDirectoryFilter for MockOutputFilter {
        fn filter_existing_output(
            &self,
            entries: Vec<aba_avner::domain::ImageEntry>,
            _output_dir: &Path,
            _similarity_threshold: u32,
            _cancellation: &dyn aba_avner::application::CancellationToken,
        ) -> Result<OutputFilterResult> {
            Ok(OutputFilterResult {
                entries: Vec::new(),
                warnings: WarningLog::from(vec![RunWarning::similar_image_in_output(
                    entries[0].path.clone(),
                )]),
            })
        }
    }

    let input_dir = PathBuf::from("/virtual-input");
    let output_dir = PathBuf::from("/virtual-output");
    let duplicate_a = input_dir.join("duplicate-a.bmp");
    let duplicate_b = input_dir.join("duplicate-b.bmp");

    let scanner = MockScanner {
        input_root: input_dir.clone(),
        input_paths: vec![duplicate_a, duplicate_b],
    };
    let writer = MockWriter;
    let hasher = MockHasher;
    let size_reader = MockSizeReader;
    let output_filter = MockOutputFilter;
    let mut publisher = NoopPublisher;

    let result = run_dedupe_with_filter(
        request(
            input_dir.display().to_string(),
            output_dir.display().to_string(),
            0,
            0,
        ),
        &scanner,
        &writer,
        &hasher,
        &size_reader,
        &output_filter,
        &mut publisher,
    )
    .unwrap();

    assert_eq!(result.total_images, 2);
    assert_eq!(result.unique_images, 0);
    assert_eq!(result.duplicate_images, 2);
    assert!(result
        .warnings
        .iter()
        .any(|warning| warning.code == RunWarning::SIMILAR_IMAGE_IN_OUTPUT_CODE));
}

#[test]
fn disabled_existing_output_filter_skips_destination_comparison() {
    struct MockScanner {
        input_root: PathBuf,
        input_paths: Vec<PathBuf>,
    }

    impl ImageScanner for MockScanner {
        fn collect_image_paths(&self, root: &Path) -> Result<Vec<PathBuf>> {
            if root == self.input_root.as_path() {
                return Ok(self.input_paths.clone());
            }

            Err(DedupeError::ReadDir(root.display().to_string()))
        }
    }

    struct MockWriter;

    impl OutputWriter for MockWriter {
        fn ensure_output_dir(&self, _path: &Path) -> Result<()> {
            Ok(())
        }

        fn copy_unique_images<F>(
            &self,
            paths: &[PathBuf],
            _output_dir: &Path,
            _cancellation: &dyn aba_avner::application::CancellationToken,
            mut on_progress: F,
        ) -> Result<Vec<PathBuf>>
        where
            F: FnMut(usize),
        {
            for idx in 0..paths.len() {
                on_progress(idx + 1);
            }
            Ok(paths.to_vec())
        }

        fn copy_non_unique_groups(
            &self,
            _groups: &[Vec<aba_avner::domain::ImageEntry>],
            _selected_paths: &[PathBuf],
            _output_dir: &Path,
            _locale: AppLocale,
            _export_similar_image_groups: bool,
            _cancellation: &dyn aba_avner::application::CancellationToken,
        ) -> Result<()> {
            Ok(())
        }
    }

    struct MockHasher;

    impl ImageHasher for MockHasher {
        fn load_and_hash(&self, path: &Path) -> std::result::Result<u64, FileIssue> {
            Ok(path.to_string_lossy().len() as u64)
        }
    }

    struct MockSizeReader;

    impl FileSizeReader for MockSizeReader {
        fn file_size_bytes(&self, _path: &Path) -> std::result::Result<u64, FileIssue> {
            Ok(10_000)
        }
    }

    struct PanicOutputFilter;

    impl OutputDirectoryFilter for PanicOutputFilter {
        fn filter_existing_output(
            &self,
            _entries: Vec<aba_avner::domain::ImageEntry>,
            _output_dir: &Path,
            _similarity_threshold: u32,
            _cancellation: &dyn aba_avner::application::CancellationToken,
        ) -> Result<OutputFilterResult> {
            panic!("output filter should be skipped when filter_existing_output is disabled");
        }
    }

    let input_dir = PathBuf::from("/virtual-input");
    let output_dir = PathBuf::from("/virtual-output");
    let source = input_dir.join("image-a.bmp");

    let scanner = MockScanner {
        input_root: input_dir.clone(),
        input_paths: vec![source],
    };
    let writer = MockWriter;
    let hasher = MockHasher;
    let size_reader = MockSizeReader;
    let output_filter = PanicOutputFilter;
    let mut publisher = NoopPublisher;

    let result = run_dedupe_with_filter(
        RunDedupeRequest::try_from(DedupeConfig {
            input_dir: input_dir.display().to_string(),
            output_dir: output_dir.display().to_string(),
            similarity_threshold: 0,
            min_image_size_bytes: 0,
            filter_existing_output: false,
            run_id: String::new(),
            locale: "en".to_string(),
        })
        .unwrap(),
        &scanner,
        &writer,
        &hasher,
        &size_reader,
        &output_filter,
        &mut publisher,
    )
    .unwrap();

    assert_eq!(result.unique_images, 1);
    assert!(result.warnings.is_empty());
}

#[test]
fn cancellation_after_unique_copy_prevents_non_unique_group_export() {
    struct MockScanner {
        input_root: PathBuf,
        input_paths: Vec<PathBuf>,
    }

    impl ImageScanner for MockScanner {
        fn collect_image_paths(&self, root: &Path) -> Result<Vec<PathBuf>> {
            if root == self.input_root.as_path() {
                return Ok(self.input_paths.clone());
            }

            Err(DedupeError::ReadDir(root.display().to_string()))
        }
    }

    struct CancellingWriter {
        token: AtomicCancellationToken,
    }

    impl OutputWriter for CancellingWriter {
        fn ensure_output_dir(&self, _path: &Path) -> Result<()> {
            Ok(())
        }

        fn copy_unique_images<F>(
            &self,
            paths: &[PathBuf],
            _output_dir: &Path,
            _cancellation: &dyn aba_avner::application::CancellationToken,
            mut on_progress: F,
        ) -> Result<Vec<PathBuf>>
        where
            F: FnMut(usize),
        {
            for idx in 0..paths.len() {
                on_progress(idx + 1);
            }
            self.token.cancel();
            Ok(paths.to_vec())
        }

        fn copy_non_unique_groups(
            &self,
            _groups: &[Vec<aba_avner::domain::ImageEntry>],
            _selected_paths: &[PathBuf],
            _output_dir: &Path,
            _locale: AppLocale,
            _export_similar_image_groups: bool,
            _cancellation: &dyn aba_avner::application::CancellationToken,
        ) -> Result<()> {
            panic!("copy_non_unique_groups should not run after cancellation")
        }
    }

    struct MockHasher;

    impl ImageHasher for MockHasher {
        fn load_and_hash(&self, path: &Path) -> std::result::Result<u64, FileIssue> {
            let hash = if path.to_string_lossy().contains("duplicate") {
                11
            } else {
                99
            };

            Ok(hash)
        }
    }

    struct MockSizeReader;

    impl FileSizeReader for MockSizeReader {
        fn file_size_bytes(&self, _path: &Path) -> std::result::Result<u64, FileIssue> {
            Ok(10_000)
        }
    }

    let input_dir = PathBuf::from("/virtual-input");
    let output_dir = PathBuf::from("/virtual-output");
    let duplicate_a = input_dir.join("duplicate-a.bmp");
    let duplicate_b = input_dir.join("duplicate-b.bmp");

    let scanner = MockScanner {
        input_root: input_dir.clone(),
        input_paths: vec![duplicate_a, duplicate_b],
    };
    let cancellation = AtomicCancellationToken::new();
    let writer = CancellingWriter {
        token: cancellation.clone(),
    };
    let hasher = MockHasher;
    let size_reader = MockSizeReader;
    let mut publisher = NoopPublisher;

    let error = run_dedupe_cancellable(
        request(
            input_dir.display().to_string(),
            output_dir.display().to_string(),
            0,
            0,
        ),
        &scanner,
        &writer,
        &hasher,
        &size_reader,
        &mut publisher,
        &cancellation,
    )
    .unwrap_err();

    assert!(matches!(error, DedupeError::Cancelled));
}

#[test]
fn result_counts_follow_writer_output_count() {
    struct MockScanner {
        input_root: PathBuf,
        input_paths: Vec<PathBuf>,
    }

    impl ImageScanner for MockScanner {
        fn collect_image_paths(&self, root: &Path) -> Result<Vec<PathBuf>> {
            if root == self.input_root.as_path() {
                return Ok(self.input_paths.clone());
            }

            Err(DedupeError::ReadDir(root.display().to_string()))
        }
    }

    struct PartialWriter;

    impl OutputWriter for PartialWriter {
        fn ensure_output_dir(&self, _path: &Path) -> Result<()> {
            Ok(())
        }

        fn copy_unique_images<F>(
            &self,
            paths: &[PathBuf],
            output_dir: &Path,
            _cancellation: &dyn aba_avner::application::CancellationToken,
            mut on_progress: F,
        ) -> Result<Vec<PathBuf>>
        where
            F: FnMut(usize),
        {
            let copied: Vec<PathBuf> = paths
                .iter()
                .take(1)
                .map(|path| output_dir.join(path.file_name().unwrap()))
                .collect();
            for idx in 0..copied.len() {
                on_progress(idx + 1);
            }
            Ok(copied)
        }

        fn copy_non_unique_groups(
            &self,
            _groups: &[Vec<aba_avner::domain::ImageEntry>],
            _selected_paths: &[PathBuf],
            _output_dir: &Path,
            _locale: AppLocale,
            _export_similar_image_groups: bool,
            _cancellation: &dyn aba_avner::application::CancellationToken,
        ) -> Result<()> {
            Ok(())
        }
    }

    struct MockHasher;

    impl ImageHasher for MockHasher {
        fn load_and_hash(&self, path: &Path) -> std::result::Result<u64, FileIssue> {
            Ok(path.to_string_lossy().len() as u64)
        }
    }

    struct MockSizeReader;

    impl FileSizeReader for MockSizeReader {
        fn file_size_bytes(&self, _path: &Path) -> std::result::Result<u64, FileIssue> {
            Ok(10_000)
        }
    }

    #[derive(Default)]
    struct PassthroughFilter;

    impl OutputDirectoryFilter for PassthroughFilter {
        fn filter_existing_output(
            &self,
            entries: Vec<aba_avner::domain::ImageEntry>,
            _output_dir: &Path,
            _similarity_threshold: u32,
            _cancellation: &dyn aba_avner::application::CancellationToken,
        ) -> Result<OutputFilterResult> {
            Ok(OutputFilterResult {
                entries,
                warnings: WarningLog::default(),
            })
        }
    }

    let input_dir = PathBuf::from("/virtual-input");
    let output_dir = PathBuf::from("/virtual-output");
    let scanner = MockScanner {
        input_root: input_dir.clone(),
        input_paths: vec![input_dir.join("one.bmp"), input_dir.join("two.bmp")],
    };
    let writer = PartialWriter;
    let hasher = MockHasher;
    let size_reader = MockSizeReader;
    let output_filter = PassthroughFilter;
    let mut publisher = NoopPublisher;

    let result = run_dedupe_with_filter(
        request(
            input_dir.display().to_string(),
            output_dir.display().to_string(),
            0,
            0,
        ),
        &scanner,
        &writer,
        &hasher,
        &size_reader,
        &output_filter,
        &mut publisher,
    )
    .unwrap();

    assert_eq!(result.total_images, 2);
    assert_eq!(result.unique_images, 1);
    assert_eq!(result.duplicate_images, 1);
}

#[test]
fn dedupe_result_includes_grouping_report_without_extra_post_group_scans() {
    struct MockScanner {
        input_root: PathBuf,
        input_paths: Vec<PathBuf>,
    }

    impl ImageScanner for MockScanner {
        fn collect_image_paths(&self, root: &Path) -> Result<Vec<PathBuf>> {
            if root == self.input_root.as_path() {
                return Ok(self.input_paths.clone());
            }

            Err(DedupeError::ReadDir(root.display().to_string()))
        }
    }

    struct MockWriter;

    impl OutputWriter for MockWriter {
        fn ensure_output_dir(&self, _path: &Path) -> Result<()> {
            Ok(())
        }

        fn copy_unique_images<F>(
            &self,
            paths: &[PathBuf],
            _output_dir: &Path,
            _cancellation: &dyn CancellationToken,
            mut on_progress: F,
        ) -> Result<Vec<PathBuf>>
        where
            F: FnMut(usize),
        {
            for idx in 0..paths.len() {
                on_progress(idx + 1);
            }

            Ok(paths.to_vec())
        }

        fn copy_non_unique_groups(
            &self,
            _groups: &[Vec<aba_avner::domain::ImageEntry>],
            _selected_paths: &[PathBuf],
            _output_dir: &Path,
            _locale: AppLocale,
            _export_similar_image_groups: bool,
            _cancellation: &dyn CancellationToken,
        ) -> Result<()> {
            Ok(())
        }
    }

    struct MockHasher;

    impl ImageHasher for MockHasher {
        fn load_and_hash(&self, path: &Path) -> std::result::Result<u64, FileIssue> {
            if path.ends_with("a.bmp") || path.ends_with("b.bmp") || path.ends_with("c.bmp") {
                Ok(11)
            } else {
                Ok(99)
            }
        }
    }

    struct MockSizeReader;

    impl FileSizeReader for MockSizeReader {
        fn file_size_bytes(&self, path: &Path) -> std::result::Result<u64, FileIssue> {
            let size = if path.ends_with("a.bmp") {
                10
            } else if path.ends_with("b.bmp") {
                20
            } else if path.ends_with("c.bmp") {
                30
            } else {
                40
            };

            Ok(size)
        }
    }

    #[derive(Default)]
    struct PassthroughFilter;

    impl OutputDirectoryFilter for PassthroughFilter {
        fn filter_existing_output(
            &self,
            entries: Vec<aba_avner::domain::ImageEntry>,
            _output_dir: &Path,
            _similarity_threshold: u32,
            _cancellation: &dyn aba_avner::application::CancellationToken,
        ) -> Result<OutputFilterResult> {
            Ok(OutputFilterResult {
                entries,
                warnings: WarningLog::default(),
            })
        }
    }

    let input_dir = PathBuf::from("/virtual-input");
    let output_dir = PathBuf::from("/virtual-output");
    let scanner = MockScanner {
        input_root: input_dir.clone(),
        input_paths: vec![
            input_dir.join("album-a").join("a.bmp"),
            input_dir.join("album-a").join("b.bmp"),
            input_dir.join("album-b").join("c.bmp"),
            input_dir.join("album-c").join("d.bmp"),
        ],
    };
    let writer = MockWriter;
    let hasher = MockHasher;
    let size_reader = MockSizeReader;
    let output_filter = PassthroughFilter;
    let mut publisher = NoopPublisher;

    let result = run_dedupe_with_filter(
        request(
            input_dir.display().to_string(),
            output_dir.display().to_string(),
            0,
            0,
        ),
        &scanner,
        &writer,
        &hasher,
        &size_reader,
        &output_filter,
        &mut publisher,
    )
    .unwrap();

    assert_eq!(result.report.storage_saved_bytes, 50);
    assert_eq!(result.report.summary.total_images, 4);
    assert_eq!(result.report.summary.unique_images, 2);
    assert_eq!(result.report.summary.duplicate_images, 2);
    assert_eq!(
        result.report.folder_breakdown,
        vec![
            aba_avner::application::FolderBreakdownEntry {
                path: "/virtual-input/album-b".to_string(),
                duplicates: 1,
                wasted_bytes: 30,
            },
            aba_avner::application::FolderBreakdownEntry {
                path: "/virtual-input/album-a".to_string(),
                duplicates: 1,
                wasted_bytes: 20,
            },
        ]
    );
}

#[test]
fn cancellation_before_start_stops_run_without_emitting_progress() {
    let temp = tempdir().unwrap();
    let input_dir = temp.path().join("input");
    let output_dir = temp.path().join("output");
    std::fs::create_dir_all(&input_dir).unwrap();
    create_pattern_image(&input_dir.join("a.bmp"), 64, 64, 0);

    struct RecordingPublisher {
        events: Vec<ProgressUpdate>,
    }

    impl ProgressPublisher for RecordingPublisher {
        fn emit(&mut self, update: ProgressUpdate) {
            self.events.push(update);
        }
    }

    let cancellation = AtomicCancellationToken::new();
    cancellation.cancel();
    let mut publisher = RecordingPublisher { events: Vec::new() };

    let error = run_dedupe_with_filesystem_and_cancellation(
        request(
            input_dir.display().to_string(),
            output_dir.display().to_string(),
            0,
            0,
        ),
        &mut publisher,
        &cancellation,
    )
    .unwrap_err();

    assert!(matches!(error, DedupeError::Cancelled));
    assert!(publisher.events.is_empty());
}

#[test]
fn cancellation_during_scan_stops_before_hashing_or_copying_results() {
    struct MockScanner {
        input_root: PathBuf,
        output_root: PathBuf,
        input_paths: Vec<PathBuf>,
        cancellation: AtomicCancellationToken,
    }

    impl ImageScanner for MockScanner {
        fn collect_image_paths(&self, root: &Path) -> Result<Vec<PathBuf>> {
            if root == self.output_root.as_path() {
                return Ok(Vec::new());
            }

            if root != self.input_root.as_path() {
                return Err(DedupeError::ReadDir(root.display().to_string()));
            }

            Ok(self.input_paths.clone())
        }

        fn collect_image_paths_with_cancellation(
            &self,
            root: &Path,
            cancellation: &dyn aba_avner::application::CancellationToken,
        ) -> Result<Vec<PathBuf>> {
            if root == self.output_root.as_path() {
                return Ok(Vec::new());
            }

            if root != self.input_root.as_path() {
                return Err(DedupeError::ReadDir(root.display().to_string()));
            }

            let mut paths = Vec::new();
            for path in &self.input_paths {
                if cancellation.is_cancelled() {
                    return Err(DedupeError::Cancelled);
                }
                paths.push(path.clone());
                self.cancellation.cancel();
            }

            Ok(paths)
        }
    }

    struct GuardWriter;

    impl OutputWriter for GuardWriter {
        fn ensure_output_dir(&self, _path: &Path) -> Result<()> {
            Ok(())
        }

        fn copy_unique_images<F>(
            &self,
            _paths: &[PathBuf],
            _output_dir: &Path,
            _cancellation: &dyn aba_avner::application::CancellationToken,
            _on_progress: F,
        ) -> Result<Vec<PathBuf>>
        where
            F: FnMut(usize),
        {
            panic!("copy_unique_images should not be reached after scan cancellation");
        }

        fn copy_non_unique_groups(
            &self,
            _groups: &[Vec<aba_avner::domain::ImageEntry>],
            _selected_paths: &[PathBuf],
            _output_dir: &Path,
            _locale: AppLocale,
            _export_similar_image_groups: bool,
            _cancellation: &dyn aba_avner::application::CancellationToken,
        ) -> Result<()> {
            panic!("copy_non_unique_groups should not be reached after scan cancellation");
        }
    }

    struct GuardHasher;

    impl ImageHasher for GuardHasher {
        fn load_and_hash(&self, _path: &Path) -> std::result::Result<u64, FileIssue> {
            panic!("hashing should not be reached after scan cancellation");
        }
    }

    struct MockSizeReader;

    impl FileSizeReader for MockSizeReader {
        fn file_size_bytes(&self, _path: &Path) -> std::result::Result<u64, FileIssue> {
            Ok(10_000)
        }
    }

    let input_dir = PathBuf::from("/virtual-input");
    let output_dir = PathBuf::from("/virtual-output");
    let cancellation = AtomicCancellationToken::new();
    let scanner = MockScanner {
        input_root: input_dir.clone(),
        output_root: output_dir.clone(),
        input_paths: vec![
            input_dir.join("first.bmp"),
            input_dir.join("second.bmp"),
            input_dir.join("third.bmp"),
        ],
        cancellation: cancellation.clone(),
    };
    struct ScanOnlyPublisher {
        events: Vec<ProgressUpdate>,
    }

    impl ProgressPublisher for ScanOnlyPublisher {
        fn emit(&mut self, update: ProgressUpdate) {
            self.events.push(update);
        }
    }

    let writer = GuardWriter;
    let hasher = GuardHasher;
    let size_reader = MockSizeReader;
    let mut publisher = ScanOnlyPublisher { events: Vec::new() };

    let error = run_dedupe_cancellable(
        request(
            input_dir.display().to_string(),
            output_dir.display().to_string(),
            0,
            0,
        ),
        &scanner,
        &writer,
        &hasher,
        &size_reader,
        &mut publisher,
        &cancellation,
    )
    .unwrap_err();

    assert!(matches!(error, DedupeError::Cancelled));
    assert!(publisher.events.iter().any(|event| event.stage == "scan"));
    assert!(!publisher.events.iter().any(|event| event.stage == "hash"));
    assert!(!publisher.events.iter().any(|event| event.stage == "copy"));
}

#[test]
fn cancellation_during_hashing_stops_before_copying_results() {
    struct MockScanner {
        input_root: PathBuf,
        output_root: PathBuf,
        input_paths: Vec<PathBuf>,
    }

    impl ImageScanner for MockScanner {
        fn collect_image_paths(&self, root: &Path) -> Result<Vec<PathBuf>> {
            if root == self.input_root.as_path() {
                return Ok(self.input_paths.clone());
            }

            if root == self.output_root.as_path() {
                return Ok(Vec::new());
            }

            Err(DedupeError::ReadDir(root.display().to_string()))
        }
    }

    struct GuardWriter;

    impl OutputWriter for GuardWriter {
        fn ensure_output_dir(&self, _path: &Path) -> Result<()> {
            Ok(())
        }

        fn copy_unique_images<F>(
            &self,
            _paths: &[PathBuf],
            _output_dir: &Path,
            _cancellation: &dyn aba_avner::application::CancellationToken,
            _on_progress: F,
        ) -> Result<Vec<PathBuf>>
        where
            F: FnMut(usize),
        {
            panic!("copy_unique_images should not be reached after cancellation");
        }

        fn copy_non_unique_groups(
            &self,
            _groups: &[Vec<aba_avner::domain::ImageEntry>],
            _selected_paths: &[PathBuf],
            _output_dir: &Path,
            _locale: AppLocale,
            _export_similar_image_groups: bool,
            _cancellation: &dyn aba_avner::application::CancellationToken,
        ) -> Result<()> {
            panic!("copy_non_unique_groups should not be reached after cancellation");
        }
    }

    struct MockHasher;

    impl ImageHasher for MockHasher {
        fn load_and_hash(&self, path: &Path) -> std::result::Result<u64, FileIssue> {
            Ok(path.to_string_lossy().len() as u64)
        }
    }

    struct MockSizeReader;

    impl FileSizeReader for MockSizeReader {
        fn file_size_bytes(&self, _path: &Path) -> std::result::Result<u64, FileIssue> {
            Ok(10_000)
        }
    }

    struct CancellingPublisher {
        cancellation: AtomicCancellationToken,
        events: Vec<ProgressUpdate>,
    }

    impl ProgressPublisher for CancellingPublisher {
        fn emit(&mut self, update: ProgressUpdate) {
            if update.stage == "hash" && update.current == 1 {
                self.cancellation.cancel();
            }
            self.events.push(update);
        }
    }

    let input_dir = PathBuf::from("/virtual-input");
    let output_dir = PathBuf::from("/virtual-output");
    let scanner = MockScanner {
        input_root: input_dir.clone(),
        output_root: output_dir.clone(),
        input_paths: vec![
            input_dir.join("first.bmp"),
            input_dir.join("second.bmp"),
            input_dir.join("third.bmp"),
        ],
    };
    let writer = GuardWriter;
    let hasher = MockHasher;
    let size_reader = MockSizeReader;
    let cancellation = AtomicCancellationToken::new();
    let mut publisher = CancellingPublisher {
        cancellation: cancellation.clone(),
        events: Vec::new(),
    };

    let error = run_dedupe_cancellable(
        request(
            input_dir.display().to_string(),
            output_dir.display().to_string(),
            0,
            0,
        ),
        &scanner,
        &writer,
        &hasher,
        &size_reader,
        &mut publisher,
        &cancellation,
    )
    .unwrap_err();

    assert!(matches!(error, DedupeError::Cancelled));
    assert!(publisher
        .events
        .iter()
        .any(|event| event.stage == "hash" && event.current == 1));
    assert!(!publisher.events.iter().any(|event| event.stage == "copy"));
}

#[test]
fn cancellation_during_grouping_stops_before_copying_results() {
    struct MockScanner {
        input_root: PathBuf,
        output_root: PathBuf,
        input_paths: Vec<PathBuf>,
    }

    impl ImageScanner for MockScanner {
        fn collect_image_paths(&self, root: &Path) -> Result<Vec<PathBuf>> {
            if root == self.input_root.as_path() {
                return Ok(self.input_paths.clone());
            }

            if root == self.output_root.as_path() {
                return Ok(Vec::new());
            }

            Err(DedupeError::ReadDir(root.display().to_string()))
        }
    }

    struct GuardWriter;

    impl OutputWriter for GuardWriter {
        fn ensure_output_dir(&self, _path: &Path) -> Result<()> {
            Ok(())
        }

        fn copy_unique_images<F>(
            &self,
            _paths: &[PathBuf],
            _output_dir: &Path,
            _cancellation: &dyn aba_avner::application::CancellationToken,
            _on_progress: F,
        ) -> Result<Vec<PathBuf>>
        where
            F: FnMut(usize),
        {
            panic!("copy_unique_images should not be reached after grouping cancellation");
        }

        fn copy_non_unique_groups(
            &self,
            _groups: &[Vec<aba_avner::domain::ImageEntry>],
            _selected_paths: &[PathBuf],
            _output_dir: &Path,
            _locale: AppLocale,
            _export_similar_image_groups: bool,
            _cancellation: &dyn aba_avner::application::CancellationToken,
        ) -> Result<()> {
            panic!("copy_non_unique_groups should not be reached after grouping cancellation");
        }
    }

    struct MockHasher;

    impl ImageHasher for MockHasher {
        fn load_and_hash(&self, path: &Path) -> std::result::Result<u64, FileIssue> {
            Ok(path.to_string_lossy().len() as u64)
        }
    }

    struct MockSizeReader;

    impl FileSizeReader for MockSizeReader {
        fn file_size_bytes(&self, _path: &Path) -> std::result::Result<u64, FileIssue> {
            Ok(10_000)
        }
    }

    struct CancellingPublisher {
        cancellation: AtomicCancellationToken,
        events: Vec<ProgressUpdate>,
    }

    impl ProgressPublisher for CancellingPublisher {
        fn emit(&mut self, update: ProgressUpdate) {
            if update.stage == "group" && update.current == 1 {
                self.cancellation.cancel();
            }
            self.events.push(update);
        }
    }

    let input_dir = PathBuf::from("/virtual-input");
    let output_dir = PathBuf::from("/virtual-output");
    let scanner = MockScanner {
        input_root: input_dir.clone(),
        output_root: output_dir.clone(),
        input_paths: vec![
            input_dir.join("first.bmp"),
            input_dir.join("second.bmp"),
            input_dir.join("third.bmp"),
        ],
    };
    let writer = GuardWriter;
    let hasher = MockHasher;
    let size_reader = MockSizeReader;
    let cancellation = AtomicCancellationToken::new();
    let mut publisher = CancellingPublisher {
        cancellation: cancellation.clone(),
        events: Vec::new(),
    };

    let error = run_dedupe_cancellable(
        request(
            input_dir.display().to_string(),
            output_dir.display().to_string(),
            0,
            0,
        ),
        &scanner,
        &writer,
        &hasher,
        &size_reader,
        &mut publisher,
        &cancellation,
    )
    .unwrap_err();

    assert!(matches!(error, DedupeError::Cancelled));
    assert!(publisher
        .events
        .iter()
        .any(|event| event.stage == "group" && event.current == 1));
    assert!(!publisher
        .events
        .iter()
        .any(|event| event.stage == "group" && event.current > 1));
    assert!(!publisher
        .events
        .iter()
        .any(|event| event.stage == "group" && event.message == "Grouping complete"));
    assert!(!publisher.events.iter().any(|event| event.stage == "copy"));
}

#[test]
fn cancellation_during_existing_output_filtering_stops_before_copying_results() {
    struct MockScanner {
        input_root: PathBuf,
        output_root: PathBuf,
        input_paths: Vec<PathBuf>,
        output_paths: Vec<PathBuf>,
    }

    impl ImageScanner for MockScanner {
        fn collect_image_paths(&self, root: &Path) -> Result<Vec<PathBuf>> {
            if root == self.input_root.as_path() {
                return Ok(self.input_paths.clone());
            }

            if root == self.output_root.as_path() {
                return Ok(self.output_paths.clone());
            }

            Err(DedupeError::ReadDir(root.display().to_string()))
        }
    }

    struct GuardWriter;

    impl OutputWriter for GuardWriter {
        fn ensure_output_dir(&self, _path: &Path) -> Result<()> {
            Ok(())
        }

        fn copy_unique_images<F>(
            &self,
            _paths: &[PathBuf],
            _output_dir: &Path,
            _cancellation: &dyn aba_avner::application::CancellationToken,
            _on_progress: F,
        ) -> Result<Vec<PathBuf>>
        where
            F: FnMut(usize),
        {
            panic!("copy_unique_images should not be reached after output-filter cancellation");
        }

        fn copy_non_unique_groups(
            &self,
            _groups: &[Vec<aba_avner::domain::ImageEntry>],
            _selected_paths: &[PathBuf],
            _output_dir: &Path,
            _locale: AppLocale,
            _export_similar_image_groups: bool,
            _cancellation: &dyn aba_avner::application::CancellationToken,
        ) -> Result<()> {
            panic!("copy_non_unique_groups should not be reached after output-filter cancellation");
        }
    }

    struct CancellingHasher {
        cancellation: AtomicCancellationToken,
        output_hash_calls: Arc<AtomicUsize>,
    }

    impl ImageHasher for CancellingHasher {
        fn load_and_hash(&self, path: &Path) -> std::result::Result<u64, FileIssue> {
            if path.starts_with("/virtual-output") {
                let call_index = self.output_hash_calls.fetch_add(1, Ordering::SeqCst);
                if call_index == 0 {
                    self.cancellation.cancel();
                }
            }

            Ok(path.to_string_lossy().len() as u64)
        }
    }

    struct MockSizeReader;

    impl FileSizeReader for MockSizeReader {
        fn file_size_bytes(&self, _path: &Path) -> std::result::Result<u64, FileIssue> {
            Ok(10_000)
        }
    }

    let input_dir = PathBuf::from("/virtual-input");
    let output_dir = PathBuf::from("/virtual-output");
    let scanner = MockScanner {
        input_root: input_dir.clone(),
        output_root: output_dir.clone(),
        input_paths: vec![input_dir.join("first.bmp"), input_dir.join("second.bmp")],
        output_paths: vec![
            output_dir.join("existing-a.bmp"),
            output_dir.join("existing-b.bmp"),
            output_dir.join("existing-c.bmp"),
        ],
    };
    let writer = GuardWriter;
    let cancellation = AtomicCancellationToken::new();
    let output_hash_calls = Arc::new(AtomicUsize::new(0));
    let hasher = CancellingHasher {
        cancellation: cancellation.clone(),
        output_hash_calls: output_hash_calls.clone(),
    };
    let size_reader = MockSizeReader;
    let mut publisher = NoopPublisher;

    let error = run_dedupe_cancellable(
        request(
            input_dir.display().to_string(),
            output_dir.display().to_string(),
            0,
            0,
        ),
        &scanner,
        &writer,
        &hasher,
        &size_reader,
        &mut publisher,
        &cancellation,
    )
    .unwrap_err();

    assert!(matches!(error, DedupeError::Cancelled));
    assert_eq!(output_hash_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn end_to_end_run_reports_counts_copies_files_and_collects_warnings() {
    let temp = tempdir().unwrap();
    let input_dir = temp.path().join("input");
    let output_dir = temp.path().join("output");
    std::fs::create_dir_all(&input_dir).unwrap();
    std::fs::create_dir_all(&output_dir).unwrap();

    let duplicate_a = input_dir.join("duplicate-a.bmp");
    let duplicate_b = input_dir.join("duplicate-b.bmp");
    let already_in_output = input_dir.join("already-in-output.bmp");
    let new_unique = input_dir.join("new-unique.bmp");
    let tiny = input_dir.join("tiny.bmp");
    let corrupt = input_dir.join("corrupt.png");

    create_pattern_image(&duplicate_a, 64, 64, 0);
    std::fs::copy(&duplicate_a, &duplicate_b).unwrap();
    create_pattern_image(&already_in_output, 64, 64, 1);
    create_pattern_image(&new_unique, 64, 64, 2);
    create_pattern_image(&tiny, 8, 8, 0);
    write(&corrupt, vec![b'x'; 2_048]).unwrap();
    std::fs::copy(&already_in_output, output_dir.join("existing.bmp")).unwrap();

    let duplicate_hash = compute_dhash(&image::open(&duplicate_a).unwrap());
    let existing_hash = compute_dhash(&image::open(&already_in_output).unwrap());
    let unique_hash = compute_dhash(&image::open(&new_unique).unwrap());
    assert_ne!(duplicate_hash, existing_hash);
    assert_ne!(duplicate_hash, unique_hash);
    assert_ne!(existing_hash, unique_hash);

    let min_image_size_bytes = std::fs::metadata(&tiny).unwrap().len() + 1;
    assert!(std::fs::metadata(&duplicate_a).unwrap().len() >= min_image_size_bytes);

    let mut publisher = NoopPublisher;
    let result = run_dedupe_with_filesystem(
        request(
            input_dir.display().to_string(),
            output_dir.display().to_string(),
            0,
            min_image_size_bytes,
        ),
        &mut publisher,
    )
    .unwrap();

    assert_eq!(result.total_images, 4);
    assert_eq!(result.unique_images, 2);
    assert_eq!(result.duplicate_images, 2);
    assert_eq!(
        result.total_images,
        result.unique_images + result.duplicate_images
    );

    let copied_paths = collect_image_paths(&output_dir).unwrap();
    let top_level_paths: Vec<_> = copied_paths
        .iter()
        .filter(|path| path.parent() == Some(output_dir.as_path()))
        .collect();
    assert_eq!(top_level_paths.len(), 3);
    assert!(top_level_paths
        .iter()
        .any(|path| path.ends_with("existing.bmp")));
    assert!(top_level_paths
        .iter()
        .any(|path| path.ends_with("duplicate-a.bmp")));
    assert!(top_level_paths
        .iter()
        .any(|path| path.ends_with("new-unique.bmp")));
    assert!(!copied_paths.iter().any(|path| path.ends_with("tiny.bmp")));
    assert!(!copied_paths
        .iter()
        .any(|path| path.ends_with("already-in-output.bmp")));
    assert!(copied_paths.iter().any(|path| {
        path.starts_with(output_dir.join("similar image groups"))
            && path.ends_with("duplicate-a__kept.bmp")
    }));
    assert!(copied_paths.iter().any(|path| {
        path.starts_with(output_dir.join("similar image groups"))
            && path.ends_with("duplicate-b.bmp")
    }));

    assert!(result.warnings.iter().any(|warning| {
        warning.code == RunWarning::SIMILAR_IMAGE_IN_OUTPUT_CODE
            && warning.path.ends_with("already-in-output.bmp")
    }));
    assert!(result.warnings.iter().any(|warning| {
        warning.code == RunWarning::FILE_ISSUE_CODE && warning.path.ends_with("corrupt.png")
    }));
}

#[test]
fn end_to_end_run_emits_expected_progress_stages() {
    let temp = tempdir().unwrap();
    let input_dir = temp.path().join("input");
    let output_dir = temp.path().join("output");
    std::fs::create_dir_all(&input_dir).unwrap();

    create_pattern_image(&input_dir.join("a.bmp"), 64, 64, 0);
    create_pattern_image(&input_dir.join("b.bmp"), 64, 64, 1);

    struct RecordingPublisher {
        events: Vec<ProgressUpdate>,
    }

    impl ProgressPublisher for RecordingPublisher {
        fn emit(&mut self, update: ProgressUpdate) {
            self.events.push(update);
        }
    }

    let mut publisher = RecordingPublisher { events: Vec::new() };
    let result = run_dedupe_with_filesystem(
        request(
            input_dir.display().to_string(),
            output_dir.display().to_string(),
            0,
            0,
        ),
        &mut publisher,
    )
    .unwrap();
    let events = publisher.events;

    assert_eq!(result.total_images, 2);
    assert!(!events.is_empty());
    assert_eq!(events.first().unwrap().stage, "scan");
    assert_eq!(events.first().unwrap().current, 0);
    assert_eq!(events.last().unwrap().stage, "copy");
    assert_eq!(events.last().unwrap().message, "Copy complete");
    assert!(events
        .iter()
        .any(|event| event.stage == "hash" && event.current == 2));
    assert!(events
        .iter()
        .any(|event| event.stage == "group" && event.message == "Grouping complete"));
    assert!(events.iter().any(|event| event.stage == "copy"
        && event.message == "Copying unique images"
        && event.current == 1));
    assert!(events
        .iter()
        .any(|event| event.stage == "copy" && event.current == result.unique_images));
}

#[test]
fn grouping_stage_emits_incremental_progress_against_stable_total() {
    let temp = tempdir().unwrap();
    let input_dir = temp.path().join("input");
    let output_dir = temp.path().join("output");
    std::fs::create_dir_all(&input_dir).unwrap();

    create_pattern_image(&input_dir.join("a.bmp"), 64, 64, 0);
    create_pattern_image(&input_dir.join("b.bmp"), 64, 64, 1);
    create_pattern_image(&input_dir.join("c.bmp"), 64, 64, 2);

    struct RecordingPublisher {
        events: Vec<ProgressUpdate>,
    }

    impl ProgressPublisher for RecordingPublisher {
        fn emit(&mut self, update: ProgressUpdate) {
            self.events.push(update);
        }
    }

    let mut publisher = RecordingPublisher { events: Vec::new() };
    run_dedupe_with_filesystem(
        request(
            input_dir.display().to_string(),
            output_dir.display().to_string(),
            0,
            0,
        ),
        &mut publisher,
    )
    .unwrap();

    let group_events: Vec<_> = publisher
        .events
        .into_iter()
        .filter(|event| event.stage == "group")
        .collect();

    assert!(group_events.len() >= 3);
    assert_eq!(group_events.first().unwrap().current, 0);
    assert_eq!(group_events.first().unwrap().total, 3);
    assert!(group_events
        .iter()
        .any(|event| event.message == "Grouping similar images" && event.current == 1));
    assert!(group_events
        .iter()
        .any(|event| event.message == "Grouping similar images" && event.current == 2));
    assert_eq!(group_events.last().unwrap().message, "Grouping complete");
    assert_eq!(group_events.last().unwrap().current, 3);
    assert_eq!(group_events.last().unwrap().total, 3);
}

#[test]
fn copy_stage_reports_completed_progress_when_all_candidates_are_filtered_out() {
    struct MockScanner {
        input_root: PathBuf,
        input_paths: Vec<PathBuf>,
    }

    impl ImageScanner for MockScanner {
        fn collect_image_paths(&self, root: &Path) -> Result<Vec<PathBuf>> {
            if root == self.input_root.as_path() {
                return Ok(self.input_paths.clone());
            }

            Err(DedupeError::ReadDir(root.display().to_string()))
        }
    }

    struct MockWriter;

    impl OutputWriter for MockWriter {
        fn ensure_output_dir(&self, _path: &Path) -> Result<()> {
            Ok(())
        }

        fn copy_unique_images<F>(
            &self,
            paths: &[PathBuf],
            _output_dir: &Path,
            _cancellation: &dyn aba_avner::application::CancellationToken,
            _on_progress: F,
        ) -> Result<Vec<PathBuf>>
        where
            F: FnMut(usize),
        {
            Ok(paths.to_vec())
        }

        fn copy_non_unique_groups(
            &self,
            _groups: &[Vec<aba_avner::domain::ImageEntry>],
            _selected_paths: &[PathBuf],
            _output_dir: &Path,
            _locale: AppLocale,
            _export_similar_image_groups: bool,
            _cancellation: &dyn aba_avner::application::CancellationToken,
        ) -> Result<()> {
            Ok(())
        }
    }

    struct MockHasher;

    impl ImageHasher for MockHasher {
        fn load_and_hash(&self, path: &Path) -> std::result::Result<u64, FileIssue> {
            let hash = if path.to_string_lossy().contains("duplicate") {
                11
            } else {
                99
            };

            Ok(hash)
        }
    }

    struct MockSizeReader;

    impl FileSizeReader for MockSizeReader {
        fn file_size_bytes(&self, _path: &Path) -> std::result::Result<u64, FileIssue> {
            Ok(10_000)
        }
    }

    struct MockOutputFilter;

    impl OutputDirectoryFilter for MockOutputFilter {
        fn filter_existing_output(
            &self,
            entries: Vec<aba_avner::domain::ImageEntry>,
            _output_dir: &Path,
            _similarity_threshold: u32,
            _cancellation: &dyn aba_avner::application::CancellationToken,
        ) -> Result<OutputFilterResult> {
            Ok(OutputFilterResult {
                entries: Vec::new(),
                warnings: WarningLog::from(vec![RunWarning::similar_image_in_output(
                    entries[0].path.clone(),
                )]),
            })
        }
    }

    struct RecordingPublisher {
        events: Vec<ProgressUpdate>,
    }

    impl ProgressPublisher for RecordingPublisher {
        fn emit(&mut self, update: ProgressUpdate) {
            self.events.push(update);
        }
    }

    let input_dir = PathBuf::from("/virtual-input");
    let output_dir = PathBuf::from("/virtual-output");
    let duplicate_a = input_dir.join("duplicate-a.bmp");
    let duplicate_b = input_dir.join("duplicate-b.bmp");

    let scanner = MockScanner {
        input_root: input_dir.clone(),
        input_paths: vec![duplicate_a, duplicate_b],
    };
    let writer = MockWriter;
    let hasher = MockHasher;
    let size_reader = MockSizeReader;
    let output_filter = MockOutputFilter;
    let mut publisher = RecordingPublisher { events: Vec::new() };

    let result = run_dedupe_with_filter(
        request(
            input_dir.display().to_string(),
            output_dir.display().to_string(),
            0,
            0,
        ),
        &scanner,
        &writer,
        &hasher,
        &size_reader,
        &output_filter,
        &mut publisher,
    )
    .unwrap();

    assert_eq!(result.unique_images, 0);

    let copy_complete = publisher
        .events
        .iter()
        .rev()
        .find(|event| event.stage == "copy" && event.message == "Copy complete")
        .unwrap();

    assert_eq!(copy_complete.current, 1);
    assert_eq!(copy_complete.total, 1);
}

#[test]
fn golden_dataset_end_to_end_matches_expected_counts() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("auto-tests")
        .join("golden");
    let input_dir = root.join("input");
    let output_seed_dir = root.join("output_seed");
    let manifest = load_golden_manifest(&root);

    let temp = tempdir().unwrap();
    let output_dir = temp.path().join("output");
    std::fs::create_dir_all(&output_dir).unwrap();
    for path in collect_image_paths(&output_seed_dir).unwrap() {
        let filename = path.file_name().unwrap();
        std::fs::copy(&path, output_dir.join(filename)).unwrap();
    }

    let mut publisher = NoopPublisher;
    let result = run_dedupe_with_filesystem(
        request(
            input_dir.display().to_string(),
            output_dir.display().to_string(),
            manifest.similarity_threshold,
            0,
        ),
        &mut publisher,
    )
    .unwrap();

    assert_eq!(result.total_images, manifest.expected_hashed_images);
    assert_eq!(
        result.unique_images,
        manifest.expected_unique_after_existing_filter
    );
    assert_eq!(result.duplicate_images, manifest.expected_duplicate_images);
    assert_eq!(
        result.total_images,
        result.unique_images + result.duplicate_images
    );

    for expected in &manifest.expected_warning_files {
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.path.ends_with(expected)));
    }

    let copied_paths = collect_image_paths(&output_dir).unwrap();
    let top_level_paths: Vec<_> = copied_paths
        .iter()
        .filter(|path| path.parent() == Some(output_dir.as_path()))
        .collect();
    assert_eq!(
        top_level_paths.len(),
        manifest.expected_unique_after_existing_filter + 1
    );
    assert!(top_level_paths
        .iter()
        .any(|path| path.ends_with("existing_seed.jpg")));
    for prefix in &manifest.expected_output_prefixes {
        assert!(top_level_paths.iter().any(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with(prefix))
                .unwrap_or(false)
        }));
    }
    assert!(copied_paths
        .iter()
        .any(|path| path.starts_with(output_dir.join("similar image groups"))));
}

#[test]
fn invalid_output_path_returns_typed_error() {
    let temp = tempdir().unwrap();
    let input_dir = temp.path().join("input");
    let output_file = temp.path().join("output-file");

    std::fs::create_dir_all(&input_dir).unwrap();
    write(&output_file, b"not a directory").unwrap();

    let mut publisher = NoopPublisher;
    let error = run_dedupe_with_filesystem(
        request(
            input_dir.display().to_string(),
            output_file.display().to_string(),
            0,
            0,
        ),
        &mut publisher,
    )
    .unwrap_err();

    match error {
        DedupeError::InvalidOutputDir(message) => {
            assert!(message.contains("output-file"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn invalid_input_path_returns_typed_error() {
    let temp = tempdir().unwrap();
    let missing_input = temp.path().join("missing-input");
    let output_dir = temp.path().join("output");

    let mut publisher = NoopPublisher;
    let error = run_dedupe_with_filesystem(
        request(
            missing_input.display().to_string(),
            output_dir.display().to_string(),
            0,
            0,
        ),
        &mut publisher,
    )
    .unwrap_err();

    match error {
        DedupeError::InvalidInputDir(message) => {
            assert!(message.contains("missing-input"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

fn create_pattern_image(path: &Path, width: u32, height: u32, variant: u8) {
    let image: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_fn(width, height, |x, _y| {
        let value = match variant {
            0 => {
                if x < width / 2 {
                    240
                } else {
                    15
                }
            }
            1 => {
                if x < width / 2 {
                    15
                } else {
                    240
                }
            }
            _ => {
                if x % 2 == 0 {
                    220
                } else {
                    20
                }
            }
        };
        Rgba([value, value / 2, 255u8.saturating_sub(value), 255])
    });
    DynamicImage::ImageRgba8(image).save(path).unwrap();
}

fn request(
    input_dir: String,
    output_dir: String,
    similarity_threshold: u32,
    min_image_size_bytes: u64,
) -> RunDedupeRequest {
    RunDedupeRequest::try_from(DedupeConfig {
        input_dir,
        output_dir,
        similarity_threshold,
        min_image_size_bytes,
        filter_existing_output: true,
        run_id: String::new(),
        locale: "en".to_string(),
    })
    .unwrap()
}

#[derive(Debug, Deserialize)]
struct GoldenManifest {
    similarity_threshold: u32,
    expected_hashed_images: usize,
    expected_unique_after_existing_filter: usize,
    expected_duplicate_images: usize,
    expected_output_prefixes: Vec<String>,
    expected_warning_files: Vec<String>,
}

fn load_golden_manifest(root: &Path) -> GoldenManifest {
    let raw = std::fs::read_to_string(root.join("manifest.json")).unwrap();
    serde_json::from_str(&raw).unwrap()
}
