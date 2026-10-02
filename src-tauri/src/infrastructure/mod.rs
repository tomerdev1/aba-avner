mod cancellation;
mod filesystem;
mod logging;
mod msix;
mod output_filter;
mod path_guard;
mod progress_adapter;
mod run_dedupe;
#[cfg(feature = "gui")]
mod tauri_progress;

pub use cancellation::AtomicCancellationToken;
pub use filesystem::{FsFileSizeReader, FsImageHasher, FsImageScanner, FsOutputWriter};
pub use logging::{init_logging, log_cancellation_requested, next_run_id, LogRunObserver};
pub use msix::is_msix_package;
pub use output_filter::FsOutputDirectoryFilter;
pub(crate) use path_guard::{normalize_input_dir, normalize_output_dir};
pub use progress_adapter::{EventProgressPublisher, ProgressEventEmitter, DEDUPE_PROGRESS_EVENT};
pub use run_dedupe::{
    export_review_with_filesystem_and_cancellation_with_run_id,
    prepare_review_with_filesystem_and_cancellation_with_run_id, run_dedupe_with_filesystem,
    run_dedupe_with_filesystem_and_cancellation, run_dedupe_with_filesystem_and_cancellation_with_run_id,
    run_dedupe_with_observer, RunObserver,
};
#[cfg(feature = "gui")]
pub use tauri_progress::TauriProgressPublisher;
