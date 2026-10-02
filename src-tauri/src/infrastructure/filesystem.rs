use crate::application::{
    AppLocale, CancellationToken, FileIssue, FileSizeReader, ImageHasher, ImageScanner,
    OutputWriter,
};
use crate::errors::Result;
use crate::fs_ops::{
    copy_non_unique_groups_with_cancellation, copy_unique_images_with_progress_and_cancellation,
    ensure_output_dir,
};
use crate::hashing::compute_dhash;
use crate::scan::collect_image_paths;
use std::fs;
use std::path::{Path, PathBuf};

pub struct FsImageScanner;

impl ImageScanner for FsImageScanner {
    fn collect_image_paths(&self, root: &Path) -> Result<Vec<PathBuf>> {
        collect_image_paths(root)
    }

    fn collect_image_paths_with_cancellation(
        &self,
        root: &Path,
        cancellation: &dyn CancellationToken,
    ) -> Result<Vec<PathBuf>> {
        crate::scan::collect_image_paths_with_cancellation(root, cancellation)
    }
}

pub struct FsOutputWriter;

impl OutputWriter for FsOutputWriter {
    fn ensure_output_dir(&self, path: &Path) -> Result<()> {
        ensure_output_dir(path)
    }

    fn copy_unique_images<F>(
        &self,
        paths: &[PathBuf],
        output_dir: &Path,
        cancellation: &dyn CancellationToken,
        on_progress: F,
    ) -> Result<Vec<PathBuf>>
    where
        F: FnMut(usize),
    {
        copy_unique_images_with_progress_and_cancellation(
            paths,
            output_dir,
            cancellation,
            on_progress,
        )
    }

    fn copy_non_unique_groups(
        &self,
        groups: &[Vec<crate::domain::ImageEntry>],
        selected_paths: &[PathBuf],
        output_dir: &Path,
        locale: AppLocale,
        export_similar_image_groups: bool,
        cancellation: &dyn CancellationToken,
    ) -> Result<()> {
        if !export_similar_image_groups {
            return Ok(());
        }

        copy_non_unique_groups_with_cancellation(
            groups,
            selected_paths,
            output_dir,
            locale,
            true,
            cancellation,
        )
    }
}

pub struct FsImageHasher;

impl ImageHasher for FsImageHasher {
    fn load_and_hash(&self, path: &Path) -> std::result::Result<u64, FileIssue> {
        let image = image::open(path).map_err(|error| FileIssue {
            path: path.to_path_buf(),
            detail: error.to_string(),
        })?;
        Ok(compute_dhash(&image))
    }
}

pub struct FsFileSizeReader;

impl FileSizeReader for FsFileSizeReader {
    fn file_size_bytes(&self, path: &Path) -> std::result::Result<u64, FileIssue> {
        fs::metadata(path)
            .map(|metadata| metadata.len())
            .map_err(|error| FileIssue {
                path: path.to_path_buf(),
                detail: error.to_string(),
            })
    }
}
