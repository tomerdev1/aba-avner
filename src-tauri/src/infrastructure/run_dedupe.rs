use crate::application::{
    export_review_selections_with_filter, prepare_review_cancellable_with_filter,
    run_dedupe_cancellable_with_filter, CancellationToken, ExportSelectionsRequest,
    NoopCancellationToken, PrepareReviewResult, ProgressPublisher, RunDedupeRequest,
    RunDedupeResult,
};
use crate::errors::{DedupeError, Result};
use std::path::Path;

use super::{
    next_run_id, normalize_input_dir, normalize_output_dir, FsFileSizeReader, FsImageHasher,
    FsImageScanner, FsOutputDirectoryFilter, FsOutputWriter, LogRunObserver,
};

pub trait RunObserver {
    fn on_started(&mut self, request: &RunDedupeRequest);
    fn on_succeeded(&mut self, request: &RunDedupeRequest, result: &RunDedupeResult);
    fn on_failed(&mut self, request: &RunDedupeRequest, error: &DedupeError);
}

pub fn run_dedupe_with_filesystem<P>(
    request: RunDedupeRequest,
    progress: &mut P,
) -> Result<RunDedupeResult>
where
    P: ProgressPublisher,
{
    let cancellation = NoopCancellationToken;
    run_dedupe_with_filesystem_and_cancellation(request, progress, &cancellation)
}

pub fn run_dedupe_with_filesystem_and_cancellation<P, C>(
    request: RunDedupeRequest,
    progress: &mut P,
    cancellation: &C,
) -> Result<RunDedupeResult>
where
    P: ProgressPublisher,
    C: CancellationToken,
{
    run_dedupe_with_filesystem_and_cancellation_with_run_id(
        request,
        progress,
        cancellation,
        next_run_id(),
    )
}

pub fn run_dedupe_with_filesystem_and_cancellation_with_run_id<P, C>(
    request: RunDedupeRequest,
    progress: &mut P,
    cancellation: &C,
    run_id: String,
) -> Result<RunDedupeResult>
where
    P: ProgressPublisher,
    C: CancellationToken,
{
    let scanner = FsImageScanner;
    let writer = FsOutputWriter;
    let hasher = FsImageHasher;
    let size_reader = FsFileSizeReader;
    let output_filter = FsOutputDirectoryFilter;
    let mut observer = LogRunObserver::with_run_id(run_id);
    let request = normalize_and_validate_filesystem_request(request.clone()).map_err(|error| {
        observer.on_failed(&request, &error);
        error
    })?;

    run_dedupe_with_observer(
        request,
        &scanner,
        &writer,
        &hasher,
        &size_reader,
        &output_filter,
        progress,
        cancellation,
        &mut observer,
    )
}

pub fn prepare_review_with_filesystem_and_cancellation_with_run_id<P, C>(
    request: RunDedupeRequest,
    progress: &mut P,
    cancellation: &C,
    run_id: String,
) -> Result<PrepareReviewResult>
where
    P: ProgressPublisher,
    C: CancellationToken,
{
    let scanner = FsImageScanner;
    let writer = FsOutputWriter;
    let hasher = FsImageHasher;
    let size_reader = FsFileSizeReader;
    let output_filter = FsOutputDirectoryFilter;
    let mut observer = LogRunObserver::with_run_id(run_id);
    let request = normalize_and_validate_filesystem_request(request.clone()).map_err(|error| {
        observer.on_failed(&request, &error);
        error
    })?;

    observer.on_started(&request);
    match prepare_review_cancellable_with_filter(
        request.clone(),
        &scanner,
        &writer,
        &hasher,
        &size_reader,
        progress,
        &output_filter,
        cancellation,
    ) {
        Ok(result) => Ok(result),
        Err(error) => {
            observer.on_failed(&request, &error);
            Err(error)
        }
    }
}

pub fn export_review_with_filesystem_and_cancellation_with_run_id<P, C>(
    request: ExportSelectionsRequest,
    progress: &mut P,
    cancellation: &C,
    run_id: String,
) -> Result<RunDedupeResult>
where
    P: ProgressPublisher,
    C: CancellationToken,
{
    let writer = FsOutputWriter;
    let hasher = FsImageHasher;
    let size_reader = FsFileSizeReader;
    let output_filter = FsOutputDirectoryFilter;
    let mut observer = LogRunObserver::with_run_id(run_id);
    let request = normalize_and_validate_export_request(request)?;

    observer.on_started(&RunDedupeRequest {
        input_dir: std::path::PathBuf::from("<reviewed-groups>"),
        output_dir: request.output_dir.clone(),
        similarity_threshold: request.similarity_threshold,
        min_image_size_bytes: 0,
        filter_existing_output: request.filter_existing_output,
        locale: request.locale,
    });
    match export_review_selections_with_filter(
        request.clone(),
        &writer,
        &hasher,
        &size_reader,
        &output_filter,
        progress,
        cancellation,
    ) {
        Ok(result) => Ok(result),
        Err(error) => {
            observer.on_failed(
                &RunDedupeRequest {
                    input_dir: std::path::PathBuf::from("<reviewed-groups>"),
                    output_dir: request.output_dir.clone(),
                    similarity_threshold: request.similarity_threshold,
                    min_image_size_bytes: 0,
                    filter_existing_output: request.filter_existing_output,
                    locale: request.locale,
                },
                &error,
            );
            Err(error)
        }
    }
}

fn normalize_and_validate_filesystem_request(
    request: RunDedupeRequest,
) -> Result<RunDedupeRequest> {
    let input_dir = normalize_input_dir(&request.input_dir)?;
    let output_dir = normalize_output_dir(&request.output_dir)?;
    ensure_non_conflicting_paths(&input_dir, &output_dir)?;

    Ok(RunDedupeRequest {
        input_dir,
        output_dir,
        ..request
    })
}

fn normalize_and_validate_export_request(
    request: ExportSelectionsRequest,
) -> Result<ExportSelectionsRequest> {
    let output_dir = normalize_output_dir(&request.output_dir)?;

    Ok(ExportSelectionsRequest {
        output_dir,
        ..request
    })
}

fn ensure_non_conflicting_paths(input_dir: &Path, output_dir: &Path) -> Result<()> {
    if input_dir == output_dir {
        return Err(DedupeError::InputOutputConflict(format!(
            "{} and {} resolve to the same directory",
            input_dir.display(),
            output_dir.display()
        )));
    }

    if output_dir.starts_with(input_dir) {
        return Err(DedupeError::InputOutputConflict(format!(
            "{} is inside {}",
            output_dir.display(),
            input_dir.display()
        )));
    }

    if input_dir.starts_with(output_dir) {
        return Err(DedupeError::InputOutputConflict(format!(
            "{} is inside {}",
            input_dir.display(),
            output_dir.display()
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        ensure_non_conflicting_paths, run_dedupe_with_filesystem_and_cancellation_with_run_id,
    };
    use crate::application::{ProgressPublisher, ProgressUpdate, RunDedupeRequest};
    use crate::errors::DedupeError;
    use crate::infrastructure::normalize_input_dir;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn existing_input_dir_requires_directory() {
        let temp = tempdir().unwrap();
        let file = temp.path().join("file.txt");
        fs::write(&file, b"data").unwrap();

        let error = normalize_input_dir(&file).unwrap_err();

        match error {
            DedupeError::InvalidInputDir(path) => assert!(path.contains("file.txt")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn conflicting_paths_reject_same_directory() {
        let error =
            ensure_non_conflicting_paths(Path::new("/input"), Path::new("/input")).unwrap_err();

        match error {
            DedupeError::InputOutputConflict(detail) => {
                assert!(detail.contains("same directory"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn conflicting_paths_reject_nested_output_dir() {
        let error = ensure_non_conflicting_paths(Path::new("/input"), Path::new("/input/output"))
            .unwrap_err();

        match error {
            DedupeError::InputOutputConflict(detail) => {
                assert!(detail.contains("/input/output"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn run_normalization_rejects_root_output_dir() {
        let temp = tempdir().unwrap();
        let input_dir = temp.path().join("input");
        fs::create_dir_all(&input_dir).unwrap();
        let mut progress = NoopProgressPublisher;

        let error = run_dedupe_with_filesystem_and_cancellation_with_run_id(
            RunDedupeRequest::new(input_dir.display().to_string(), "/", 10, 0, true).unwrap(),
            &mut progress,
            &crate::application::NoopCancellationToken,
            "run-test".to_string(),
        )
        .unwrap_err();

        match error {
            DedupeError::InvalidOutputDir(message) => {
                assert!(message.contains("not a safe output directory"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    struct NoopProgressPublisher;

    impl ProgressPublisher for NoopProgressPublisher {
        fn emit(&mut self, _update: ProgressUpdate) {}
    }
}

pub fn run_dedupe_with_observer<S, W, H, M, F, P, C, O>(
    request: RunDedupeRequest,
    scanner: &S,
    writer: &W,
    hasher: &H,
    size_reader: &M,
    output_filter: &F,
    progress: &mut P,
    cancellation: &C,
    observer: &mut O,
) -> Result<RunDedupeResult>
where
    S: crate::application::ImageScanner,
    W: crate::application::OutputWriter,
    H: crate::application::ImageHasher + Sync,
    M: crate::application::FileSizeReader,
    F: crate::application::OutputDirectoryFilter,
    P: ProgressPublisher,
    C: CancellationToken,
    O: RunObserver,
{
    observer.on_started(&request);
    match run_dedupe_cancellable_with_filter(
        request.clone(),
        scanner,
        writer,
        hasher,
        size_reader,
        output_filter,
        progress,
        cancellation,
    ) {
        Ok(result) => {
            observer.on_succeeded(&request, &result);
            Ok(result)
        }
        Err(error) => {
            observer.on_failed(&request, &error);
            Err(error)
        }
    }
}
