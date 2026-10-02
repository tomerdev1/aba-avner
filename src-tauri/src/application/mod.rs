mod contracts;
mod image_entries;
mod output_filter;
mod review;
mod run_dedupe;

pub use contracts::{
    AppLocale, DedupeReport, DedupeSummary, DuplicateReviewGroup, ExportSelectionsRequest,
    FolderBreakdownEntry, PrepareReviewResult, ProgressUpdate, RunDedupeRequest,
    RunDedupeResult, RunWarning, MAX_SIMILARITY_THRESHOLD,
};
#[derive(Default)]
pub struct NoopCancellationToken;

impl run_dedupe::CancellationToken for NoopCancellationToken {
    fn is_cancelled(&self) -> bool {
        false
    }
}

pub(crate) use output_filter::ComposedOutputDirectoryFilter;
pub use output_filter::{OutputDirectoryFilter, OutputFilterResult, WarningLog};
pub use run_dedupe::{
    run_dedupe, run_dedupe_cancellable, run_dedupe_cancellable_with_filter, run_dedupe_with_filter,
    CancellationToken, FileIssue, FileSizeReader, ImageHasher, ImageScanner, OutputWriter,
    ProgressPublisher,
};
pub use review::{export_review_selections_with_filter, prepare_review_cancellable_with_filter};
