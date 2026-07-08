use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Debug)]
pub(crate) struct InterruptFlag {
    interrupted: Arc<AtomicBool>,
}

impl InterruptFlag {
    pub(crate) fn new() -> Self {
        Self {
            interrupted: Arc::new(AtomicBool::new(false)),
        }
    }

    pub(crate) fn listen_for_ctrl_c(&self) {
        let interrupted = Arc::clone(&self.interrupted);
        tokio::spawn(async move {
            if tokio::signal::ctrl_c().await.is_ok() {
                interrupted.store(true, Ordering::SeqCst);
            }
        });
    }

    pub(crate) fn interrupted(&self) -> bool {
        self.interrupted.load(Ordering::SeqCst)
    }
}