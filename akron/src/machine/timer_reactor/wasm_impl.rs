use super::*;

pub(crate) struct TimerReactor;

impl TimerReactor {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn submit(
        &self,
        _duration: Duration,
        _cancelled: Arc<AtomicBool>,
    ) -> Result<JobReceiver, RuntimeError> {
        Err(RuntimeError::capability_denied(
            "timer reactor is unavailable without an explicit WASM clock adapter",
        ))
    }

    pub(crate) fn wake(&self) {}
}
