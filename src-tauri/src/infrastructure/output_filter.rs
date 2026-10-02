use crate::application::{
    CancellationToken, ComposedOutputDirectoryFilter, OutputDirectoryFilter, OutputFilterResult,
};
use crate::domain::ImageEntry;
use crate::errors::Result;
use std::path::Path;

use super::{FsFileSizeReader, FsImageHasher, FsImageScanner};

pub struct FsOutputDirectoryFilter;

impl OutputDirectoryFilter for FsOutputDirectoryFilter {
    fn filter_existing_output(
        &self,
        entries: Vec<ImageEntry>,
        output_dir: &Path,
        similarity_threshold: u32,
        cancellation: &dyn CancellationToken,
    ) -> Result<OutputFilterResult> {
        let filter =
            ComposedOutputDirectoryFilter::new(&FsImageScanner, &FsImageHasher, &FsFileSizeReader);
        filter.filter_existing_output(entries, output_dir, similarity_threshold, cancellation)
    }
}
