use aba_avner::application::{
    AppLocale, CancellationToken, ImageHasher, ImageScanner, OutputWriter,
};
use aba_avner::errors::DedupeError;
use aba_avner::infrastructure::{FsImageHasher, FsImageScanner, FsOutputWriter};
use image::{DynamicImage, ImageBuffer, Rgba};
use std::cell::Cell;
use std::fs;
use tempfile::tempdir;

struct CancelAfterChecks {
    remaining_active_checks: Cell<usize>,
}

impl CancelAfterChecks {
    fn new(remaining_active_checks: usize) -> Self {
        Self {
            remaining_active_checks: Cell::new(remaining_active_checks),
        }
    }
}

impl CancellationToken for CancelAfterChecks {
    fn is_cancelled(&self) -> bool {
        let remaining = self.remaining_active_checks.get();
        if remaining == 0 {
            true
        } else {
            self.remaining_active_checks.set(remaining - 1);
            false
        }
    }
}

#[test]
fn fs_image_scanner_filters_non_images_and_recurses() {
    let temp = tempdir().unwrap();
    let nested = temp.path().join("nested");
    fs::create_dir_all(&nested).unwrap();

    let image_path = nested.join("photo.png");
    let text_path = nested.join("notes.txt");
    create_image(&image_path, [120, 60, 30, 255]);
    fs::write(&text_path, b"ignore me").unwrap();

    let scanner = FsImageScanner;
    let paths = scanner.collect_image_paths(temp.path()).unwrap();

    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0], image_path);
}

#[test]
fn fs_image_scanner_rejects_missing_roots() {
    let temp = tempdir().unwrap();
    let missing = temp.path().join("missing");

    let scanner = FsImageScanner;
    let error = scanner.collect_image_paths(&missing).unwrap_err();

    match error {
        DedupeError::InvalidInputDir(path) => {
            assert!(path.contains("missing"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn fs_image_scanner_honors_cancellation_during_traversal() {
    let temp = tempdir().unwrap();
    let nested = temp.path().join("nested");
    fs::create_dir_all(&nested).unwrap();
    create_image(&temp.path().join("first.png"), [120, 60, 30, 255]);
    create_image(&nested.join("second.png"), [30, 120, 60, 255]);
    create_image(&nested.join("third.png"), [60, 30, 120, 255]);

    let scanner = FsImageScanner;
    let cancellation = CancelAfterChecks::new(1);
    let error = scanner
        .collect_image_paths_with_cancellation(temp.path(), &cancellation)
        .unwrap_err();

    assert!(matches!(error, DedupeError::Cancelled));
}

#[test]
fn fs_output_writer_creates_output_and_avoids_overwrites() {
    let temp = tempdir().unwrap();
    let source_a = temp.path().join("a/photo.png");
    let source_b = temp.path().join("b/photo.png");
    let output_dir = temp.path().join("output");

    fs::create_dir_all(source_a.parent().unwrap()).unwrap();
    fs::create_dir_all(source_b.parent().unwrap()).unwrap();
    create_image(&source_a, [255, 0, 0, 255]);
    create_image(&source_b, [0, 255, 0, 255]);

    let writer = FsOutputWriter;
    writer.ensure_output_dir(&output_dir).unwrap();
    let mut progress = Vec::new();
    let cancellation = CancelAfterChecks::new(usize::MAX);
    let copied = writer
        .copy_unique_images(
            &[source_a, source_b],
            &output_dir,
            &cancellation,
            |current| {
                progress.push(current);
            },
        )
        .unwrap();

    assert_eq!(copied[0], output_dir.join("photo.png"));
    assert_eq!(copied[1], output_dir.join("photo-1.png"));
    assert!(copied.iter().all(|path| path.exists()));
    assert_eq!(progress, vec![1, 2]);
}

#[test]
fn fs_output_writer_exports_duplicate_groups_into_non_unique_images_folder() {
    let temp = tempdir().unwrap();
    let output_dir = temp.path().join("output");
    let source_a = temp.path().join("a/photo.png");
    let source_b = temp.path().join("b/photo.png");
    let unique = temp.path().join("unique.png");

    fs::create_dir_all(source_a.parent().unwrap()).unwrap();
    fs::create_dir_all(source_b.parent().unwrap()).unwrap();
    create_image(&source_a, [255, 0, 0, 255]);
    create_image(&source_b, [0, 255, 0, 255]);
    create_image(&unique, [0, 0, 255, 255]);

    let writer = FsOutputWriter;
    writer.ensure_output_dir(&output_dir).unwrap();
    let cancellation = CancelAfterChecks::new(usize::MAX);
    writer
        .copy_non_unique_groups(
            &[
                vec![
                    aba_avner::domain::ImageEntry {
                        path: source_a.clone(),
                        hash: 1,
                        size_bytes: 0,
                    },
                    aba_avner::domain::ImageEntry {
                        path: source_b.clone(),
                        hash: 1,
                        size_bytes: 0,
                    },
                ],
                vec![aba_avner::domain::ImageEntry {
                    path: unique,
                    hash: 2,
                    size_bytes: 0,
                }],
            ],
            std::slice::from_ref(&source_a),
            &output_dir,
            AppLocale::En,
            true,
            &cancellation,
        )
        .unwrap();

    let group_dir = output_dir.join("similar image groups").join("group-001");
    assert!(group_dir.join("photo__kept.png").exists());
    assert!(group_dir.join("photo.png").exists());
    assert!(!output_dir
        .join("similar image groups")
        .join("group-002")
        .exists());
}

#[test]
fn fs_output_writer_rejects_file_as_output_directory() {
    let temp = tempdir().unwrap();
    let output_file = temp.path().join("output-file");
    fs::write(&output_file, b"not a directory").unwrap();

    let writer = FsOutputWriter;
    let error = writer.ensure_output_dir(&output_file).unwrap_err();

    match error {
        DedupeError::InvalidOutputDir(path) => {
            assert!(path.contains("output-file"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn fs_output_writer_skips_similar_group_export_when_disabled() {
    let temp = tempdir().unwrap();
    let output_dir = temp.path().join("output");
    let source_a = temp.path().join("a/photo.png");
    let source_b = temp.path().join("b/photo.png");
    let stale = output_dir
        .join("similar image groups")
        .join("group-001")
        .join("stale.txt");

    fs::create_dir_all(source_a.parent().unwrap()).unwrap();
    fs::create_dir_all(source_b.parent().unwrap()).unwrap();
    fs::create_dir_all(stale.parent().unwrap()).unwrap();
    create_image(&source_a, [255, 0, 0, 255]);
    create_image(&source_b, [0, 255, 0, 255]);
    fs::write(&stale, b"stale").unwrap();

    let writer = FsOutputWriter;
    writer.ensure_output_dir(&output_dir).unwrap();
    let cancellation = CancelAfterChecks::new(usize::MAX);

    writer
        .copy_non_unique_groups(
            &[vec![
                aba_avner::domain::ImageEntry {
                    path: source_a,
                    hash: 1,
                    size_bytes: 0,
                },
                aba_avner::domain::ImageEntry {
                    path: source_b,
                    hash: 1,
                    size_bytes: 0,
                },
            ]],
            &[],
            &output_dir,
            AppLocale::En,
            false,
            &cancellation,
        )
        .unwrap();

    assert_eq!(fs::read(&stale).unwrap(), b"stale");
}

#[test]
fn fs_output_writer_cancels_unique_copy_and_rolls_back_partial_output() {
    let temp = tempdir().unwrap();
    let source_a = temp.path().join("a/photo-a.png");
    let source_b = temp.path().join("b/photo-b.png");
    let output_dir = temp.path().join("output");

    fs::create_dir_all(source_a.parent().unwrap()).unwrap();
    fs::create_dir_all(source_b.parent().unwrap()).unwrap();
    create_image(&source_a, [255, 0, 0, 255]);
    create_image(&source_b, [0, 255, 0, 255]);

    let writer = FsOutputWriter;
    writer.ensure_output_dir(&output_dir).unwrap();
    let cancellation = CancelAfterChecks::new(1);
    let mut progress = Vec::new();

    let error = writer
        .copy_unique_images(
            &[source_a, source_b],
            &output_dir,
            &cancellation,
            |current| {
                progress.push(current);
            },
        )
        .unwrap_err();

    assert!(matches!(error, DedupeError::Cancelled));
    assert_eq!(progress, vec![1]);
    assert_eq!(fs::read_dir(&output_dir).unwrap().count(), 0);
}

#[test]
fn fs_output_writer_cancels_duplicate_group_refresh_without_replacing_existing_output() {
    let temp = tempdir().unwrap();
    let output_dir = temp.path().join("output");
    let source_a = temp.path().join("a/photo-a.png");
    let source_b = temp.path().join("b/photo-b.png");
    let stale = output_dir
        .join("similar image groups")
        .join("group-001")
        .join("stale.txt");

    fs::create_dir_all(source_a.parent().unwrap()).unwrap();
    fs::create_dir_all(source_b.parent().unwrap()).unwrap();
    fs::create_dir_all(stale.parent().unwrap()).unwrap();
    create_image(&source_a, [255, 0, 0, 255]);
    create_image(&source_b, [0, 255, 0, 255]);
    fs::write(&stale, b"stale").unwrap();

    let writer = FsOutputWriter;
    writer.ensure_output_dir(&output_dir).unwrap();
    let cancellation = CancelAfterChecks::new(1);

    let error = writer
        .copy_non_unique_groups(
            &[vec![
                aba_avner::domain::ImageEntry {
                    path: source_a,
                    hash: 1,
                    size_bytes: 0,
                },
                aba_avner::domain::ImageEntry {
                    path: source_b,
                    hash: 1,
                    size_bytes: 0,
                },
            ]],
            &[],
            &output_dir,
            AppLocale::En,
            true,
            &cancellation,
        )
        .unwrap_err();

    assert!(matches!(error, DedupeError::Cancelled));
    assert_eq!(fs::read(&stale).unwrap(), b"stale");
}

#[test]
fn fs_image_hasher_loads_hashes_and_reports_bad_images() {
    let temp = tempdir().unwrap();
    let valid = temp.path().join("valid.png");
    let corrupt = temp.path().join("corrupt.png");
    create_image(&valid, [10, 20, 30, 255]);
    fs::write(&corrupt, b"broken").unwrap();

    let hasher = FsImageHasher;
    hasher.load_and_hash(&valid).unwrap();
    let error = hasher.load_and_hash(&corrupt).unwrap_err();

    assert_eq!(error.path, corrupt);
    assert!(error.to_warning_message().contains("corrupt.png"));
}

fn create_image(path: &std::path::Path, rgba: [u8; 4]) {
    let image: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_pixel(32, 32, Rgba(rgba));
    DynamicImage::ImageRgba8(image).save(path).unwrap();
}
