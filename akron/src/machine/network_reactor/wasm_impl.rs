use super::*;

pub(crate) struct NetworkReactor;

impl NetworkReactor {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn submit(
        &self,
        _request: NativeNetworkRequest,
        _cancelled: Arc<AtomicBool>,
    ) -> Result<JobReceiver, RuntimeError> {
        Err(RuntimeError::capability_denied(
            "network reactor is unavailable without an explicit WASM host adapter",
        ))
    }

    pub(crate) fn close_silently(&self, _handle: u32) {}

    pub(crate) fn wake(&self) {}
}
