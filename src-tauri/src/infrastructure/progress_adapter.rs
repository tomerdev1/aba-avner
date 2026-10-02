use crate::application::{ProgressPublisher, ProgressUpdate};
use crate::presentation::contracts::ProgressEvent;

pub const DEDUPE_PROGRESS_EVENT: &str = "dedupe_progress";

pub trait ProgressEventEmitter {
    fn emit_progress(&mut self, event: &'static str, payload: ProgressEvent) -> Result<(), String>;
}

pub struct EventProgressPublisher<E> {
    emitter: E,
    run_id: String,
}

impl<E> EventProgressPublisher<E> {
    pub fn new(emitter: E, run_id: impl Into<String>) -> Self {
        Self {
            emitter,
            run_id: run_id.into(),
        }
    }

    pub fn into_inner(self) -> E {
        self.emitter
    }
}

impl<E> ProgressPublisher for EventProgressPublisher<E>
where
    E: ProgressEventEmitter,
{
    fn emit(&mut self, update: ProgressUpdate) {
        let _ = self
            .emitter
            .emit_progress(DEDUPE_PROGRESS_EVENT, ProgressEvent::from_update(&self.run_id, update));
    }
}
