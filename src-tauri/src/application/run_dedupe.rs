use super::image_entries::collect_image_entries;
use super::output_filter::{ComposedOutputDirectoryFilter, OutputDirectoryFilter, WarningLog};
use super::{
    AppLocale, DedupeReport, DedupeSummary, FolderBreakdownEntry, ProgressUpdate, RunDedupeRequest,
    RunDedupeResult, RunWarning,
};
use crate::domain::ImageEntry;
use crate::errors::{DedupeError, Result};
use crate::grouping::try_group_by_similarity_with_progress_and_cancellation;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const RUN_LOG_TARGET: &str = "aba_avner::dedupe_run";

pub trait ProgressPublisher {
    fn emit(&mut self, update: ProgressUpdate);
}

pub trait CancellationToken {
    fn is_cancelled(&self) -> bool;
}

pub trait ImageScanner {
    fn collect_image_paths(&self, root: &Path) -> Result<Vec<PathBuf>>;
    fn collect_image_paths_with_cancellation(
        &self,
        root: &Path,
        _cancellation: &dyn CancellationToken,
    ) -> Result<Vec<PathBuf>> {
        self.collect_image_paths(root)
    }
}

pub trait OutputWriter {
    fn ensure_output_dir(&self, path: &Path) -> Result<()>;
    fn copy_unique_images<F>(
        &self,
        paths: &[PathBuf],
        output_dir: &Path,
        cancellation: &dyn CancellationToken,
        on_progress: F,
    ) -> Result<Vec<PathBuf>>
    where
        F: FnMut(usize);
    fn copy_non_unique_groups(
        &self,
        groups: &[Vec<ImageEntry>],
        selected_paths: &[PathBuf],
        output_dir: &Path,
        locale: AppLocale,
        export_similar_image_groups: bool,
        cancellation: &dyn CancellationToken,
    ) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileIssue {
    pub path: PathBuf,
    pub detail: String,
}

impl FileIssue {
    pub fn to_warning_message(&self) -> String {
        format!("{}: {}", self.path.display(), self.detail)
    }

    pub fn into_warning(self) -> RunWarning {
        RunWarning::file_issue(self.path, self.detail)
    }
}

pub trait ImageHasher {
    fn load_and_hash(&self, path: &Path) -> std::result::Result<u64, FileIssue>;
}

pub trait FileSizeReader {
    fn file_size_bytes(&self, path: &Path) -> std::result::Result<u64, FileIssue>;
}

pub fn run_dedupe<S, W, H, M, P>(
    request: RunDedupeRequest,
    scanner: &S,
    writer: &W,
    hasher: &H,
    size_reader: &M,
    progress: &mut P,
) -> Result<RunDedupeResult>
where
    S: ImageScanner,
    W: OutputWriter,
    H: ImageHasher + Sync,
    M: FileSizeReader,
    P: ProgressPublisher,
{
    let output_filter = ComposedOutputDirectoryFilter::new(scanner, hasher, size_reader);
    let cancellation = super::NoopCancellationToken;
    run_dedupe_cancellable_with_filter(
        request,
        scanner,
        writer,
        hasher,
        size_reader,
        &output_filter,
        progress,
        &cancellation,
    )
}

pub fn run_dedupe_cancellable<S, W, H, M, P, C>(
    request: RunDedupeRequest,
    scanner: &S,
    writer: &W,
    hasher: &H,
    size_reader: &M,
    progress: &mut P,
    cancellation: &C,
) -> Result<RunDedupeResult>
where
    S: ImageScanner,
    W: OutputWriter,
    H: ImageHasher + Sync,
    M: FileSizeReader,
    P: ProgressPublisher,
    C: CancellationToken,
{
    let output_filter = ComposedOutputDirectoryFilter::new(scanner, hasher, size_reader);
    run_dedupe_cancellable_with_filter(
        request,
        scanner,
        writer,
        hasher,
        size_reader,
        &output_filter,
        progress,
        cancellation,
    )
}

pub fn run_dedupe_with_filter<S, W, H, M, F, P>(
    request: RunDedupeRequest,
    scanner: &S,
    writer: &W,
    hasher: &H,
    size_reader: &M,
    output_filter: &F,
    progress: &mut P,
) -> Result<RunDedupeResult>
where
    S: ImageScanner,
    W: OutputWriter,
    H: ImageHasher + Sync,
    M: FileSizeReader,
    F: OutputDirectoryFilter,
    P: ProgressPublisher,
{
    let cancellation = super::NoopCancellationToken;
    run_dedupe_cancellable_with_filter(
        request,
        scanner,
        writer,
        hasher,
        size_reader,
        output_filter,
        progress,
        &cancellation,
    )
}

pub fn run_dedupe_cancellable_with_filter<S, W, H, M, F, P, C>(
    request: RunDedupeRequest,
    scanner: &S,
    writer: &W,
    hasher: &H,
    size_reader: &M,
    output_filter: &F,
    progress: &mut P,
    cancellation: &C,
) -> Result<RunDedupeResult>
where
    S: ImageScanner,
    W: OutputWriter,
    H: ImageHasher + Sync,
    M: FileSizeReader,
    F: OutputDirectoryFilter,
    P: ProgressPublisher,
    C: CancellationToken,
{
    let mut warnings = WarningLog::default();
    let run_started_at = Instant::now();

    ensure_not_cancelled(cancellation)?;
    writer.ensure_output_dir(&request.output_dir)?;

    progress.emit(ProgressUpdate::new("scan", 0, 1, "Scanning for images"));
    let scan_started_at = Instant::now();
    let paths = scanner.collect_image_paths_with_cancellation(&request.input_dir, cancellation)?;
    log_stage_timing("scan", scan_started_at.elapsed(), paths.len());
    progress.emit(ProgressUpdate::new(
        "scan",
        completed_progress_current(paths.len()),
        stage_progress_total(paths.len()),
        "Scan complete",
    ));

    ensure_not_cancelled(cancellation)?;
    progress.emit(ProgressUpdate::new(
        "hash",
        0,
        paths.len().max(1),
        "Hashing images",
    ));
    let total = stage_progress_total(paths.len());
    let hash_started_at = Instant::now();
    let hashed = collect_image_entries(
        &paths,
        hasher,
        size_reader,
        request.min_image_size_bytes,
        cancellation,
        |idx| {
            progress.emit(ProgressUpdate::new(
                "hash",
                idx + 1,
                total,
                "Hashing images",
            ));
        },
    )?;
    log_stage_timing("hash", hash_started_at.elapsed(), hashed.entries.len());
    warnings.extend(hashed.warnings);
    let entries = hashed.entries;

    ensure_not_cancelled(cancellation)?;
    progress.emit(ProgressUpdate::new(
        "group",
        0,
        stage_progress_total(entries.len()),
        "Grouping similar images",
    ));
    let group_total = stage_progress_total(entries.len());
    let group_started_at = Instant::now();
    let groups = try_group_by_similarity_with_progress_and_cancellation(
        &entries,
        request.similarity_threshold,
        || ensure_not_cancelled(cancellation),
        |idx| {
            progress.emit(ProgressUpdate::new(
                "group",
                idx,
                group_total,
                "Grouping similar images",
            ));
        },
    )?;
    log_stage_timing("group", group_started_at.elapsed(), groups.len());
    progress.emit(ProgressUpdate::new(
        "group",
        completed_progress_current(entries.len()),
        group_total,
        "Grouping complete",
    ));

    ensure_not_cancelled(cancellation)?;
    let unique_entries: Vec<ImageEntry> = groups
        .iter()
        .filter_map(|group| group.first().cloned())
        .collect();
    let unique_entries = if request.filter_existing_output {
        let filter_started_at = Instant::now();
        let filtered = output_filter.filter_existing_output(
            unique_entries,
            &request.output_dir,
            request.similarity_threshold,
            cancellation,
        )?;
        log_stage_timing(
            "filter_output",
            filter_started_at.elapsed(),
            filtered.entries.len(),
        );
        warnings.extend(filtered.warnings);
        filtered.entries
    } else {
        unique_entries
    };
    let unique_paths: Vec<PathBuf> = unique_entries
        .iter()
        .map(|entry| entry.path.clone())
        .collect();
    let copy_total = stage_progress_total(unique_paths.len());
    progress.emit(ProgressUpdate::new(
        "copy",
        0,
        copy_total,
        "Copying unique images",
    ));
    ensure_not_cancelled(cancellation)?;
    let copy_started_at = Instant::now();
    let copied_paths = writer.copy_unique_images(
        &unique_paths,
        &request.output_dir,
        cancellation,
        |current| {
            progress.emit(ProgressUpdate::new(
                "copy",
                current,
                copy_total,
                "Copying unique images",
            ));
        },
    )?;
    log_stage_timing("copy", copy_started_at.elapsed(), copied_paths.len());
    ensure_not_cancelled(cancellation)?;
    writer.copy_non_unique_groups(
        &groups,
        &unique_paths,
        &request.output_dir,
        request.locale,
        true,
        cancellation,
    )?;
    progress.emit(ProgressUpdate::new(
        "copy",
        completed_progress_current(copied_paths.len()),
        copy_total,
        "Copy complete",
    ));

    let compared_image_count = entries.len();
    let copied_unique_image_count = copied_paths.len();
    let report = build_dedupe_report(&groups);

    let (_warning_messages, warning_details) = warnings.into_parts();

    Ok(RunDedupeResult {
        total_images: compared_image_count,
        unique_images: copied_unique_image_count,
        duplicate_images: compared_image_count.saturating_sub(copied_unique_image_count),
        report,
        output_dir: request.output_dir,
        warnings: warning_details,
    })
    .inspect(|result| {
        log_run_timing(
            run_started_at.elapsed(),
            result.total_images,
            result.unique_images,
        );
    })
}

pub(crate) fn ensure_not_cancelled(cancellation: &(impl CancellationToken + ?Sized)) -> Result<()> {
    if cancellation.is_cancelled() {
        Err(DedupeError::Cancelled)
    } else {
        Ok(())
    }
}

pub(crate) fn log_stage_timing(stage: &str, elapsed: Duration, item_count: usize) {
    log::info!(
        target: RUN_LOG_TARGET,
        "{}",
        format_stage_timing_log(stage, elapsed, item_count)
    );
}

pub(crate) fn log_run_timing(elapsed: Duration, total_images: usize, unique_images: usize) {
    log::info!(
        target: RUN_LOG_TARGET,
        "event=run_timing elapsed_ms={} total_images={} unique_images={}",
        elapsed.as_millis(),
        total_images,
        unique_images
    );
}

fn format_stage_timing_log(stage: &str, elapsed: Duration, item_count: usize) -> String {
    format!(
        "event=stage_timing stage={} elapsed_ms={} item_count={}",
        stage,
        elapsed.as_millis(),
        item_count
    )
}

pub(crate) fn stage_progress_total(item_count: usize) -> usize {
    item_count.max(1)
}

pub(crate) fn completed_progress_current(item_count: usize) -> usize {
    stage_progress_total(item_count)
}

pub(crate) fn build_dedupe_report(groups: &[Vec<ImageEntry>]) -> DedupeReport {
    let total_images = groups.iter().map(Vec::len).sum::<usize>();
    let unique_images = groups.len();
    let duplicate_images = total_images.saturating_sub(unique_images);

    let mut storage_saved_bytes = 0_u64;
    let mut per_folder: HashMap<String, (usize, u64)> = HashMap::new();

    for group in groups {
        for duplicate in group.iter().skip(1) {
            storage_saved_bytes += duplicate.size_bytes;

            let folder_path = duplicate
                .path
                .parent()
                .map(|path| path.display().to_string())
                .unwrap_or_default();
            let stats = per_folder.entry(folder_path).or_insert((0, 0));
            stats.0 += 1;
            stats.1 += duplicate.size_bytes;
        }
    }

    let mut folder_breakdown: Vec<FolderBreakdownEntry> = per_folder
        .into_iter()
        .map(|(path, (duplicates, wasted_bytes))| FolderBreakdownEntry {
            path,
            duplicates,
            wasted_bytes,
        })
        .collect();
    folder_breakdown.sort_by(|left, right| {
        right
            .wasted_bytes
            .cmp(&left.wasted_bytes)
            .then_with(|| right.duplicates.cmp(&left.duplicates))
            .then_with(|| left.path.cmp(&right.path))
    });

    DedupeReport {
        storage_saved_bytes,
        storage_saved_human: humanize_bytes(storage_saved_bytes),
        summary: DedupeSummary {
            total_images,
            unique_images,
            duplicate_images,
        },
        folder_breakdown,
    }
}

fn humanize_bytes(size_bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

    let mut value = size_bytes as f64;
    let mut unit_index = 0;
    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{size_bytes} {}", UNITS[unit_index])
    } else {
        format!("{value:.1} {}", UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_dedupe_report, ensure_not_cancelled, format_stage_timing_log, humanize_bytes,
        CancellationToken,
    };
    use crate::application::FolderBreakdownEntry;
    use crate::domain::ImageEntry;
    use crate::errors::DedupeError;
    use std::path::PathBuf;
    use std::time::Duration;

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

    #[test]
    fn stage_timing_log_uses_structured_fields() {
        let message = format_stage_timing_log("hash", Duration::from_millis(17), 42);

        assert_eq!(
            message,
            "event=stage_timing stage=hash elapsed_ms=17 item_count=42"
        );
    }

    #[test]
    fn ensure_not_cancelled_accepts_active_token() {
        assert!(ensure_not_cancelled(&NeverCancelled).is_ok());
    }

    #[test]
    fn ensure_not_cancelled_rejects_cancelled_token() {
        let error = ensure_not_cancelled(&AlwaysCancelled).unwrap_err();

        assert!(matches!(error, DedupeError::Cancelled));
    }

    #[test]
    fn dedupe_report_sums_duplicate_sizes_and_sorts_folders_by_wasted_space() {
        let groups = vec![
            vec![
                ImageEntry {
                    path: PathBuf::from("/input/album-a/keep.bmp"),
                    hash: 1,
                    size_bytes: 10,
                },
                ImageEntry {
                    path: PathBuf::from("/input/album-a/drop.bmp"),
                    hash: 1,
                    size_bytes: 20,
                },
                ImageEntry {
                    path: PathBuf::from("/input/album-b/drop.bmp"),
                    hash: 1,
                    size_bytes: 30,
                },
            ],
            vec![ImageEntry {
                path: PathBuf::from("/input/album-c/solo.bmp"),
                hash: 2,
                size_bytes: 40,
            }],
        ];

        let report = build_dedupe_report(&groups);

        assert_eq!(report.storage_saved_bytes, 50);
        assert_eq!(report.storage_saved_human, "50 B");
        assert_eq!(report.summary.total_images, 4);
        assert_eq!(report.summary.unique_images, 2);
        assert_eq!(report.summary.duplicate_images, 2);
        assert_eq!(
            report.folder_breakdown,
            vec![
                FolderBreakdownEntry {
                    path: "/input/album-b".to_string(),
                    duplicates: 1,
                    wasted_bytes: 30,
                },
                FolderBreakdownEntry {
                    path: "/input/album-a".to_string(),
                    duplicates: 1,
                    wasted_bytes: 20,
                },
            ]
        );
    }

    #[test]
    fn humanize_bytes_formats_binary_units() {
        assert_eq!(humanize_bytes(512), "512 B");
        assert_eq!(humanize_bytes(1_048_576), "1.0 MB");
        assert_eq!(humanize_bytes(1_073_741_824), "1.0 GB");
    }
}
