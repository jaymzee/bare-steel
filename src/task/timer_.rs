use core::{
    sync::atomic::{AtomicU64, Ordering},
    pin::Pin,
    task::{Context, Poll},
};
use futures_util::{
    stream::{Stream},
    task::AtomicWaker,
};

static TIMER: AtomicU64 = AtomicU64::new(0);
static WAKER: AtomicWaker = AtomicWaker::new();

/// Called by the timer interrupt handler
///
/// Must not block or allocate.
pub(crate) fn set_timer(time: u64) {
    TIMER.store(time, Ordering::Relaxed);
    WAKER.wake();
}

pub struct TimerStream {
    _private: (),
}

impl TimerStream {
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Stream for TimerStream {
    type Item = u64;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u64>> {
        WAKER.register(&cx.waker());
        let timer_value = TIMER.load(Ordering::Relaxed);
        WAKER.take();
        Poll::Ready(Some(timer_value))
    }
}
