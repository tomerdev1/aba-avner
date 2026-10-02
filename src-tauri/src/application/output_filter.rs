use crate::application::RunWarning;
use crate::bk_tree::BKTree;
use crate::domain::ImageEntry;
use crate::errors::Result;
use std::ops::Index;
use std::path::{Path, PathBuf};
use super::run_dedupe::{
    ensure_not_cancelled, CancellationToken, FileSizeReader, ImageHasher, ImageScanner,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WarningLog {
    warnings: Vec<RunWarning>,
}

impl WarningLog {
    pub fn is_empty(&self) -> bool {
        self.warnings.is_empty()
    }

    pub fn len(&self) -> usize {
        self.warnings.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, RunWarning> {
        self.warnings.iter()
    }

    pub fn push(&mut self, warning: RunWarning) {
        self.warnings.push(warning);
    }

    pub fn extend(&mut self, other: WarningLog) {
        self.warnings.extend(other.warnings);
    }

    pub fn into_parts(self) -> (Vec<String>, Vec<RunWarning>) {
        let messages = self.warnings.iter().map(RunWarning::to_message).collect();
        (messages, self.warnings)
    }
}

impl From<Vec<RunWarning>> for WarningLog {
    fn from(warnings: Vec<RunWarning>) -> Self {
        Self { warnings }
    }
}

impl Index<usize> for WarningLog {
    type Output = RunWarning;

    fn index(&self, index: usize) -> &Self::Output {
        &self.warnings[index]
    }
}

pub trait OutputDirectoryFilter {
    fn filter_existing_output(
        &self,
        entries: Vec<ImageEntry>,
        output_dir: &Path,
        similarity_threshold: u32,
        cancellation: &dyn CancellationToken,
    ) -> Result<OutputFilterResult>;
}

#[derive(Debug)]
pub struct OutputFilterResult {
    pub entries: Vec<ImageEntry>,
    pub warnings: WarningLog,
}

pub(crate) struct ComposedOutputDirectoryFilter<'a, S, H, M> {
    scanner: &'a S,
    hasher: &'a H,
    size_reader: &'a M,
}

impl<'a, S, H, M> ComposedOutputDirectoryFilter<'a, S, H, M> {
    pub(crate) fn new(scanner: &'a S, hasher: &'a H, size_reader: &'a M) -> Self {
        Self {
            scanner,
            hasher,
            size_reader,
        }
    }
}

impl<S, H, M> OutputDirectoryFilter for ComposedOutputDirectoryFilter<'_, S, H, M>
where
    S: ImageScanner,
    H: ImageHasher + Sync,
    M: FileSizeReader,
{
    fn filter_existing_output(
        &self,
        entries: Vec<ImageEntry>,
        output_dir: &Path,
        similarity_threshold: u32,
        cancellation: &dyn CancellationToken,
    ) -> Result<OutputFilterResult> {
        filter_existing_output_entries(
            entries,
            self.scanner,
            self.hasher,
            self.size_reader,
            output_dir,
            similarity_threshold,
            cancellation,
        )
    }
}

pub(crate) fn filter_existing_output_entries(
    entries: Vec<ImageEntry>,
    scanner: &impl ImageScanner,
    hasher: &(impl ImageHasher + Sync),
    _size_reader: &impl FileSizeReader,
    output_dir: &Path,
    similarity_threshold: u32,
    cancellation: &dyn CancellationToken,
) -> Result<OutputFilterResult> {
    let mut warnings = WarningLog::default();

    ensure_not_cancelled(cancellation)?;
    let existing_paths = match scanner.collect_image_paths(output_dir) {
        Ok(paths) => paths,
        Err(error) => {
            warnings.push(RunWarning::output_dir_scan_failed(
                output_dir.to_path_buf(),
                error.to_string(),
            ));
            return Ok(OutputFilterResult { entries, warnings });
        }
    };

    if existing_paths.is_empty() {
        return Ok(OutputFilterResult { entries, warnings });
    }

    let existing_index = collect_existing_output_index(
        &existing_paths,
        hasher,
        cancellation,
        &mut warnings,
    )?;
    if existing_index.hashed_image_count == 0 {
        return Ok(OutputFilterResult { entries, warnings });
    }

    let mut filtered = Vec::new();
    for entry in entries {
        ensure_not_cancelled(cancellation)?;
        let mut matches = Vec::new();
        existing_index
            .tree
            .search(entry.hash, similarity_threshold, &mut matches);
        if matches.is_empty() {
            filtered.push(entry);
        } else {
            warnings.push(RunWarning::similar_image_in_output(entry.path));
        }
    }

    Ok(OutputFilterResult {
        entries: filtered,
        warnings,
    })
}

struct ExistingOutputIndex {
    tree: BKTree,
    hashed_image_count: usize,
}

fn collect_existing_output_index(
    paths: &[PathBuf],
    hasher: &(impl ImageHasher + Sync),
    cancellation: &(impl CancellationToken + ?Sized),
    warnings: &mut WarningLog,
) -> Result<ExistingOutputIndex> {
    let mut tree = BKTree::new();
    let mut hashed_image_count = 0;
    let batch_size = existing_hash_batch_size(paths.len());

    for batch_paths in paths.chunks(batch_size) {
        hash_existing_output_batch(
            batch_paths,
            hasher,
            cancellation,
            warnings,
            &mut tree,
            &mut hashed_image_count,
        )?;
    }

    Ok(ExistingOutputIndex {
        tree,
        hashed_image_count,
    })
}

fn hash_existing_output_batch(
    batch_paths: &[PathBuf],
    hasher: &(impl ImageHasher + Sync),
    cancellation: &(impl CancellationToken + ?Sized),
    warnings: &mut WarningLog,
    tree: &mut BKTree,
    hashed_image_count: &mut usize,
) -> Result<()> {
    for path in batch_paths {
        ensure_not_cancelled(cancellation)?;
        let result = hasher.load_and_hash(path);
        ensure_not_cancelled(cancellation)?;
        match result {
            Ok(hash) => {
                tree.insert(hash, *hashed_image_count);
                *hashed_image_count += 1;
            }
            Err(issue) => warnings.push(issue.into_warning()),
        }
    }

    Ok(())
}

fn existing_hash_batch_size(path_count: usize) -> usize {
    path_count.max(1)
}

#[cfg(test)]
mod tests {
    use super::{ComposedOutputDirectoryFilter, OutputDirectoryFilter};
    use crate::application::{CancellationToken, FileIssue, ImageHasher, ImageScanner};
    use crate::domain::ImageEntry;
    use crate::errors::{DedupeError, Result};
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    struct StubScanner {
        paths: Vec<PathBuf>,
        error: Option<DedupeError>,
    }

    impl ImageScanner for StubScanner {
        fn collect_image_paths(&self, _root: &Path) -> Result<Vec<PathBuf>> {
            match &self.error {
                Some(DedupeError::ReadDir(path)) => Err(DedupeError::ReadDir(path.clone())),
                Some(other) => panic!("unexpected scanner error variant: {other:?}"),
                None => Ok(self.paths.clone()),
            }
        }
    }

    struct StubHasher;

    impl ImageHasher for StubHasher {
        fn load_and_hash(&self, path: &Path) -> std::result::Result<u64, FileIssue> {
            match path.file_name().and_then(|name| name.to_str()) {
                Some("existing.png") | Some("incoming.png") => Ok(7),
                Some("other.png") => Ok(11),
                _ => Err(FileIssue {
                    path: path.to_path_buf(),
                    detail: "unexpected path".to_string(),
                }),
            }
        }
    }

    struct StubSizeReader;

    impl crate::application::FileSizeReader for StubSizeReader {
        fn file_size_bytes(&self, _path: &Path) -> std::result::Result<u64, FileIssue> {
            Ok(1)
        }
    }

    struct NeverCancelled;

    impl CancellationToken for NeverCancelled {
        fn is_cancelled(&self) -> bool {
            false
        }
    }

    #[derive(Default)]
    struct TestCancellation {
        cancelled: AtomicBool,
    }

    impl TestCancellation {
        fn cancel(&self) {
            self.cancelled.store(true, Ordering::SeqCst);
        }
    }

    impl CancellationToken for TestCancellation {
        fn is_cancelled(&self) -> bool {
            self.cancelled.load(Ordering::SeqCst)
        }
    }

    #[test]
    fn composed_filter_hashes_existing_output_without_input_min_size_gate() {
        let scanner = scanner_with_paths(&["existing.png"]);
        let filter = ComposedOutputDirectoryFilter::new(&scanner, &StubHasher, &StubSizeReader);
        let incoming_path = PathBuf::from("/input/incoming.png");

        let result = filter
            .filter_existing_output(
                vec![ImageEntry {
                    path: incoming_path.clone(),
                    hash: 7,
                    size_bytes: 0,
                }],
                Path::new("/output"),
                0,
                &NeverCancelled,
            )
            .unwrap();

        assert!(result.entries.is_empty());
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].path, incoming_path);
    }

    #[test]
    fn composed_filter_returns_original_entries_when_output_scan_fails() {
        let scanner = StubScanner {
            paths: Vec::new(),
            error: Some(DedupeError::ReadDir("/output".to_string())),
        };
        let filter = ComposedOutputDirectoryFilter::new(&scanner, &StubHasher, &StubSizeReader);
        let incoming = ImageEntry {
            path: PathBuf::from("/input/incoming.png"),
            hash: 7,
            size_bytes: 0,
        };

        let result = filter
            .filter_existing_output(
                vec![incoming.clone()],
                Path::new("/output"),
                0,
                &NeverCancelled,
            )
            .unwrap();

        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].path, incoming.path);
        assert_eq!(result.entries[0].hash, incoming.hash);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(
            result.warnings[0].code,
            crate::application::RunWarning::OUTPUT_DIR_SCAN_FAILED_CODE
        );
    }

    #[test]
    fn composed_filter_preserves_surviving_entry_order_and_warning_order() {
        struct MixedHasher;

        impl ImageHasher for MixedHasher {
            fn load_and_hash(&self, path: &Path) -> std::result::Result<u64, FileIssue> {
                match path.file_name().and_then(|name| name.to_str()) {
                    Some("broken.png") => Err(FileIssue {
                        path: path.to_path_buf(),
                        detail: "simulated failure".to_string(),
                    }),
                    Some("existing-a.png") | Some("skip-a.png") => Ok(7),
                    Some("existing-b.png") | Some("skip-b.png") => Ok(11),
                    Some("keep-a.png") => Ok(13),
                    Some("keep-b.png") => Ok(17),
                    _ => Err(FileIssue {
                        path: path.to_path_buf(),
                        detail: "unexpected path".to_string(),
                    }),
                }
            }
        }

        let scanner = scanner_with_paths(&["broken.png", "existing-a.png", "existing-b.png"]);
        let filter = ComposedOutputDirectoryFilter::new(&scanner, &MixedHasher, &StubSizeReader);
        let keep_a = PathBuf::from("/input/keep-a.png");
        let skip_a = PathBuf::from("/input/skip-a.png");
        let keep_b = PathBuf::from("/input/keep-b.png");
        let skip_b = PathBuf::from("/input/skip-b.png");

        let result = filter
            .filter_existing_output(
                vec![
                    ImageEntry {
                        path: keep_a.clone(),
                        hash: 13,
                        size_bytes: 0,
                    },
                    ImageEntry {
                        path: skip_a.clone(),
                        hash: 7,
                        size_bytes: 0,
                    },
                    ImageEntry {
                        path: keep_b.clone(),
                        hash: 17,
                        size_bytes: 0,
                    },
                    ImageEntry {
                        path: skip_b.clone(),
                        hash: 11,
                        size_bytes: 0,
                    },
                ],
                Path::new("/output"),
                0,
                &NeverCancelled,
            )
            .unwrap();

        let kept_paths: Vec<_> = result.entries.iter().map(|entry| entry.path.clone()).collect();
        assert_eq!(kept_paths, vec![keep_a, keep_b]);

        let warning_codes: Vec<_> = result.warnings.iter().map(|warning| warning.code).collect();
        assert_eq!(
            warning_codes,
            vec![
                crate::application::RunWarning::FILE_ISSUE_CODE,
                crate::application::RunWarning::SIMILAR_IMAGE_IN_OUTPUT_CODE,
                crate::application::RunWarning::SIMILAR_IMAGE_IN_OUTPUT_CODE,
            ]
        );
        assert_eq!(result.warnings[0].path, PathBuf::from("/output/broken.png"));
        assert_eq!(result.warnings[1].path, skip_a);
        assert_eq!(result.warnings[2].path, skip_b);
    }

    #[test]
    fn composed_filter_stops_existing_output_hashing_after_cancellation() {
        struct CancellingHasher {
            cancellation: Arc<TestCancellation>,
            existing_hash_calls: Arc<AtomicUsize>,
        }

        impl ImageHasher for CancellingHasher {
            fn load_and_hash(&self, path: &Path) -> std::result::Result<u64, FileIssue> {
                if path.starts_with("/output") {
                    let call_index = self.existing_hash_calls.fetch_add(1, Ordering::SeqCst);
                    if call_index == 0 {
                        self.cancellation.cancel();
                    }
                }

                Ok(path.to_string_lossy().len() as u64)
            }
        }

        let scanner = scanner_with_paths(&["existing-a.png", "existing-b.png", "existing-c.png"]);
        let cancellation = Arc::new(TestCancellation::default());
        let existing_hash_calls = Arc::new(AtomicUsize::new(0));
        let hasher = CancellingHasher {
            cancellation: cancellation.clone(),
            existing_hash_calls: existing_hash_calls.clone(),
        };
        let filter = ComposedOutputDirectoryFilter::new(&scanner, &hasher, &StubSizeReader);

        let error = filter
            .filter_existing_output(
                vec![ImageEntry {
                    path: PathBuf::from("/input/incoming.png"),
                    hash: 99,
                    size_bytes: 0,
                }],
                Path::new("/output"),
                0,
                cancellation.as_ref(),
            )
            .unwrap_err();

        assert!(matches!(error, DedupeError::Cancelled));
        assert_eq!(existing_hash_calls.load(Ordering::SeqCst), 1);
    }

    fn scanner_with_paths(names: &[&str]) -> StubScanner {
        StubScanner {
            paths: names
                .iter()
                .map(|name| PathBuf::from("/output").join(name))
                .collect(),
            error: None,
        }
    }
}
