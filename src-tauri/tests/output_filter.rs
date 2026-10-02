use aba_avner::application::{
    CancellationToken, NoopCancellationToken, OutputDirectoryFilter, RunWarning,
};
use aba_avner::domain::ImageEntry;
use aba_avner::errors::DedupeError;
use aba_avner::hashing::compute_dhash;
use aba_avner::infrastructure::FsOutputDirectoryFilter;
use image::{DynamicImage, ImageBuffer, Rgba};
use std::fs::write;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn fs_output_filter_skips_similar_images_and_warns() {
    let temp = tempdir().unwrap();
    let output_dir = temp.path().join("output");
    let incoming_path = temp.path().join("incoming.png");
    let existing_path = output_dir.join("existing.png");

    std::fs::create_dir_all(&output_dir).unwrap();
    create_test_image(&incoming_path, 64, 64, [10, 20, 30, 255]);
    std::fs::copy(&incoming_path, &existing_path).unwrap();

    let filter = FsOutputDirectoryFilter;
    let result = filter
        .filter_existing_output(
            vec![ImageEntry {
                path: incoming_path.clone(),
                hash: compute_dhash(&image::open(&incoming_path).unwrap()),
                size_bytes: std::fs::metadata(&incoming_path).unwrap().len(),
            }],
            &output_dir,
            0,
            &NoopCancellationToken,
        )
        .unwrap();

    assert!(result.entries.is_empty());
    assert_eq!(result.warnings.len(), 1);
    assert_eq!(
        result.warnings[0].code,
        RunWarning::SIMILAR_IMAGE_IN_OUTPUT_CODE
    );
    assert_eq!(result.warnings[0].path, incoming_path);
}

#[test]
fn fs_output_filter_ignores_unhashable_existing_files_but_warns() {
    let temp = tempdir().unwrap();
    let output_dir = temp.path().join("output");
    let incoming_path = temp.path().join("incoming.png");
    let corrupt_existing = output_dir.join("broken.png");

    std::fs::create_dir_all(&output_dir).unwrap();
    create_pattern_image(&incoming_path, 64, 64, 0);
    write(&corrupt_existing, vec![b'x'; 2_048]).unwrap();

    let incoming = ImageEntry {
        path: incoming_path.clone(),
        hash: compute_dhash(&image::open(&incoming_path).unwrap()),
        size_bytes: std::fs::metadata(&incoming_path).unwrap().len(),
    };
    let filter = FsOutputDirectoryFilter;
    let result = filter
        .filter_existing_output(
            vec![incoming.clone()],
            &output_dir,
            0,
            &NoopCancellationToken,
        )
        .unwrap();

    assert_eq!(result.entries.len(), 1);
    assert_eq!(result.entries[0].path, incoming.path);
    assert!(result.warnings.iter().any(|warning| {
        warning.code == RunWarning::FILE_ISSUE_CODE && warning.path.ends_with("broken.png")
    }));
}

#[test]
fn fs_output_filter_honors_cancellation() {
    let temp = tempdir().unwrap();
    let output_dir = temp.path().join("output");
    let incoming_path = temp.path().join("incoming.png");

    std::fs::create_dir_all(&output_dir).unwrap();
    create_pattern_image(&incoming_path, 64, 64, 0);

    let filter = FsOutputDirectoryFilter;
    let error = filter
        .filter_existing_output(
            vec![ImageEntry {
                path: incoming_path.clone(),
                hash: compute_dhash(&image::open(&incoming_path).unwrap()),
                size_bytes: std::fs::metadata(&incoming_path).unwrap().len(),
            }],
            &output_dir,
            0,
            &AlwaysCancelled,
        )
        .unwrap_err();

    assert!(matches!(error, DedupeError::Cancelled));
}

#[test]
fn fs_output_filter_falls_back_to_warning_when_output_scan_fails() {
    let temp = tempdir().unwrap();
    let output_path = temp.path().join("missing-output");
    let incoming_path = temp.path().join("incoming.png");

    create_pattern_image(&incoming_path, 64, 64, 0);

    let incoming = ImageEntry {
        path: incoming_path.clone(),
        hash: compute_dhash(&image::open(&incoming_path).unwrap()),
        size_bytes: std::fs::metadata(&incoming_path).unwrap().len(),
    };
    let filter = FsOutputDirectoryFilter;
    let result = filter
        .filter_existing_output(
            vec![incoming.clone()],
            &output_path,
            0,
            &NoopCancellationToken,
        )
        .unwrap();

    assert_eq!(result.entries.len(), 1);
    assert_eq!(result.entries[0].path, incoming.path);
    assert_eq!(result.warnings.len(), 1);
    assert_eq!(
        result.warnings[0].code,
        RunWarning::OUTPUT_DIR_SCAN_FAILED_CODE
    );
    assert_eq!(result.warnings[0].path, output_path);
}

struct AlwaysCancelled;

impl CancellationToken for AlwaysCancelled {
    fn is_cancelled(&self) -> bool {
        true
    }
}

fn create_test_image(path: &Path, width: u32, height: u32, rgba: [u8; 4]) {
    let image: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_pixel(width, height, Rgba(rgba));
    DynamicImage::ImageRgba8(image).save(path).unwrap();
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
