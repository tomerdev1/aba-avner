use super::image_entries::collect_image_entries;
use super::output_filter::{OutputDirectoryFilter, WarningLog};
use super::run_dedupe::{
    build_dedupe_report, completed_progress_current, ensure_not_cancelled, log_run_timing,
    log_stage_timing, stage_progress_total, CancellationToken, FileSizeReader, ImageHasher,
    ImageScanner, OutputWriter, ProgressPublisher,
};
use super::{
    DuplicateReviewGroup, ExportSelectionsRequest, PrepareReviewResult, ProgressUpdate,
    RunDedupeRequest, RunDedupeResult,
};
use crate::domain::ImageEntry;
use crate::errors::{DedupeError, Result};
use crate::grouping::try_group_by_similarity_with_progress_and_cancellation;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

pub fn prepare_review_cancellable_with_filter<S, W, H, M, F, P, C>(
    request: RunDedupeRequest,
    scanner: &S,
    writer: &W,
    hasher: &H,
    size_reader: &M,
    progress: &mut P,
    output_filter: &F,
    cancellation: &C,
) -> Result<PrepareReviewResult>
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
    let paths = scanner.collect_image_paths_with_cancellation(&request.input_dir, cancellation)?;
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

    let (_warning_messages, warning_details) = warnings.into_parts();
    let review_result = PrepareReviewResult {
        groups: build_review_groups(&groups),
        warnings: warning_details,
    };
    log_run_timing(
        run_started_at.elapsed(),
        entries.len(),
        review_result.groups.len(),
    );

    let _ = output_filter;
    Ok(review_result)
}

pub fn export_review_selections_with_filter<W, H, M, F, P, C>(
    request: ExportSelectionsRequest,
    writer: &W,
    hasher: &H,
    size_reader: &M,
    output_filter: &F,
    progress: &mut P,
    cancellation: &C,
) -> Result<RunDedupeResult>
where
    W: OutputWriter,
    H: ImageHasher + Sync,
    M: FileSizeReader,
    F: OutputDirectoryFilter,
    P: ProgressPublisher,
    C: CancellationToken,
{
    let mut warnings = WarningLog::default();

    ensure_not_cancelled(cancellation)?;
    writer.ensure_output_dir(&request.output_dir)?;

    let groups = materialize_review_groups(&request.groups, size_reader)?;
    let selected_paths = resolve_selected_paths(&request.groups, &request.selections)?;
    let selected_entries = load_selected_entries(&groups, &selected_paths, hasher)?;
    let selected_entries = if request.filter_existing_output {
        let filtered = output_filter.filter_existing_output(
            selected_entries,
            &request.output_dir,
            request.similarity_threshold,
            cancellation,
        )?;
        warnings.extend(filtered.warnings);
        filtered.entries
    } else {
        selected_entries
    };

    let unique_paths: Vec<PathBuf> = selected_entries
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
    ensure_not_cancelled(cancellation)?;
    writer.copy_non_unique_groups(
        &groups,
        &unique_paths,
        &request.output_dir,
        request.locale,
        request.export_similar_image_groups,
        cancellation,
    )?;
    progress.emit(ProgressUpdate::new(
        "copy",
        completed_progress_current(copied_paths.len()),
        copy_total,
        "Copy complete",
    ));

    let compared_image_count = groups.iter().map(Vec::len).sum::<usize>();
    let report = build_dedupe_report(&groups);
    let (_warning_messages, warning_details) = warnings.into_parts();

    Ok(RunDedupeResult {
        total_images: compared_image_count,
        unique_images: copied_paths.len(),
        duplicate_images: compared_image_count.saturating_sub(copied_paths.len()),
        report,
        output_dir: request.output_dir,
        warnings: warning_details,
    })
}

fn build_review_groups(groups: &[Vec<ImageEntry>]) -> Vec<DuplicateReviewGroup> {
    groups
        .iter()
        .enumerate()
        .map(|(index, group)| DuplicateReviewGroup {
            group_id: format!("g{}", index + 1),
            images: group
                .iter()
                .map(|entry| entry.path.display().to_string())
                .collect(),
            suggested: group
                .first()
                .map(|entry| entry.path.display().to_string())
                .unwrap_or_default(),
        })
        .collect()
}

fn materialize_review_groups(
    groups: &[DuplicateReviewGroup],
    size_reader: &impl FileSizeReader,
) -> Result<Vec<Vec<ImageEntry>>> {
    groups
        .iter()
        .map(|group| {
            if group.images.is_empty() {
                return Err(DedupeError::InvalidSelection(format!(
                    "group {} has no images",
                    group.group_id
                )));
            }

            group
                .images
                .iter()
                .map(|path| {
                    let normalized = normalize_review_path(path)?;
                    let size_bytes = size_reader
                        .file_size_bytes(&normalized)
                        .map_err(file_issue_to_dedupe_error)?;
                    Ok(ImageEntry {
                        path: normalized,
                        hash: 0,
                        size_bytes,
                    })
                })
                .collect()
        })
        .collect()
}

fn resolve_selected_paths(
    groups: &[DuplicateReviewGroup],
    selections: &HashMap<String, PathBuf>,
) -> Result<Vec<PathBuf>> {
    let mut selected = Vec::with_capacity(groups.len());

    for group in groups {
        let available: Vec<PathBuf> = group
            .images
            .iter()
            .map(|path| normalize_review_path(path))
            .collect::<Result<Vec<_>>>()?;
        let suggested = normalize_review_path(&group.suggested)?;
        if !available.iter().any(|path| path == &suggested) {
            return Err(DedupeError::InvalidSelection(format!(
                "group {} suggested path is not part of the group",
                group.group_id
            )));
        }

        let candidate = selections
            .get(&group.group_id)
            .cloned()
            .unwrap_or_else(|| suggested.clone());
        if !available.iter().any(|path| path == &candidate) {
            return Err(DedupeError::InvalidSelection(format!(
                "group {} selection {} is not part of the group",
                group.group_id,
                candidate.display()
            )));
        }

        selected.push(candidate);
    }

    let known_group_ids: HashSet<&str> =
        groups.iter().map(|group| group.group_id.as_str()).collect();
    if let Some((unknown_group_id, _)) = selections
        .iter()
        .find(|(group_id, _)| !known_group_ids.contains(group_id.as_str()))
    {
        return Err(DedupeError::InvalidSelection(format!(
            "group {} is not part of this review result",
            unknown_group_id
        )));
    }

    Ok(selected)
}

fn load_selected_entries(
    groups: &[Vec<ImageEntry>],
    selected_paths: &[PathBuf],
    hasher: &(impl ImageHasher + Sync),
) -> Result<Vec<ImageEntry>> {
    let selected_paths: HashSet<&PathBuf> = selected_paths.iter().collect();
    let mut selected_entries = Vec::with_capacity(selected_paths.len());

    for group in groups {
        for entry in group {
            if !selected_paths.contains(&entry.path) {
                continue;
            }

            let hash = hasher
                .load_and_hash(&entry.path)
                .map_err(file_issue_to_dedupe_error)?;
            selected_entries.push(ImageEntry {
                path: entry.path.clone(),
                hash,
                size_bytes: entry.size_bytes,
            });
        }
    }

    Ok(selected_entries)
}

fn normalize_review_path(path: &str) -> Result<PathBuf> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(DedupeError::InvalidSelection(
            "review group image path cannot be empty".to_string(),
        ));
    }

    Ok(normalize_path(Path::new(trimmed)))
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if matches!(
                    normalized.components().next_back(),
                    Some(std::path::Component::Normal(_))
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

fn file_issue_to_dedupe_error(issue: super::run_dedupe::FileIssue) -> DedupeError {
    DedupeError::LoadImage(issue.to_warning_message())
}

#[cfg(test)]
mod tests {
    use super::{
        build_review_groups, export_review_selections_with_filter, resolve_selected_paths,
    };
    use crate::application::{
        AppLocale, CancellationToken, DuplicateReviewGroup, ExportSelectionsRequest, FileIssue,
        FileSizeReader, ImageHasher, OutputDirectoryFilter, OutputFilterResult, OutputWriter,
        ProgressPublisher, ProgressUpdate, WarningLog,
    };
    use crate::domain::ImageEntry;
    use crate::errors::DedupeError;
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn review_groups_use_stable_ids_and_first_image_as_suggested() {
        let groups = build_review_groups(&[
            vec![
                ImageEntry {
                    path: PathBuf::from("/input/a.jpg"),
                    hash: 1,
                    size_bytes: 10,
                },
                ImageEntry {
                    path: PathBuf::from("/input/b.jpg"),
                    hash: 1,
                    size_bytes: 10,
                },
            ],
            vec![ImageEntry {
                path: PathBuf::from("/input/c.jpg"),
                hash: 2,
                size_bytes: 10,
            }],
        ]);

        assert_eq!(
            groups,
            vec![
                DuplicateReviewGroup {
                    group_id: "g1".to_string(),
                    images: vec!["/input/a.jpg".to_string(), "/input/b.jpg".to_string()],
                    suggested: "/input/a.jpg".to_string(),
                },
                DuplicateReviewGroup {
                    group_id: "g2".to_string(),
                    images: vec!["/input/c.jpg".to_string()],
                    suggested: "/input/c.jpg".to_string(),
                },
            ]
        );
    }

    #[test]
    fn missing_selection_uses_suggested_path() {
        let selected = resolve_selected_paths(
            &[DuplicateReviewGroup {
                group_id: "g1".to_string(),
                images: vec!["/input/a.jpg".to_string(), "/input/b.jpg".to_string()],
                suggested: "/input/a.jpg".to_string(),
            }],
            &HashMap::new(),
        )
        .unwrap();

        assert_eq!(selected, vec![PathBuf::from("/input/a.jpg")]);
    }

    #[test]
    fn invalid_selection_path_returns_clear_error() {
        let error = resolve_selected_paths(
            &[DuplicateReviewGroup {
                group_id: "g1".to_string(),
                images: vec!["/input/a.jpg".to_string(), "/input/b.jpg".to_string()],
                suggested: "/input/a.jpg".to_string(),
            }],
            &HashMap::from([("g1".to_string(), PathBuf::from("/input/missing.jpg"))]),
        )
        .unwrap_err();

        match error {
            DedupeError::InvalidSelection(message) => {
                assert_eq!(
                    message,
                    "group g1 selection /input/missing.jpg is not part of the group"
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn export_review_skips_similar_group_refresh_when_disabled() {
        struct RecordingWriter {
            called: AtomicBool,
        }

        impl OutputWriter for RecordingWriter {
            fn ensure_output_dir(&self, _path: &Path) -> crate::errors::Result<()> {
                Ok(())
            }

            fn copy_unique_images<F>(
                &self,
                paths: &[PathBuf],
                _output_dir: &Path,
                _cancellation: &dyn CancellationToken,
                mut on_progress: F,
            ) -> crate::errors::Result<Vec<PathBuf>>
            where
                F: FnMut(usize),
            {
                for index in 0..paths.len() {
                    on_progress(index + 1);
                }
                Ok(paths.to_vec())
            }

            fn copy_non_unique_groups(
                &self,
                _groups: &[Vec<ImageEntry>],
                _selected_paths: &[PathBuf],
                _output_dir: &Path,
                _locale: AppLocale,
                export_similar_image_groups: bool,
                _cancellation: &dyn CancellationToken,
            ) -> crate::errors::Result<()> {
                self.called
                    .store(export_similar_image_groups, Ordering::SeqCst);
                Ok(())
            }
        }

        struct NoopHasher;

        impl ImageHasher for NoopHasher {
            fn load_and_hash(&self, _path: &Path) -> std::result::Result<u64, FileIssue> {
                Ok(1)
            }
        }

        struct FixedSizeReader;

        impl FileSizeReader for FixedSizeReader {
            fn file_size_bytes(&self, _path: &Path) -> std::result::Result<u64, FileIssue> {
                Ok(10)
            }
        }

        struct PassthroughOutputFilter;

        impl OutputDirectoryFilter for PassthroughOutputFilter {
            fn filter_existing_output(
                &self,
                entries: Vec<ImageEntry>,
                _output_dir: &Path,
                _similarity_threshold: u32,
                _cancellation: &dyn CancellationToken,
            ) -> crate::errors::Result<OutputFilterResult> {
                Ok(OutputFilterResult {
                    entries,
                    warnings: WarningLog::default(),
                })
            }
        }

        struct RecordingProgress;

        impl ProgressPublisher for RecordingProgress {
            fn emit(&mut self, _update: ProgressUpdate) {}
        }

        struct NeverCancelled;

        impl CancellationToken for NeverCancelled {
            fn is_cancelled(&self) -> bool {
                false
            }
        }

        let writer = RecordingWriter {
            called: AtomicBool::new(true),
        };
        let hasher = NoopHasher;
        let size_reader = FixedSizeReader;
        let output_filter = PassthroughOutputFilter;
        let mut progress = RecordingProgress;
        let cancellation = NeverCancelled;

        let result = export_review_selections_with_filter(
            ExportSelectionsRequest {
                output_dir: PathBuf::from("/output"),
                similarity_threshold: 10,
                filter_existing_output: false,
                export_similar_image_groups: false,
                locale: AppLocale::En,
                groups: vec![DuplicateReviewGroup {
                    group_id: "g1".to_string(),
                    images: vec!["/input/a.jpg".to_string(), "/input/b.jpg".to_string()],
                    suggested: "/input/a.jpg".to_string(),
                }],
                selections: HashMap::new(),
            },
            &writer,
            &hasher,
            &size_reader,
            &output_filter,
            &mut progress,
            &cancellation,
        )
        .unwrap();

        assert_eq!(result.unique_images, 1);
        assert!(!writer.called.load(Ordering::SeqCst));
    }
}
