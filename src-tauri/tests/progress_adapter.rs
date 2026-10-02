use aba_avner::application::{ProgressPublisher, ProgressUpdate};
use aba_avner::infrastructure::{
    EventProgressPublisher, ProgressEventEmitter, DEDUPE_PROGRESS_EVENT,
};
use aba_avner::presentation::contracts::ProgressEvent;

#[test]
fn event_progress_publisher_emits_stable_event_name_and_payload_mapping() {
    let emitter = RecordingEmitter::default();
    let mut publisher = EventProgressPublisher::new(emitter, "run-123");

    publisher.emit(ProgressUpdate::new("hash", 2, 5, "Hashing images"));

    let emitter = publisher.into_inner();
    assert_eq!(emitter.events.len(), 1);
    assert_eq!(emitter.events[0].0, DEDUPE_PROGRESS_EVENT);
    assert_eq!(
        emitter.events[0].1,
        ProgressEvent {
            run_id: "run-123".to_string(),
            stage: "hash".to_string(),
            current: 2,
            total: 5,
            message: "Hashing images".to_string(),
        }
    );
}

#[test]
fn event_progress_publisher_ignores_emitter_failures() {
    let emitter = FailingEmitter::default();
    let mut publisher = EventProgressPublisher::new(emitter, "run-123");

    publisher.emit(ProgressUpdate::new("scan", 0, 1, "Scanning for images"));
    publisher.emit(ProgressUpdate::new("copy", 1, 1, "Copy complete"));

    let emitter = publisher.into_inner();
    assert_eq!(emitter.attempted_events.len(), 2);
    assert_eq!(emitter.attempted_events[0].0, DEDUPE_PROGRESS_EVENT);
    assert_eq!(emitter.attempted_events[0].1.run_id, "run-123");
    assert_eq!(emitter.attempted_events[1].1.stage, "copy");
}

#[derive(Default)]
struct RecordingEmitter {
    events: Vec<(&'static str, ProgressEvent)>,
}

impl ProgressEventEmitter for RecordingEmitter {
    fn emit_progress(&mut self, event: &'static str, payload: ProgressEvent) -> Result<(), String> {
        self.events.push((event, payload));
        Ok(())
    }
}

#[derive(Default)]
struct FailingEmitter {
    attempted_events: Vec<(&'static str, ProgressEvent)>,
}

impl ProgressEventEmitter for FailingEmitter {
    fn emit_progress(&mut self, event: &'static str, payload: ProgressEvent) -> Result<(), String> {
        self.attempted_events.push((event, payload));
        Err("simulated emit failure".to_string())
    }
}
