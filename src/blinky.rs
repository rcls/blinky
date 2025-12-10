#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

// For link_llvm_intrinsics.
#![allow(internal_features)]
// I value sane syntax over appeasing the Rust Gods.
#![allow(unpredictable_function_pointer_comparisons)]
#![feature(derive_const)]
#![feature(const_clone, const_cmp, const_default, const_index)]
#![feature(const_slice_make_iter)]
#![feature(const_trait_impl)]
// For getting that Future into a static.
#![feature(impl_trait_in_assoc_type)]
// Convenience for dbgln! implementation.
#![feature(format_args_nl)]
// For frameaddress() in the crash handler.
#![feature(link_llvm_intrinsics)]
#![feature(never_type)]

use core::task;

use stm_common::vcell::UCell;

mod cpu;
mod chars;
mod debug;
mod leds;
mod pendsv;
mod pulse;

/// Flag for global enable/disable of debugging.
const DEBUG_ENABLE: bool = !CONFIG.no_debug;

/// Entry point used by the dbg! and dbgln! macros.
fn debug_fmt(fmt: core::fmt::Arguments) {
    if DEBUG_ENABLE {
        stm_common::debug::debug_fmt::<debug::DebugMeta>(fmt);
    }
}

const CONFIG: cpu::Config =
    *cpu::Config::new(16_000_000).debug().pendsv().pulse();

/// We really work hard to get at the type of our future...
static GO: UCell<Option<<() as SomeThing>::Thing>> = UCell::new(None);

/// Hack to extract an unnamed type.
trait SomeThing {
    type Thing;
    fn thing(&self) -> Self::Thing;
}

impl SomeThing for () {
    type Thing = impl Future<Output = !>;
    fn thing(&self) -> Self::Thing {dummy()}
}

async fn dummy() -> ! {
    loop {
        pendsv::sleep(1000).await;
    }
}

pub fn main() -> ! {
    let rcc  = unsafe {&*stm32g030::RCC::ptr()};
    rcc.IOPENR.write(|w| w.bits(0x0f));

    cpu::init();

    debug::init();

    for i in 0 .. 4 {
        let gpio = leds::gpio(i as u8);
        let bits = leds::PORT_BITS[i];
        let bit2 = leds::PORT_BIT2[i];
        gpio.BSRR.write(|w| w.bits(bits));
        gpio.OTYPER.write(|w| w.bits(bits));
        gpio.MODER.modify(|r, w| w.bits(r.bits() & !(bit2 * 2) | bit2));
    }

    let waker = unsafe {task::Waker::from_raw(pendsv::raw_waker())};
    let mut context = task::Context::from_waker(&waker);

    let go = unsafe{GO.as_mut()}.insert(().thing());
    let core::task::Poll::Pending
        = core::pin::Pin::static_mut(go).poll(&mut context);

    loop {
        for led in leds::LEDS {
            let gpio = leds::gpio(led);
            let bit = leds::bit(led);
            gpio.BRR.write(|w| w.bits(bit));
            for _ in 0 .. CONFIG.clk / 36 {
                cpu::nothing();
            }
            gpio.BSRR.write(|w| w.bits(bit));
        }
    }
}
