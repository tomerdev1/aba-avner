use aba_avner::domain::ImageEntry;
use aba_avner::fs_ops::copy_unique_images;
use aba_avner::grouping::group_by_similarity;
use aba_avner::hashing::compute_dhash;
use aba_avner::scan::collect_image_paths;
use image::DynamicImage;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use walkdir::WalkDir;

const BACKUP_DIR: &str = "../auto-tests/small_images_limiter";
const SYNTHETIC_DIR: &str = "../auto-tests/synthetic-images";
const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "tiff", "tif", "webp"];

fn collect_images_manual(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = match entry {
            Ok(value) => value,
            Err(_) => continue,
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let is_image = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| IMAGE_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
            .unwrap_or(false);
        if is_image {
            paths.push(path.to_path_buf());
        }
    }
    paths
}

#[test]
fn backup_dataset_scan_matches_manual_filter() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(BACKUP_DIR);
    assert!(root.exists(), "backup dataset missing at {BACKUP_DIR}");

    let manual = collect_images_manual(&root);
    let scanned = collect_image_paths(&root).expect("scan failed");

    assert_eq!(manual.len(), scanned.len());
    assert!(!scanned.is_empty(), "expected at least one image");

    let scanned_set: HashSet<_> = scanned.iter().collect();
    assert!(
        scanned_set
            .iter()
            .any(|path| path.ends_with("large_b.png")),
        "expected large_b.png in scan results"
    );
    assert!(
        scanned_set
            .iter()
            .all(|path| !path.ends_with("zoom_amd64.deb")),
        "non-image file should not be scanned"
    );
    assert!(
        scanned_set.iter().all(|path| !path
            .ends_with("55cd92b2-ab0b-4d2c-9c72-c4d7f32c10f4_695a4171ac33a1001b9da198_logs.zip")),
        "archive should not be scanned"
    );
}

#[test]
fn hashing_and_grouping_detect_duplicates_within_threshold() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(BACKUP_DIR);
    let source = root.join("large_b.png");
    assert!(source.exists(), "missing test image large_b.png");

    let temp = tempdir().expect("tempdir");
    let original = temp.path().join("original.png");
    let duplicate = temp.path().join("duplicate.png");
    let near_duplicate = temp.path().join("near.png");

    fs::copy(&source, &original).expect("copy original");
    fs::copy(&source, &duplicate).expect("copy duplicate");
    write_near_duplicate(&source, &near_duplicate);

    let paths = collect_image_paths(temp.path()).expect("scan temp dir");
    let mut entries = Vec::new();
    for path in &paths {
        let image = image::open(path).expect("load image");
        entries.push(aba_avner::domain::ImageEntry {
            path: path.clone(),
            hash: compute_dhash(&image),
            size_bytes: fs::metadata(path).expect("stat image").len(),
        });
    }

    let groups = group_by_similarity(&entries, 10);
    let mut found = false;

    for group in &groups {
        let names: HashSet<_> = group
            .iter()
            .filter_map(|entry| entry.path.file_name().and_then(|n| n.to_str()))
            .collect();
        if names.contains("original.png")
            && names.contains("duplicate.png")
            && names.contains("near.png")
        {
            found = true;
            break;
        }
    }

    assert!(found, "expected grouped near duplicates in same cluster");

    let output = temp.path().join("output");
    fs::create_dir_all(&output).expect("create output");
    let unique_paths: Vec<PathBuf> = groups
        .iter()
        .filter_map(|group| group.first().map(|entry| entry.path.clone()))
        .collect();

    let copied = copy_unique_images(&unique_paths, &output).expect("copy uniques");
    assert_eq!(copied.len(), unique_paths.len());
    for path in copied {
        assert!(path.exists(), "expected copied file to exist");
    }
}

#[test]
fn scan_recurses_into_subdirectories() {
    let temp = tempdir().expect("tempdir");
    let nested_dir = temp.path().join("nested");
    fs::create_dir_all(&nested_dir).expect("create nested dir");

    let image_path = nested_dir.join("nested.png");
    let mut image = DynamicImage::new_rgba8(20, 20).to_rgba8();
    for x in 0..20 {
        for y in 0..20 {
            image.put_pixel(x, y, image::Rgba([120, 40, 200, 255]));
        }
    }
    DynamicImage::ImageRgba8(image)
        .save(&image_path)
        .expect("save nested image");

    let paths = collect_image_paths(temp.path()).expect("scan nested");
    assert!(paths.iter().any(|path| path.ends_with("nested.png")));
}

#[test]
fn synthetic_dataset_scans_hashes_and_groups_consistently() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(SYNTHETIC_DIR);
    assert!(root.exists(), "synthetic dataset missing at {SYNTHETIC_DIR}");

    let manual = collect_images_manual(&root);
    let mut scanned = collect_image_paths(&root).expect("scan synthetic dataset");
    scanned.sort();

    assert_eq!(manual.len(), scanned.len());
    assert!(
        scanned.len() >= 40,
        "expected a curated synthetic dataset, found {} images",
        scanned.len()
    );

    let sample_paths: Vec<PathBuf> = scanned.iter().take(40).cloned().collect();
    assert_eq!(sample_paths.len(), 40);

    let entries = hash_entries(&sample_paths);
    assert_eq!(
        entries.len(),
        sample_paths.len(),
        "sample synthetic images should hash"
    );

    let unique_hashes: HashSet<u64> = entries.iter().map(|entry| entry.hash).collect();
    assert!(
        unique_hashes.len() > entries.len() / 2,
        "too many hash collisions: {} unique hashes for {} images",
        unique_hashes.len(),
        entries.len()
    );

    let exact_groups = group_by_similarity(&entries, 0);
    let exact_group_entry_count: usize = exact_groups.iter().map(Vec::len).sum();
    assert_eq!(exact_group_entry_count, entries.len());
    assert_eq!(exact_groups.len(), unique_hashes.len());

    let similar_groups = group_by_similarity(&entries, 10);
    let similar_group_entry_count: usize = similar_groups.iter().map(Vec::len).sum();
    let largest_group = similar_groups.iter().map(Vec::len).max().unwrap_or(0);
    assert_eq!(similar_group_entry_count, entries.len());
    assert!(
        similar_groups.len() > 1,
        "similarity grouping should not collapse the sample synthetic dataset into one cluster"
    );
    assert!(
        largest_group < entries.len(),
        "similarity grouping should not merge all sampled images into one group"
    );
}

#[test]
#[ignore = "slow full-dataset smoke test"]
fn synthetic_dataset_full_hash_smoke() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(SYNTHETIC_DIR);
    assert!(root.exists(), "synthetic dataset missing at {SYNTHETIC_DIR}");

    let mut scanned = collect_image_paths(&root).expect("scan synthetic dataset");
    scanned.sort();
    assert!(
        scanned.len() >= 40,
        "expected a curated synthetic dataset, found {} images",
        scanned.len()
    );

    let entries = hash_entries(&scanned);
    assert_eq!(
        entries.len(),
        scanned.len(),
        "all synthetic images should hash"
    );
}

fn write_near_duplicate(source: &Path, dest: &Path) {
    let mut image = image::open(source).expect("open source image").to_rgba8();
    let (width, height) = image.dimensions();
    let max_x = width.min(5);
    let max_y = height.min(5);
    for x in 0..max_x {
        for y in 0..max_y {
            let mut pixel = *image.get_pixel(x, y);
            pixel.0[0] = pixel.0[0].saturating_add(12);
            image.put_pixel(x, y, pixel);
        }
    }
    DynamicImage::ImageRgba8(image)
        .save(dest)
        .expect("save near duplicate");
}

fn hash_entries(paths: &[PathBuf]) -> Vec<ImageEntry> {
    let mut entries = Vec::with_capacity(paths.len());
    for path in paths {
        let image = image::open(path).unwrap_or_else(|error| {
            panic!("failed to load {}: {error}", path.display());
        });
        entries.push(ImageEntry {
            path: path.clone(),
            hash: compute_dhash(&image),
            size_bytes: std::fs::metadata(path).expect("stat image").len(),
        });
    }
    entries
}
