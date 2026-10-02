use crate::application::RunDedupeRequest;
use crate::errors::TauriError;
use crate::infrastructure::{
    export_review_with_filesystem_and_cancellation_with_run_id, is_msix_package,
    log_cancellation_requested, next_run_id,
    prepare_review_with_filesystem_and_cancellation_with_run_id,
    run_dedupe_with_filesystem_and_cancellation_with_run_id, AtomicCancellationToken,
    TauriProgressPublisher,
};
use crate::presentation::contracts::{
    DedupeConfig, DedupeResult, ExportSelectionsConfig, ReviewGroupsResult,
};
use std::sync::{Arc, Mutex};
use tauri::{State, Window};

#[derive(Clone)]
struct ActiveRun {
    token: AtomicCancellationToken,
    run_id: String,
}

#[derive(Clone, Default)]
pub struct RunCancellationState {
    active_run: Arc<Mutex<Option<ActiveRun>>>,
}

impl RunCancellationState {
    fn start_run(&self, run_id: String) -> std::result::Result<ActiveRun, TauriError> {
        let mut active_run = self
            .active_run
            .lock()
            .expect("run cancellation state lock poisoned");
        if active_run.is_some() {
            return Err(TauriError::new(
                TauriError::RUN_IN_PROGRESS_CODE,
                "A dedupe run is already in progress",
            ));
        }

        let run = ActiveRun {
            token: AtomicCancellationToken::new(),
            run_id,
        };
        *active_run = Some(run.clone());
        Ok(run)
    }

    fn finish_run(&self) {
        let mut active_run = self
            .active_run
            .lock()
            .expect("run cancellation state lock poisoned");
        *active_run = None;
    }

    fn cancel_active_run(&self) -> bool {
        let active_run = self
            .active_run
            .lock()
            .expect("run cancellation state lock poisoned");
        if let Some(run) = active_run.as_ref() {
            let was_already_cancelled =
                crate::application::CancellationToken::is_cancelled(&run.token);
            log_cancellation_requested(&run.run_id, was_already_cancelled);
            run.token.cancel();
            true
        } else {
            false
        }
    }
}

#[tauri::command]
pub async fn run_dedupe(
    window: Window,
    cancellation_state: State<'_, RunCancellationState>,
    config: DedupeConfig,
) -> std::result::Result<DedupeResult, TauriError> {
    let run_id = requested_or_generated_run_id_for_dedupe(&config);
    let request = RunDedupeRequest::try_from(config).map_err(TauriError::from)?;
    let active_run = cancellation_state.start_run(run_id)?;
    let finish_guard = ActiveRunGuard::new(cancellation_state.inner().clone());

    let result =
        tauri::async_runtime::spawn_blocking(move || run_dedupe_sync(window, request, active_run))
            .await
            .map_err(|error| TauriError::unknown(error.to_string()));
    drop(finish_guard);
    result?
}

#[tauri::command]
pub async fn prepare_dedupe_review(
    window: Window,
    cancellation_state: State<'_, RunCancellationState>,
    config: DedupeConfig,
) -> std::result::Result<ReviewGroupsResult, TauriError> {
    let run_id = requested_or_generated_run_id_for_dedupe(&config);
    let request = RunDedupeRequest::try_from(config).map_err(TauriError::from)?;
    let active_run = cancellation_state.start_run(run_id)?;
    let finish_guard = ActiveRunGuard::new(cancellation_state.inner().clone());

    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut publisher = TauriProgressPublisher::new(window, active_run.run_id.clone());

        prepare_review_with_filesystem_and_cancellation_with_run_id(
            request,
            &mut publisher,
            &active_run.token,
            active_run.run_id,
        )
        .map(ReviewGroupsResult::from)
        .map_err(TauriError::from)
    })
    .await
    .map_err(|error| TauriError::unknown(error.to_string()));
    drop(finish_guard);
    result?
}

#[tauri::command]
pub async fn export_dedupe_review(
    window: Window,
    cancellation_state: State<'_, RunCancellationState>,
    config: ExportSelectionsConfig,
) -> std::result::Result<DedupeResult, TauriError> {
    let run_id = requested_or_generated_run_id_for_export(&config);
    let request =
        crate::application::ExportSelectionsRequest::try_from(config).map_err(TauriError::from)?;
    let active_run = cancellation_state.start_run(run_id)?;
    let finish_guard = ActiveRunGuard::new(cancellation_state.inner().clone());

    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut publisher = TauriProgressPublisher::new(window, active_run.run_id.clone());

        export_review_with_filesystem_and_cancellation_with_run_id(
            request,
            &mut publisher,
            &active_run.token,
            active_run.run_id,
        )
        .map(DedupeResult::from)
        .map_err(TauriError::from)
    })
    .await
    .map_err(|error| TauriError::unknown(error.to_string()));
    drop(finish_guard);
    result?
}

#[tauri::command]
pub fn cancel_dedupe(cancellation_state: State<'_, RunCancellationState>) -> bool {
    cancellation_state.cancel_active_run()
}

/// Whether this install is an MSIX package (Store install, updates via
/// Windows Update) rather than the unpackaged NSIS distribution (updates via
/// `tauri-plugin-updater`). The frontend uses this to skip the self-update
/// check for MSIX installs, where it would be pointless at best.
#[tauri::command]
pub fn is_msix_install() -> bool {
    is_msix_package()
}

fn run_dedupe_sync(
    window: Window,
    request: RunDedupeRequest,
    active_run: ActiveRun,
) -> std::result::Result<DedupeResult, TauriError> {
    let mut publisher = TauriProgressPublisher::new(window, active_run.run_id.clone());

    run_dedupe_with_filesystem_and_cancellation_with_run_id(
        request,
        &mut publisher,
        &active_run.token,
        active_run.run_id,
    )
    .map(DedupeResult::from)
    .map_err(TauriError::from)
}

fn requested_or_generated_run_id_for_dedupe(config: &DedupeConfig) -> String {
    requested_or_generated_run_id(&config.run_id)
}

fn requested_or_generated_run_id_for_export(config: &ExportSelectionsConfig) -> String {
    requested_or_generated_run_id(&config.run_id)
}

fn requested_or_generated_run_id(run_id: &str) -> String {
    let trimmed = run_id.trim();
    if trimmed.is_empty() {
        next_run_id()
    } else {
        trimmed.to_string()
    }
}

struct ActiveRunGuard {
    state: RunCancellationState,
}

impl ActiveRunGuard {
    fn new(state: RunCancellationState) -> Self {
        Self { state }
    }
}

impl Drop for ActiveRunGuard {
    fn drop(&mut self) {
        self.state.finish_run();
    }
}

#[cfg(test)]
mod tests {
    use crate::application::CancellationToken;
    use crate::presentation::contracts::{DedupeConfig, ExportSelectionsConfig};
    use std::collections::HashMap;

    use super::{
        requested_or_generated_run_id_for_dedupe, requested_or_generated_run_id_for_export,
        RunCancellationState,
    };

    #[test]
    fn cancellation_state_tracks_single_active_run() {
        let state = RunCancellationState::default();
        let active_run = state.start_run("run-1".to_string()).unwrap();

        assert!(!active_run.token.is_cancelled());
        assert!(state.start_run("run-2".to_string()).is_err());

        assert!(state.cancel_active_run());
        assert!(active_run.token.is_cancelled());

        state.finish_run();

        assert!(!state.cancel_active_run());
        assert!(state.start_run("run-3".to_string()).is_ok());
    }

    #[test]
    fn requested_run_id_is_used_when_present() {
        let run_id = requested_or_generated_run_id_for_dedupe(&DedupeConfig {
            input_dir: "/input".to_string(),
            output_dir: "/output".to_string(),
            similarity_threshold: 10,
            min_image_size_bytes: 0,
            filter_existing_output: true,
            run_id: "custom-run".to_string(),
            locale: "en".to_string(),
        });

        assert_eq!(run_id, "custom-run");
    }

    #[test]
    fn requested_export_run_id_is_used_when_present() {
        let run_id = requested_or_generated_run_id_for_export(&ExportSelectionsConfig {
            output_dir: "/output".to_string(),
            similarity_threshold: 10,
            filter_existing_output: true,
            export_similar_image_groups: true,
            groups: vec![],
            selections: HashMap::new(),
            run_id: "export-run".to_string(),
            locale: "en".to_string(),
        });

        assert_eq!(run_id, "export-run");
    }
}
