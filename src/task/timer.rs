use core::{
    sync::atomic::{self, AtomicBool, AtomicU64},
    pin::Pin,
    task::{Context, Poll},
    future::Future,
};
use futures_util::task::AtomicWaker;

const TASKS: usize = 5;
const FLAG_INIT: AtomicBool = AtomicBool::new(false);
const WAKER_INIT: AtomicWaker = AtomicWaker::new();
const WAKER_TASKS_INIT: [AtomicWaker; TASKS] = [WAKER_INIT; TASKS];

static FLAG: [AtomicBool; 2] = [FLAG_INIT; 2];
static WAKER: [[AtomicWaker; TASKS]; 2] = [WAKER_TASKS_INIT; 2];
static TIMER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Copy, Clone)]
pub enum TickTock {
    Tick = 1,
    Tock = 0
}

/// Called by the timer interrupt handler
///
/// Must not block or allocate.
pub(crate) fn set_timer(timer: u64) {
    let tick_tock = (timer % 2) as usize;

    TIMER.store(timer, atomic::Ordering::Relaxed);
    FLAG[tick_tock ^ 1].store(false, atomic::Ordering::Relaxed);
    FLAG[tick_tock].store(true, atomic::Ordering::Relaxed);

    for task in 0..TASKS {
        WAKER[tick_tock][task].wake();
    }
}

pub struct Timer {
    tick_tock: TickTock,
    task_id: usize
}

impl Timer {
    pub const fn new(tick_tock: TickTock, task_id: usize) -> Self {
        Self { tick_tock, task_id }
    }
}

impl Future for Timer {
    type Output = u64;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<u64> {
        let id = self.tick_tock as usize;
        let task = self.task_id;
        WAKER[id][task].register(&cx.waker());
        if FLAG[id].load(atomic::Ordering::Relaxed) {
            Poll::Ready(TIMER.load(atomic::Ordering::Relaxed))
        } else {
            Poll::Pending
        }
    }
}

pub async fn system_timer(id: usize) {
    use TickTock::{Tick, Tock};
    use crate::vga::{self, Color, ScreenAttribute};
    let cyan = ScreenAttribute::new(Color::LightCyan, Color::Black);
    let pos = (1, 3 + 10 * id as u8);
    loop {
        let timer = Timer::new(Tick, id).await;
        vga::display(&format!("{:>4}  ", timer), pos, cyan);
        let timer = Timer::new(Tock, id).await;
        vga::display(&format!("{:>4} .", timer), pos, cyan);
    }
}
