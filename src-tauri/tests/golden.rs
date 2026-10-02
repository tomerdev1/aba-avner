use aba_avner::domain::ImageEntry;
use aba_avner::grouping::group_by_similarity;
use aba_avner::hashing::compute_dhash;
use aba_avner::scan::collect_image_paths;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

const GOLDEN_DIR: &str = "../auto-tests/golden";

#[derive(Debug, Deserialize)]
struct GoldenManifest {
    similarity_threshold: u32,
    expected_hashed_images: usize,
    expected_groups: Vec<Vec<String>>,
}

#[test]
fn golden_dataset_matches_expected_similarity_groups() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(GOLDEN_DIR);
    let input_dir = root.join("input");
    let manifest = load_manifest(&root);

    let mut paths = collect_image_paths(&input_dir).expect("scan golden input");
    paths.sort();

    let (entries, warnings) = hash_entries_with_warnings(&paths);

    assert_eq!(entries.len(), manifest.expected_hashed_images);
    assert_eq!(warnings.len(), 1);
    assert!(warnings
        .iter()
        .any(|warning| warning.contains("corrupt_input.jpg")));

    let groups = normalize_groups(group_by_similarity(&entries, manifest.similarity_threshold));
    let mut expected_groups = manifest.expected_groups;
    expected_groups.sort();

    assert_eq!(groups, expected_groups);
}

fn load_manifest(root: &Path) -> GoldenManifest {
    let raw = fs::read_to_string(root.join("manifest.json")).expect("read manifest");
    serde_json::from_str(&raw).expect("parse manifest")
}

fn hash_entries_with_warnings(paths: &[PathBuf]) -> (Vec<ImageEntry>, Vec<String>) {
    let mut entries = Vec::new();
    let mut warnings = Vec::new();

    for path in paths {
        match image::open(path) {
            Ok(image) => entries.push(ImageEntry {
                path: path.clone(),
                hash: compute_dhash(&image),
                size_bytes: std::fs::metadata(path).expect("stat image").len(),
            }),
            Err(error) => warnings.push(format!("{}: {}", path.display(), error)),
        }
    }

    (entries, warnings)
}

fn normalize_groups(groups: Vec<Vec<ImageEntry>>) -> Vec<Vec<String>> {
    let mut normalized: Vec<Vec<String>> = groups
        .into_iter()
        .map(|group| {
            let mut names: Vec<String> = group
                .into_iter()
                .map(|entry| {
                    entry
                        .path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .into_owned()
                })
                .collect();
            names.sort();
            names
        })
        .collect();
    normalized.sort();
    normalized
}
