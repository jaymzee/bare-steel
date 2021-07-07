use core::{
    sync::atomic::{self, AtomicBool, AtomicU64},
    pin::Pin,
    task::{Context, Poll, Waker},
    future::Future,
};
use futures_util::task::AtomicWaker;

const MAX_TIMERS: usize = 8;
const READY_DEFAULT: AtomicBool = AtomicBool::new(false);
const WAKER_DEFAULT: AtomicWaker = AtomicWaker::new();

// semaphore for tick and tock
static READY: [AtomicBool; 2] = [READY_DEFAULT; 2];
static WAKER: [AtomicWaker; MAX_TIMERS] = [WAKER_DEFAULT; MAX_TIMERS];
static TIMER: AtomicU64 = AtomicU64::new(0);

/// Called by the timer interrupt handler
///
/// Must not block or allocate.
pub(crate) fn set_timer(timer: u64) {
    TIMER.store(timer, atomic::Ordering::Relaxed);

    // tick = 1, tock = 0
    let index = (timer % 2) as usize;
    READY[index].store(true, atomic::Ordering::Relaxed);
    READY[index ^ 1].store(false, atomic::Ordering::Relaxed);

    // notify each task that is waiting for a timer tick
    for id in 0..MAX_TIMERS {
        WAKER[id].wake();
    }
}

pub enum Timer {
    Tick(usize),
    Tock(usize),
}

impl Timer {
    fn get_id(&self) -> (usize, usize) {
        match self {
            Timer::Tick(id) => (1, *id),
            Timer::Tock(id) => (0, *id),
        }
    }

    fn is_ready(&self, waker: &Waker) -> bool {
        let (t, id) = self.get_id();    // (tick/tock = 1/0, timer id)
        WAKER[id].register(waker);      // call before checking result
        READY[t].load(atomic::Ordering::Relaxed)
    }
}

impl Future for Timer {
    type Output = u64;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<u64> {
        if self.is_ready(cx.waker()) {
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
