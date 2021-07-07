use core::{
    sync::atomic::{self, AtomicU64},
    pin::Pin,
    task::{Context, Poll},
    future::Future,
};
use futures_util::task::AtomicWaker;

const MAX_TIMERS: usize = 8;
const WAKER_DEFAULT: AtomicWaker = AtomicWaker::new();

/// timer value
static TIMER: AtomicU64 = AtomicU64::new(0);
/// synchronized task wakeup for each timer
static WAKER: [AtomicWaker; MAX_TIMERS] = [WAKER_DEFAULT; MAX_TIMERS];

/// Called by the timer interrupt handler
///
/// Must not block or allocate.
pub(crate) fn set_timer(timer: u64) {
    TIMER.store(timer, atomic::Ordering::Relaxed);

    // notify each task that is waiting for a timer tick
    for waker in WAKER.iter() {
        waker.wake();
    }
}

pub enum Timer {
    Tick(usize),
    Tock(usize),
}

impl Timer {
    fn get_id(&self) -> (usize, u8) {
        match self {
            Timer::Tick(id) => (*id, 1),
            Timer::Tock(id) => (*id, 0),
        }
    }
}

impl Future for Timer {
    type Output = u64;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<u64> {
        let (id, tick) = self.get_id(); // (timer id, tick/tock = 1/0)

        // clock is the lsb of TIMER
        WAKER[id].register(cx.waker()); // call before checking result
        let clock = TIMER.load(atomic::Ordering::Relaxed) as u8 & 1;

        if tick == clock {
            Poll::Ready(TIMER.load(atomic::Ordering::Relaxed))
        } else {
            Poll::Pending
        }
    }
}
