use crate::application::{RunDedupeRequest, RunDedupeResult};
use crate::errors::DedupeError;
use log::{error, info, warn, LevelFilter, Metadata, Record, SetLoggerError};
use std::io::{self, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

use super::RunObserver;

const RUN_LOG_TARGET: &str = "aba_avner::dedupe_run";
static RUN_COUNTER: AtomicU64 = AtomicU64::new(1);
static LOGGER_INIT: OnceLock<()> = OnceLock::new();
static STDERR_LOGGER: SimpleStderrLogger = SimpleStderrLogger;

pub struct LogRunObserver {
    run_id: String,
}

impl LogRunObserver {
    pub fn new() -> Self {
        Self::with_run_id(next_run_id())
    }

    pub fn with_run_id(run_id: String) -> Self {
        Self { run_id }
    }
}

impl RunObserver for LogRunObserver {
    fn on_started(&mut self, request: &RunDedupeRequest) {
        info!(
            target: RUN_LOG_TARGET,
            "event=run_started run_id={} input_dir={} output_dir={} similarity_threshold={} min_image_size_bytes={} filter_existing_output={}",
            self.run_id,
            request.input_dir.display(),
            request.output_dir.display(),
            request.similarity_threshold,
            request.min_image_size_bytes,
            request.filter_existing_output
        );
    }

    fn on_succeeded(&mut self, request: &RunDedupeRequest, result: &RunDedupeResult) {
        info!(
            target: RUN_LOG_TARGET,
            "event=run_succeeded run_id={} input_dir={} output_dir={} total_images={} unique_images={} duplicate_images={} warnings={}",
            self.run_id,
            request.input_dir.display(),
            result.output_dir.display(),
            result.total_images,
            result.unique_images,
            result.duplicate_images,
            result.warnings.len()
        );
    }

    fn on_failed(&mut self, request: &RunDedupeRequest, error: &DedupeError) {
        match error {
            DedupeError::Cancelled => warn!(
                target: RUN_LOG_TARGET,
                "event=run_cancelled run_id={} input_dir={} output_dir={} code={}",
                self.run_id,
                request.input_dir.display(),
                request.output_dir.display(),
                error.code()
            ),
            _ => error!(
                target: RUN_LOG_TARGET,
                "event=run_failed run_id={} input_dir={} output_dir={} code={} message={}",
                self.run_id,
                request.input_dir.display(),
                request.output_dir.display(),
                error.code(),
                error
            ),
        }
    }
}

pub fn init_logging() -> Result<(), SetLoggerError> {
    if LOGGER_INIT.get().is_some() {
        return Ok(());
    }

    log::set_logger(&STDERR_LOGGER)?;
    log::set_max_level(level_filter_from_env());
    let _ = LOGGER_INIT.set(());
    Ok(())
}

pub fn next_run_id() -> String {
    format!("run-{}", RUN_COUNTER.fetch_add(1, Ordering::Relaxed))
}

pub fn log_cancellation_requested(run_id: &str, was_already_cancelled: bool) {
    let event = if was_already_cancelled {
        "cancel_requested_duplicate"
    } else {
        "cancel_requested"
    };

    info!(target: RUN_LOG_TARGET, "event={} run_id={}", event, run_id);
}

fn level_filter_from_env() -> LevelFilter {
    let raw_level = std::env::var("ABA_AVNER_LOG")
        .ok()
        .or_else(|| std::env::var("RUST_LOG").ok())
        .unwrap_or_else(|| "info".to_string());

    parse_level_filter(&raw_level)
}

fn parse_level_filter(raw_level: &str) -> LevelFilter {
    match raw_level.trim().to_ascii_lowercase().as_str() {
        "error" => LevelFilter::Error,
        "warn" | "warning" => LevelFilter::Warn,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        "off" => LevelFilter::Off,
        _ => LevelFilter::Info,
    }
}

struct SimpleStderrLogger;

impl log::Log for SimpleStderrLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let _ = writeln!(
            io::stderr(),
            "level={} target={} {}",
            record.level(),
            record.target(),
            record.args()
        );
    }

    fn flush(&self) {}
}

#[cfg(test)]
mod tests {
    use super::{log_cancellation_requested, next_run_id, parse_level_filter};
    use crate::errors::DedupeError;
    use log::LevelFilter;

    #[test]
    fn next_run_id_is_prefixed_and_unique() {
        let first = next_run_id();
        let second = next_run_id();

        assert!(first.starts_with("run-"));
        assert!(second.starts_with("run-"));
        assert_ne!(first, second);
    }

    #[test]
    fn dedupe_error_code_matches_frontend_contract() {
        assert_eq!(
            DedupeError::InputOutputConflict("conflict".to_string()).code(),
            "input_output_conflict"
        );
    }

    #[test]
    fn parse_level_filter_defaults_to_info_for_unknown_values() {
        assert_eq!(parse_level_filter(""), LevelFilter::Info);
        assert_eq!(parse_level_filter("banana"), LevelFilter::Info);
    }

    #[test]
    fn cancellation_logging_helper_is_callable() {
        log_cancellation_requested("run-42", false);
        log_cancellation_requested("run-42", true);
    }
}
