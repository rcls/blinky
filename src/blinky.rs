#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

// For link_llvm_intrinsics.
#![allow(internal_features)]
// I value sane syntax over appeasing the Rust Gods.
#![allow(unpredictable_function_pointer_comparisons)]
// We do lots of const.
#![feature(derive_const)]
#![feature(const_clone, const_cmp, const_default, const_index)]
#![feature(const_slice_make_iter, const_trait_impl)]
// For getting that Future into a static.
#![feature(impl_trait_in_assoc_type)]
// Convenience for dbgln! implementation.
#![feature(format_args_nl)]
// For frameaddress() in the crash handler.
#![feature(link_llvm_intrinsics)]
#![feature(never_type)]

#![feature(const_async_blocks)]

use stm_common::vcell::UCell;

use pulse::SECOND;

mod adc;
mod chars;
mod cpu;
mod debug;
mod leds;
mod marque;
mod pendsv;
mod pulse;

/// Flag for global enable/disable of debugging.
const DEBUG_ENABLE: bool = !CONFIG.no_debug;

const CONFIG: cpu::Config =
    *cpu::Config::new(250_000).adc().no_debug().pendsv().pulse();

/// Entry point used by the dbg! and dbgln! macros.
fn debug_fmt(fmt: core::fmt::Arguments) {
    if DEBUG_ENABLE {
        stm_common::debug::debug_fmt::<debug::DebugMeta>(fmt);
    }
}

/// We really work hard to get at the type of our future...
static APP: UCell<<Start as Thing>::Thing> = UCell::new(Start.thing());

struct Start;

/// Hack to extract an unnamed type.
const trait Thing {
    type Thing;
    fn thing(&self) -> Self::Thing;
}

const impl Thing for Start {
    type Thing = impl Future<Output = !>;
    fn thing(&self) -> Self::Thing {start()}
}

/// Returns the future for running the asynchronous application code.
const fn start() -> impl Future<Output = !> {
    async {
        //pendsv::sleep(500).await;
        let mut display = marque::Display::default();
        loop {
            pendsv::sleep(SECOND).await;

            const STR: &[u8] = &chars::map_str(b"MERRY CHRISTMAS ");
            display.marque_string(STR, SECOND / 5).await;
        }
    }
}

fn main() -> ! {
    let rcc  = unsafe {&*stm32g030::RCC::PTR};

    cpu::init();

    rcc.IOPENR.write(
        |w|w.GPIOAEN().set_bit().GPIOBEN().set_bit().GPIOCEN().set_bit()
            .GPIODEN().set_bit());

    if CONFIG.is_lazy_debug() || CONFIG.no_debug {
        let gpioa = unsafe {&*stm32g030::GPIOA::PTR};
        // gpioa.OTYPER.modify(|_, w| w.OT9().set_bit());
        gpioa.BSRR.write(|w| w.BS9().set_bit());
        gpioa.MODER.write(|w| w.MODER9().bits(1));
    }
    else {
        debug::init();
    }

    for i in 0 .. 4 {
        let gpio = leds::gpio(i as u8);
        let bits = leds::PORT_BITS[i];
        let bit2 = leds::PORT_BIT2[i];
        gpio.BSRR.write(|w| w.bits(bits));
        gpio.MODER.modify(|r, w| w.bits(r.bits() & !(bit2 * 2) | bit2));
    }

    pendsv::init();
    pulse::init();

    loop {
        stm_common::utils::WFE();
    }
}
