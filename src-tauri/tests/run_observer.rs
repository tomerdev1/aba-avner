use aba_avner::application::{
    AppLocale, CancellationToken, FileIssue, FileSizeReader, ImageHasher, ImageScanner,
    OutputDirectoryFilter, OutputFilterResult, OutputWriter, ProgressPublisher, ProgressUpdate,
    RunDedupeRequest, WarningLog,
};
use aba_avner::errors::{DedupeError, Result};
use aba_avner::infrastructure::run_dedupe_with_observer;
use aba_avner::presentation::contracts::DedupeConfig;
use std::path::{Path, PathBuf};

#[test]
fn observer_records_start_and_success_for_completed_run() {
    let input_dir = PathBuf::from("/virtual-input");
    let output_dir = PathBuf::from("/virtual-output");
    let scanner = MockScanner {
        input_root: input_dir.clone(),
        input_paths: vec![input_dir.join("one.bmp"), input_dir.join("two.bmp")],
    };
    let writer = MockWriter {
        fail_on_copy: false,
    };
    let hasher = MockHasher;
    let size_reader = MockSizeReader;
    let output_filter = PassthroughOutputFilter::default();
    let cancellation = NeverCancelled;
    let mut progress = NoopPublisher;
    let mut observer = RecordingObserver::default();

    let result = run_dedupe_with_observer(
        request(input_dir.clone(), output_dir.clone()),
        &scanner,
        &writer,
        &hasher,
        &size_reader,
        &output_filter,
        &mut progress,
        &cancellation,
        &mut observer,
    )
    .unwrap();

    assert_eq!(result.total_images, 2);
    assert_eq!(observer.started, 1);
    assert_eq!(observer.succeeded, 1);
    assert_eq!(observer.failed.len(), 0);
    assert_eq!(observer.last_output_dir, Some(output_dir));
}

#[test]
fn observer_records_cancelled_run_as_failure() {
    let input_dir = PathBuf::from("/virtual-input");
    let output_dir = PathBuf::from("/virtual-output");
    let scanner = MockScanner {
        input_root: input_dir.clone(),
        input_paths: vec![input_dir.join("one.bmp")],
    };
    let writer = MockWriter {
        fail_on_copy: false,
    };
    let hasher = MockHasher;
    let size_reader = MockSizeReader;
    let output_filter = PassthroughOutputFilter::default();
    let cancellation = AlwaysCancelled;
    let mut progress = NoopPublisher;
    let mut observer = RecordingObserver::default();

    let error = run_dedupe_with_observer(
        request(input_dir, output_dir),
        &scanner,
        &writer,
        &hasher,
        &size_reader,
        &output_filter,
        &mut progress,
        &cancellation,
        &mut observer,
    )
    .unwrap_err();

    assert!(matches!(error, DedupeError::Cancelled));
    assert_eq!(observer.started, 1);
    assert_eq!(observer.succeeded, 0);
    assert_eq!(observer.failed, vec!["Cancelled".to_string()]);
}

#[test]
fn observer_records_copy_failures() {
    let input_dir = PathBuf::from("/virtual-input");
    let output_dir = PathBuf::from("/virtual-output");
    let scanner = MockScanner {
        input_root: input_dir.clone(),
        input_paths: vec![input_dir.join("one.bmp")],
    };
    let writer = MockWriter { fail_on_copy: true };
    let hasher = MockHasher;
    let size_reader = MockSizeReader;
    let output_filter = PassthroughOutputFilter::default();
    let cancellation = NeverCancelled;
    let mut progress = NoopPublisher;
    let mut observer = RecordingObserver::default();

    let error = run_dedupe_with_observer(
        request(input_dir, output_dir),
        &scanner,
        &writer,
        &hasher,
        &size_reader,
        &output_filter,
        &mut progress,
        &cancellation,
        &mut observer,
    )
    .unwrap_err();

    match error {
        DedupeError::CopyImage(message) => assert!(message.contains("simulated copy failure")),
        other => panic!("unexpected error: {other:?}"),
    }
    assert_eq!(observer.started, 1);
    assert_eq!(observer.succeeded, 0);
    assert_eq!(observer.failed.len(), 1);
    assert!(observer.failed[0].contains("CopyImage"));
}

#[derive(Default)]
struct RecordingObserver {
    started: usize,
    succeeded: usize,
    failed: Vec<String>,
    last_output_dir: Option<PathBuf>,
}

impl aba_avner::infrastructure::RunObserver for RecordingObserver {
    fn on_started(&mut self, _request: &RunDedupeRequest) {
        self.started += 1;
    }

    fn on_succeeded(
        &mut self,
        _request: &RunDedupeRequest,
        result: &aba_avner::application::RunDedupeResult,
    ) {
        self.succeeded += 1;
        self.last_output_dir = Some(result.output_dir.clone());
    }

    fn on_failed(&mut self, _request: &RunDedupeRequest, error: &DedupeError) {
        self.failed.push(format!("{error:?}"));
    }
}

struct NoopPublisher;

impl ProgressPublisher for NoopPublisher {
    fn emit(&mut self, _update: ProgressUpdate) {}
}

struct NeverCancelled;

impl CancellationToken for NeverCancelled {
    fn is_cancelled(&self) -> bool {
        false
    }
}

struct AlwaysCancelled;

impl CancellationToken for AlwaysCancelled {
    fn is_cancelled(&self) -> bool {
        true
    }
}

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

struct MockWriter {
    fail_on_copy: bool,
}

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
        if self.fail_on_copy {
            return Err(DedupeError::CopyImage("simulated copy failure".to_string()));
        }

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
struct PassthroughOutputFilter;

impl OutputDirectoryFilter for PassthroughOutputFilter {
    fn filter_existing_output(
        &self,
        entries: Vec<aba_avner::domain::ImageEntry>,
        _output_dir: &Path,
        _similarity_threshold: u32,
        cancellation: &dyn CancellationToken,
    ) -> Result<OutputFilterResult> {
        if cancellation.is_cancelled() {
            return Err(DedupeError::Cancelled);
        }

        Ok(OutputFilterResult {
            entries,
            warnings: WarningLog::default(),
        })
    }
}

fn request(input_dir: PathBuf, output_dir: PathBuf) -> RunDedupeRequest {
    RunDedupeRequest::try_from(DedupeConfig {
        input_dir: input_dir.display().to_string(),
        output_dir: output_dir.display().to_string(),
        similarity_threshold: 0,
        min_image_size_bytes: 0,
        filter_existing_output: true,
        run_id: String::new(),
        locale: "en".to_string(),
    })
    .unwrap()
}
