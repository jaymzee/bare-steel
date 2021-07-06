use core::{
    sync::atomic::{self, AtomicBool, AtomicU64},
    pin::Pin,
    task::{Context, Poll, Waker},
    future::Future,
};
use futures_util::task::AtomicWaker;

const TIMERS: usize = 3;
const FLAG_INIT: AtomicBool = AtomicBool::new(false);
const WAKER_INIT: AtomicWaker = AtomicWaker::new();
const WAKERS_INIT: [AtomicWaker; TIMERS] = [WAKER_INIT; TIMERS];

// semaphore for tick and tock
static FLAG: [AtomicBool; 2] = [FLAG_INIT; 2];
static WAKER: [[AtomicWaker; TIMERS]; 2] = [WAKERS_INIT; 2];
static TIMER: AtomicU64 = AtomicU64::new(0);

/// Called by the timer interrupt handler
///
/// Must not block or allocate.
pub(crate) fn set_timer(timer: u64) {
    let tick_tock = (timer % 2) as usize;

    TIMER.store(timer, atomic::Ordering::Relaxed);
    FLAG[tick_tock ^ 1].store(false, atomic::Ordering::Relaxed);
    FLAG[tick_tock].store(true, atomic::Ordering::Relaxed);

    for id in 0..TIMERS {
        WAKER[tick_tock][id].wake();
    }
}

pub enum Timer {
    Tick(usize),
    Tock(usize),
}

impl Timer {
    fn ids(&self) -> (usize, usize) {
        match self {
            Timer::Tick(id) => (1, *id),
            Timer::Tock(id) => (0, *id),
        }
    }

    fn ready(&self, waker: &Waker) -> bool {
        let (tick_tock, id) = self.ids();
        WAKER[tick_tock][id].register(waker);
        FLAG[tick_tock].load(atomic::Ordering::Relaxed)
    }
}

impl Future for Timer {
    type Output = u64;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<u64> {
        if self.ready(cx.waker()) {
            Poll::Ready(TIMER.load(atomic::Ordering::Relaxed))
        } else {
            Poll::Pending
        }
    }
}

pub async fn system_timer(id: usize) {
    use crate::vga::{self, Color, ScreenAttribute};
    let cyan = ScreenAttribute::new(Color::LightCyan, Color::Black);
    let pos = (1, 3 + 10 * id as u8);
    loop {
        let timer = Timer::Tick(id).await;
        vga::display(&format!("{:>4}", timer), pos, cyan);
        let timer = Timer::Tock(id).await;
        vga::display(&format!("{:>4}", timer), pos, cyan);
    }
}
