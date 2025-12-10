use stm_common::vcell::{UCell, VCell};

use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

static PENDSV: UCell<Pendsv> = UCell::default();

static COUNT: VCell<i32> = VCell::new(0);

static VTABLE: RawWakerVTable = RawWakerVTable::new(
    |_| raw_waker(), |_| wake(), |_| wake(), |_|());

#[derive_const(Default)]
struct Pendsv {
    alloc: i32,
    waker: Option<Waker>,
}

struct PendSVFuture {
    wakeup_count: i32,
}

impl Future for PendSVFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>)
            -> Poll<Self::Output> {
        if COUNT.read().wrapping_sub(self.wakeup_count) >= 0 {
            Poll::Ready(())
        }
        else {
            unsafe {PENDSV.as_mut()}.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

pub fn sleep(ticks: u32) -> impl Future {
    let pendsv = unsafe {PENDSV.as_mut()};
    pendsv.alloc = pendsv.alloc.wrapping_add_unsigned(ticks);
    PendSVFuture{wakeup_count: pendsv.alloc}
}

fn wake() {
    if let Some(w) = unsafe {PENDSV.as_mut()}.waker.take() {
        w.wake();
    }
}

fn pendsv_handler() {
    wake();
}

pub fn trigger() {
    COUNT.write(COUNT.read().wrapping_add(1));
    cortex_m::peripheral::SCB::set_pendsv();
}

pub const fn raw_waker() -> RawWaker {
    RawWaker::new(core::ptr::null(), &VTABLE)
}

impl crate::cpu::Config {
    pub const fn pendsv(&mut self) -> &mut Self {
        self.vectors.pendsv = pendsv_handler;
        self
    }
}