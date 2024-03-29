pub mod pit;

use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{self, AtomicU64},
    task::{Context, Poll},
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
    fn id(&self) -> (usize, u8) {
        match *self {
            Timer::Tick(id) => (id, 1),
            Timer::Tock(id) => (id, 0),
        }
    }
}

impl Future for Timer {
    type Output = u64;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<u64> {
        let (id, tick) = self.id(); // (timer id, tick/tock = 1/0)

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

pub async fn sleep(id: usize, ticks: u32) -> u64 {
    assert!(ticks >= 2);
    let hticks = ticks / 2;
    let mut timer: u64 = 0;
    for _ in 0..hticks {
        Timer::Tick(id).await;
        timer = Timer::Tock(id).await;
    }
    timer
}
