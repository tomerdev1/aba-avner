use crate::domain::ImageEntry;
use crate::errors::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

use super::output_filter::WarningLog;
use super::run_dedupe::{ensure_not_cancelled, CancellationToken, FileSizeReader, ImageHasher};

const MIN_PARALLEL_HASH_COUNT: usize = 4;
const HASH_BATCH_MULTIPLIER: usize = 4;

pub(crate) struct CollectedEntries {
    pub(crate) entries: Vec<ImageEntry>,
    pub(crate) warnings: WarningLog,
}

pub(crate) fn collect_image_entries<F: FnMut(usize)>(
    paths: &[PathBuf],
    hasher: &(impl ImageHasher + Sync),
    size_reader: &impl FileSizeReader,
    min_image_size_bytes: u64,
    cancellation: &(impl CancellationToken + ?Sized),
    mut on_progress: F,
) -> Result<CollectedEntries> {
    let mut entries = Vec::with_capacity(paths.len());
    let mut warnings = WarningLog::default();
    let batch_size = parallel_hash_batch_size(paths.len());

    for (batch_idx, batch_paths) in paths.chunks(batch_size).enumerate() {
        process_hash_batch(
            batch_paths,
            batch_idx * batch_size,
            hasher,
            size_reader,
            min_image_size_bytes,
            cancellation,
            &mut entries,
            &mut warnings,
            &mut on_progress,
        )?;
    }

    Ok(CollectedEntries { entries, warnings })
}

fn process_hash_batch<F: FnMut(usize)>(
    batch_paths: &[PathBuf],
    batch_start: usize,
    hasher: &(impl ImageHasher + Sync),
    size_reader: &impl FileSizeReader,
    min_image_size_bytes: u64,
    cancellation: &(impl CancellationToken + ?Sized),
    entries: &mut Vec<ImageEntry>,
    warnings: &mut WarningLog,
    on_progress: &mut F,
) -> Result<()> {
    let mut hash_inputs = Vec::with_capacity(batch_paths.len());
    let mut hash_input_sizes = Vec::with_capacity(batch_paths.len());
    let mut hash_slots = Vec::with_capacity(batch_paths.len());

    for path in batch_paths {
        ensure_not_cancelled(cancellation)?;
        if let Some(size_bytes) = load_size_if_eligible(path, size_reader, min_image_size_bytes, warnings)
        {
            hash_slots.push(Some(hash_inputs.len()));
            hash_inputs.push(path);
            hash_input_sizes.push(size_bytes);
        } else {
            hash_slots.push(None);
        }
    }

    let hash_results = if hash_inputs.len() >= MIN_PARALLEL_HASH_COUNT {
        hash_inputs
            .par_iter()
            .map(|path| hasher.load_and_hash(path))
            .collect::<Vec<_>>()
    } else {
        hash_inputs
            .iter()
            .map(|path| hasher.load_and_hash(path))
            .collect::<Vec<_>>()
    };

    for (offset, hash_slot) in hash_slots.into_iter().enumerate() {
        ensure_not_cancelled(cancellation)?;
        if let Some(hash_idx) = hash_slot {
            let path = hash_inputs[hash_idx];
            match &hash_results[hash_idx] {
                Ok(hash) => entries.push(ImageEntry {
                    path: path.clone(),
                    hash: *hash,
                    size_bytes: hash_input_sizes[hash_idx],
                }),
                Err(issue) => warnings.push(issue.clone().into_warning()),
            }
        }

        on_progress(batch_start + offset);
    }

    Ok(())
}

fn parallel_hash_batch_size(path_count: usize) -> usize {
    let worker_count = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);

    if worker_count <= 1 || path_count < MIN_PARALLEL_HASH_COUNT {
        path_count.max(1)
    } else {
        (worker_count * HASH_BATCH_MULTIPLIER).max(1)
    }
}

fn load_size_if_eligible(
    path: &Path,
    size_reader: &impl FileSizeReader,
    min_image_size_bytes: u64,
    warnings: &mut WarningLog,
) -> Option<u64> {
    match size_reader.file_size_bytes(path) {
        Ok(size_bytes) if size_bytes >= min_image_size_bytes => Some(size_bytes),
        Ok(_) => None,
        Err(issue) => {
            warnings.push(issue.into_warning());
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::{FileIssue, ImageHasher, NoopCancellationToken};
    use crate::hashing::compute_dhash;
    use crate::infrastructure::FsFileSizeReader;
    use crate::scan::collect_image_paths;
    use image::{DynamicImage, ImageBuffer, Rgba};
    use std::collections::HashMap;
    use std::fs;
    use std::fs::write;
    use tempfile::tempdir;
    use std::time::Duration;

    struct FsTestHasher;

    impl ImageHasher for FsTestHasher {
        fn load_and_hash(&self, path: &Path) -> std::result::Result<u64, FileIssue> {
            let image = image::open(path).map_err(|e| FileIssue {
                path: path.to_path_buf(),
                detail: e.to_string(),
            })?;
            Ok(compute_dhash(&image))
        }
    }

    #[test]
    fn skips_images_below_min_size() {
        let temp = tempdir().unwrap();
        let small_path = temp.path().join("small.png");
        let large_path = temp.path().join("large.png");

        write(&small_path, vec![0u8; 10]).unwrap();

        let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_pixel(128, 128, Rgba([12, 34, 56, 255]));
        img.save(&large_path).unwrap();

        let paths = vec![small_path.clone(), large_path.clone()];
        let hasher = FsTestHasher;
        let size_reader = FsFileSizeReader;
        let cancellation = NoopCancellationToken;
        let result =
            collect_image_entries(&paths, &hasher, &size_reader, 100, &cancellation, |_| {})
                .unwrap();

        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].path, large_path);
    }

    #[test]
    fn respects_min_size_for_fixture_images() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let backup_dir = manifest_dir
            .join("..")
            .join("auto-tests")
            .join("small_images_limiter");

        let paths = collect_image_paths(&backup_dir).unwrap();
        assert!(
            !paths.is_empty(),
            "expected fixture images under auto-tests/small_images_limiter"
        );

        let min_size_bytes = 1_000_000;
        let expected = paths
            .iter()
            .filter(|path| {
                fs::metadata(path)
                    .map(|m| m.len() >= min_size_bytes)
                    .unwrap_or(false)
            })
            .count();

        assert!(
            expected < paths.len(),
            "fixture set should include some images below the min size threshold",
        );

        let hasher = FsTestHasher;
        let size_reader = FsFileSizeReader;
        let cancellation = NoopCancellationToken;
        let result = collect_image_entries(
            &paths,
            &hasher,
            &size_reader,
            min_size_bytes,
            &cancellation,
            |_| {},
        )
        .unwrap();

        assert_eq!(result.entries.len(), expected);
    }

    #[test]
    fn missing_file_after_scan_emits_warning_and_does_not_increase_total_images() {
        let temp = tempdir().unwrap();
        let image_path = temp.path().join("present.bmp");
        let missing_path = temp.path().join("missing.bmp");

        create_pattern_image(&image_path, 64, 64, 0);
        create_pattern_image(&missing_path, 64, 64, 1);

        let paths = vec![image_path.clone(), missing_path.clone()];
        std::fs::remove_file(&missing_path).unwrap();

        let hasher = FsTestHasher;
        let size_reader = FsFileSizeReader;
        let cancellation = NoopCancellationToken;
        let result =
            collect_image_entries(&paths, &hasher, &size_reader, 1, &cancellation, |_| {}).unwrap();

        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].path, image_path);
        let (_messages, warnings) = result.warnings.into_parts();
        assert!(warnings
            .iter()
            .any(|warning| warning.path.ends_with("missing.bmp")));
    }

    #[test]
    fn collect_image_entries_preserves_input_order_warnings_and_progress_across_mixed_results() {
        struct DelayedHasher {
            hashes: HashMap<PathBuf, u64>,
            delays_ms: HashMap<PathBuf, u64>,
            failing_path: PathBuf,
        }

        impl ImageHasher for DelayedHasher {
            fn load_and_hash(&self, path: &Path) -> std::result::Result<u64, FileIssue> {
                if let Some(delay_ms) = self.delays_ms.get(path) {
                    std::thread::sleep(Duration::from_millis(*delay_ms));
                }

                if path == self.failing_path.as_path() {
                    return Err(FileIssue {
                        path: path.to_path_buf(),
                        detail: "simulated failure".to_string(),
                    });
                }

                Ok(*self.hashes.get(path).unwrap())
            }
        }

        struct FixedSizeReader;

        impl FileSizeReader for FixedSizeReader {
            fn file_size_bytes(&self, _path: &Path) -> std::result::Result<u64, FileIssue> {
                Ok(10_000)
            }
        }

        let temp = tempdir().unwrap();
        let first = temp.path().join("first.bmp");
        let second = temp.path().join("second.bmp");
        let third = temp.path().join("third.bmp");

        create_pattern_image(&first, 64, 64, 0);
        create_pattern_image(&second, 64, 64, 1);
        create_pattern_image(&third, 64, 64, 2);

        let hasher = DelayedHasher {
            hashes: HashMap::from([
                (first.clone(), 101),
                (third.clone(), 303),
            ]),
            delays_ms: HashMap::from([
                (first.clone(), 50),
                (second.clone(), 0),
                (third.clone(), 0),
            ]),
            failing_path: second.clone(),
        };
        let size_reader = FixedSizeReader;
        let cancellation = NoopCancellationToken;
        let mut progress = Vec::new();

        let result = collect_image_entries(
            &[first.clone(), second.clone(), third.clone()],
            &hasher,
            &size_reader,
            0,
            &cancellation,
            |idx| progress.push(idx),
        )
        .unwrap();

        assert_eq!(progress, vec![0, 1, 2]);
        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.entries[0].path, first);
        assert_eq!(result.entries[0].hash, 101);
        assert_eq!(result.entries[1].path, third);
        assert_eq!(result.entries[1].hash, 303);

        let (_messages, warnings) = result.warnings.into_parts();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].path, second);
        assert_eq!(warnings[0].code, crate::application::RunWarning::FILE_ISSUE_CODE);
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
}
