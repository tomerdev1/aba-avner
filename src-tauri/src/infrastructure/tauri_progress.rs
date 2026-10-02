use crate::application::{ProgressPublisher, ProgressUpdate};
use tauri::{Emitter, Window};

use super::{EventProgressPublisher, ProgressEventEmitter};

pub struct TauriProgressPublisher {
    inner: EventProgressPublisher<TauriWindowEmitter>,
}

impl TauriProgressPublisher {
    pub fn new(window: Window, run_id: impl Into<String>) -> Self {
        Self {
            inner: EventProgressPublisher::new(TauriWindowEmitter { window }, run_id),
        }
    }
}

impl ProgressPublisher for TauriProgressPublisher {
    fn emit(&mut self, update: ProgressUpdate) {
        self.inner.emit(update);
    }
}

struct TauriWindowEmitter {
    window: Window,
}

impl ProgressEventEmitter for TauriWindowEmitter {
    fn emit_progress(
        &mut self,
        event: &'static str,
        payload: crate::presentation::contracts::ProgressEvent,
    ) -> Result<(), String> {
        self.window
            .emit(event, payload)
            .map_err(|error| error.to_string())
    }
}
