use stm_common::vcell::{SCell, UCell, VCell};

use core::pin::Pin;
use core::task::{Context, Poll, Waker};

static PENDSV: UCell<Pendsv> = UCell::default();

static COUNT: VCell<i32> = VCell::new(0);

static CONTEXT: SCell<Context> = SCell::new(Context::from_waker(Waker::noop()));

/// Number of wake-ups per second.
pub const SECOND: u32 = 100;

#[derive_const(Default)]
struct Pendsv {
    alloc: i32,
}

struct PendSVFuture {
    wakeup_at: i32,
}

impl Future for PendSVFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>)
            -> Poll<Self::Output> {
        if COUNT.read().wrapping_sub(self.wakeup_at) >= 0 {
            Poll::Ready(())
        }
        else {
            Poll::Pending
        }
    }
}

pub fn init() {
    // We use the PENDSV exception to dispatch some work at lower priority.
    let scb = unsafe {&*cortex_m::peripheral::SCB::PTR};
    let pendsv_prio = &scb.shpr[1];
    // Cortex-M crate has two different ideas of what the SHPR is, make sure we
    // are built with the correct one.
    stm_common::link_assert!(pendsv_prio as *const _ as usize == 0xe000ed20);
    #[cfg(target_os = "none")]
    unsafe {pendsv_prio.write(crate::cpu::PRIO_APP as u32 * 65536)};
}

pub fn sleep(ticks: u32) -> impl Future {
    let pendsv = unsafe {PENDSV.as_mut()};
    pendsv.alloc = pendsv.alloc.wrapping_add_unsigned(ticks);
    PendSVFuture{wakeup_at: pendsv.alloc}
}

fn pendsv_handler() {
    let go = Pin::static_mut(unsafe {super::APP.as_mut()});
    let Poll::Pending = go.poll(unsafe {CONTEXT.as_mut()});
}

pub fn trigger() {
    let count = COUNT.read();
    let do_adc = (count & 7) == 0;
    if do_adc {
        crate::adc::power_up();
    }
    COUNT.write(count.wrapping_add(1));
    cortex_m::peripheral::SCB::set_pendsv();
    if do_adc {
        crate::adc::start();
    }
}

impl crate::cpu::Config {
    pub const fn pendsv(&mut self) -> &mut Self {
        self.vectors.pendsv = pendsv_handler;
        self
    }
}

#[test]
fn check_isr() {
    assert!(crate::cpu::VECTORS.pendsv == pendsv_handler);
}
