use crate::errors::{DedupeError, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "tiff", "tif", "webp"];

pub fn collect_image_paths(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Err(DedupeError::InvalidInputDir(root.display().to_string()));
    }

    let mut paths = Vec::new();
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry.map_err(|e| DedupeError::ReadDir(e.to_string()))?;
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        if is_image_path(path) {
            paths.push(path.to_path_buf());
        }
    }

    paths.sort();

    Ok(paths)
}

pub fn collect_image_paths_with_cancellation(
    root: &Path,
    cancellation: &dyn crate::application::CancellationToken,
) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Err(DedupeError::InvalidInputDir(root.display().to_string()));
    }

    let mut paths = Vec::new();
    for entry in WalkDir::new(root).follow_links(false) {
        if cancellation.is_cancelled() {
            return Err(DedupeError::Cancelled);
        }

        let entry = entry.map_err(|e| DedupeError::ReadDir(e.to_string()))?;
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        if is_image_path(path) {
            paths.push(path.to_path_buf());
        }
    }

    paths.sort();

    Ok(paths)
}

fn is_image_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| IMAGE_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::CancellationToken;
    use std::fs;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tempfile::tempdir;

    #[test]
    fn collects_only_image_files() {
        let dir = tempdir().unwrap();
        let img_path = dir.path().join("photo.jpg");
        let txt_path = dir.path().join("notes.txt");
        fs::write(&img_path, "fake").unwrap();
        fs::write(&txt_path, "fake").unwrap();

        let results = collect_image_paths(dir.path()).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].ends_with("photo.jpg"));
    }

    #[test]
    fn collects_image_files_in_sorted_path_order() {
        let dir = tempdir().unwrap();
        let later = dir.path().join("zeta.png");
        let nested = dir.path().join("nested");
        let earlier = nested.join("alpha.png");

        fs::create_dir_all(&nested).unwrap();
        fs::write(&later, "fake").unwrap();
        fs::write(&earlier, "fake").unwrap();

        let results = collect_image_paths(dir.path()).unwrap();

        assert_eq!(results, vec![earlier, later]);
    }

    #[derive(Default)]
    struct TestCancellation {
        cancelled: AtomicBool,
    }

    impl CancellationToken for TestCancellation {
        fn is_cancelled(&self) -> bool {
            self.cancelled.load(Ordering::SeqCst)
        }
    }

    impl TestCancellation {
        fn cancel(&self) {
            self.cancelled.store(true, Ordering::SeqCst);
        }
    }

    #[test]
    fn collect_image_paths_with_cancellation_stops_before_traversing_remaining_files() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("nested");
        fs::create_dir_all(&nested).unwrap();

        let first = dir.path().join("a.png");
        let second = nested.join("b.png");
        let third = nested.join("c.png");
        fs::write(&first, "fake").unwrap();
        fs::write(&second, "fake").unwrap();
        fs::write(&third, "fake").unwrap();

        struct CancelAfterFirstCheck<'a> {
            inner: &'a TestCancellation,
            first_check_seen: AtomicBool,
        }

        impl CancellationToken for CancelAfterFirstCheck<'_> {
            fn is_cancelled(&self) -> bool {
                if !self.first_check_seen.swap(true, Ordering::SeqCst) {
                    self.inner.cancel();
                    false
                } else {
                    self.inner.is_cancelled()
                }
            }
        }

        let cancellation = TestCancellation::default();
        let wrapper = CancelAfterFirstCheck {
            inner: &cancellation,
            first_check_seen: AtomicBool::new(false),
        };

        let error = collect_image_paths_with_cancellation(dir.path(), &wrapper).unwrap_err();

        assert!(matches!(error, DedupeError::Cancelled));
    }
}
