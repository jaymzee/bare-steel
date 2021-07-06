use crate::println;
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::{
    stream::{Stream, StreamExt},
    task::AtomicWaker,
};

static TIMER_QUEUE: OnceCell<ArrayQueue<u64>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

/// Called by the timer interrupt handler
///
/// Must not block or allocate.
pub(crate) fn set_timer(time: u64) {
    if let Ok(queue) = TIMER_QUEUE.try_get() {
        if let Err(_) = queue.push(time) {
            println!("WARNING: timer queue full; dropping timer value");
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING: timer queue uninitialized");
    }
}

pub struct TimerStream {
    _private: (),
}

impl TimerStream {
    pub fn new() -> Self {
        TIMER_QUEUE.try_init_once(|| ArrayQueue::new(100))
            .expect("TimerStream::new should only be called once");
        Self { _private: () }
    }
}

impl Stream for TimerStream {
    type Item = u64;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u64>> {
        let queue = TIMER_QUEUE
            .try_get()
            .expect("timer queue not initialized");

        // fast path
        if let Ok(time) = queue.pop() {
            return Poll::Ready(Some(time));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Ok(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}

pub async fn display_timer() {
    use crate::vga::{display, Color, ScreenAttribute};

    let cyan = ScreenAttribute::new(Color::LightCyan, Color::Black);
    let mut stream = TimerStream::new();
    loop {
        let s = match stream.next().await {
            Some(timer) => format!("{:>4}", timer),
            _ => format!("error")
        };
        display(&s, (1, 3), cyan);
    }
}
