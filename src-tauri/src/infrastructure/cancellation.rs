use crate::application::CancellationToken;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Clone, Default)]
pub struct AtomicCancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl AtomicCancellationToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
}

impl CancellationToken for AtomicCancellationToken {
    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::AtomicCancellationToken;
    use crate::application::CancellationToken;

    #[test]
    fn atomic_cancellation_token_flips_to_cancelled_state() {
        let token = AtomicCancellationToken::new();
        assert!(!token.is_cancelled());

        token.cancel();

        assert!(token.is_cancelled());
    }
}
