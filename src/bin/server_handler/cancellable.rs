use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::spawn;

pub struct Cancellable {
    is_cancelled: Arc<AtomicBool>,
}

impl Cancellable {
    pub fn new<F>(f: F) -> Cancellable
    where
        F: Send + 'static + FnOnce(Arc<AtomicBool>) -> (),
    {
        let is_cancelled = Arc::new(AtomicBool::new(false));
        let is_cancelled_copy = is_cancelled.clone();

        spawn(move || f(is_cancelled_copy));

        Cancellable { is_cancelled }
    }

    pub fn cancel(&self) {
        self.is_cancelled.store(true, Ordering::SeqCst);
    }
}
