use aba_avner::application::{ProgressPublisher, ProgressUpdate, RunDedupeRequest};
use aba_avner::infrastructure::{init_logging, run_dedupe_with_filesystem};
use aba_avner::scan::collect_image_paths;
use std::path::PathBuf;
use tempfile::tempdir;

struct NoopPublisher;

impl ProgressPublisher for NoopPublisher {
    fn emit(&mut self, _update: ProgressUpdate) {}
}

#[test]
#[ignore = "manual timing baseline for optimization work"]
fn golden_hash_timing_baseline() {
    run_fixture("golden/input", 0, false);
}

#[test]
#[ignore = "manual timing baseline for optimization work"]
fn small_images_limiter_hash_timing_baseline() {
    run_fixture("small_images_limiter", 1_000_000, false);
}

#[test]
#[ignore = "manual timing baseline for optimization work"]
fn golden_existing_output_filter_timing_baseline() {
    run_fixture_with_seeded_output("golden", "input", "output_seed", 0, true);
}

fn run_fixture(fixture: &str, min_image_size_bytes: u64, filter_existing_output: bool) {
    let _ = init_logging();

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("auto-tests")
        .join(fixture);
    assert!(root.exists(), "fixture missing at {}", root.display());

    let temp = tempdir().unwrap();
    let output_dir = temp.path().join("output");
    let mut publisher = NoopPublisher;

    let request = RunDedupeRequest::new(
        root.display().to_string(),
        output_dir.display().to_string(),
        10,
        min_image_size_bytes,
        filter_existing_output,
    )
    .unwrap();

    let result = run_dedupe_with_filesystem(request, &mut publisher).unwrap();
    assert!(result.total_images > 0);
}

fn run_fixture_with_seeded_output(
    fixture_root: &str,
    input_subdir: &str,
    output_seed_subdir: &str,
    min_image_size_bytes: u64,
    filter_existing_output: bool,
) {
    let _ = init_logging();

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("auto-tests")
        .join(fixture_root);
    let input_dir = root.join(input_subdir);
    let output_seed_dir = root.join(output_seed_subdir);
    assert!(input_dir.exists(), "fixture missing at {}", input_dir.display());
    assert!(
        output_seed_dir.exists(),
        "output seed missing at {}",
        output_seed_dir.display()
    );

    let temp = tempdir().unwrap();
    let output_dir = temp.path().join("output");
    std::fs::create_dir_all(&output_dir).unwrap();
    for path in collect_image_paths(&output_seed_dir).unwrap() {
        let filename = path.file_name().unwrap();
        std::fs::copy(&path, output_dir.join(filename)).unwrap();
    }

    let mut publisher = NoopPublisher;
    let request = RunDedupeRequest::new(
        input_dir.display().to_string(),
        output_dir.display().to_string(),
        10,
        min_image_size_bytes,
        filter_existing_output,
    )
    .unwrap();

    let result = run_dedupe_with_filesystem(request, &mut publisher).unwrap();
    assert!(result.total_images > 0);
}
