use crate::println;
use conquer_once::spin::OnceCell;
use core::{
    sync::atomic::{AtomicU64, Ordering},
    pin::Pin,
    task::{Context, Poll},
};
use futures_util::{
    stream::{Stream},
    task::AtomicWaker,
};

static TIMER: OnceCell<AtomicU64> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

/// Called by the timer interrupt handler
///
/// Must not block or allocate.
pub(crate) fn set_timer(time: u64) {
    if let Ok(timer) = TIMER.try_get() {
        timer.store(time, Ordering::Relaxed);
        WAKER.wake();
    } else {
        println!("WARNING: timer uninitialized");
    }
}

pub struct TimerStream {
    _private: (),
}

impl TimerStream {
    pub fn new() -> Self {
        TIMER.try_init_once(|| AtomicU64::new(0))
            .expect("TimerStream::new should only be called once");
        Self { _private: () }
    }
}

impl Stream for TimerStream {
    type Item = u64;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u64>> {
        let timer = TIMER
            .try_get()
            .expect("timer not initialized");

        WAKER.register(&cx.waker());
        let timer_value = timer.load(Ordering::Relaxed);
        WAKER.take();
        Poll::Ready(Some(timer_value))
    }
}
