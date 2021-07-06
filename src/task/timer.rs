use core::{
    sync::atomic::{AtomicBool, Ordering},
    pin::Pin,
    task::{Context, Poll},
    future::Future,
};
use futures_util::task::AtomicWaker;

static NOTIFY: AtomicBool = AtomicBool::new(false);
static WAKER: AtomicWaker = AtomicWaker::new();

/// Called by the timer interrupt handler
///
/// Must not block or allocate.
pub(crate) fn notify_timer_tick() {
    NOTIFY.swap(true, Ordering::Release);
    WAKER.wake();
}

pub struct TickFuture;

impl TickFuture {
    pub fn new() -> Self {
        Self { }
    }
}

impl Future for TickFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        WAKER.register(&cx.waker());
        if NOTIFY.swap(false, Ordering::Release) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

pub async fn system_timer() {
    use crate::vga::{self, Color, ScreenAttribute};
    let cyan = ScreenAttribute::new(Color::LightCyan, Color::Black);
    let mut timer: u64 = 0;

    loop {
        TickFuture::new().await;
        timer += 1;
        let time_str = format!("{:>4}", timer);
        vga::display(&time_str, (1, 3), cyan);
    }
}
