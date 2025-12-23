#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

// For link_llvm_intrinsics.
#![allow(internal_features)]
// I value sane syntax over appeasing the Rust Gods.
#![allow(unpredictable_function_pointer_comparisons)]
// We do lots of const.
#![feature(derive_const)]
#![feature(const_clone, const_cmp, const_convert, const_default, const_index)]
#![feature(const_slice_make_iter, const_trait_impl)]
// For getting that Future into a static.
#![feature(impl_trait_in_assoc_type)]
// Convenience for dbgln! implementation.
#![feature(format_args_nl)]
// For frameaddress() in the crash handler.
#![feature(link_llvm_intrinsics)]
#![feature(never_type)]

#![feature(const_async_blocks)]

use crate::pendsv::{SECOND, animate, hold_display};

mod adc;
mod chars;
mod cpu;
mod config;
mod debug;
mod leds;
mod marque;
mod pendsv;
mod pulse;
mod random;

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

/// Sequence LEDs on demo board....
fn demo() {
    let mut d = 14 << 40;
    for _ in 0 .. 6 {
        pendsv::sleep(SECOND);
        pendsv::set_display(d);
        d = marque::shr8(d);
    }
    pendsv::sleep(SECOND);
    pendsv::set_display(0x10 << 16);
    pendsv::sleep(SECOND);
    pendsv::set_display(0x10 << 24);
}

fn blink_in() {
    for _ in 0 .. 5 {
        hold_display(0, 1);
        hold_display(chars::CDOT, 1);
    }
}

fn cycles() {
    for _ in 0 .. 3 {
        animate(&chars::IDOTS, 1);
    }
    hold_display(chars::MDOTS, 1);
    for _ in 0 .. 2 {
        animate(&chars::ODOTPS, 1);
    }
    animate(&chars::ODOTPS[0 .. 4], 1);
    hold_display(chars::CORNERS, 1);
}

fn common_display() {
     const STR: &[u8] = &chars::map_str(b"MERRY CHRISTMAS ");
     marque::Display::default().marque_string(STR, SECOND / 5);
}

fn nice1() {
    const I: u64 = chars::picture('I');
    const U: u64 = chars::picture('U');
    hold_display(I, 2 * SECOND);
    for _ in 0 .. 5 {
        hold_display(0, 1);
        hold_display(chars::LOVE, 1);
    }
    hold_display(0, 1);
    hold_display(U, 2 * SECOND);
}

fn nice2() {
     const STR: &[u8] = &chars::map_str(b"HEY GOOD LOOKING ");
     marque::Display::default().marque_string(STR, SECOND / 5);
}

fn nice3() {
     const STR: &[u8] = &chars::map_str(b"NICE HAIR ");
     marque::Display::default().marque_string(STR, SECOND / 5);
}

fn naughty1() {
     const STR: &[u8] = &chars::map_str(b"WHO FARTED? ");
     marque::Display::default().marque_string(STR, SECOND / 5);
}

fn naughty2() {
     const STR: &[u8] = &chars::map_str(b"LICK ME ");
     marque::Display::default().marque_string(STR, SECOND / 5);
}

/// Returns the future for running the asynchronous application code.
fn run() -> ! {
    if false {
        loop {
            demo();
        }
    }
    if false {
        loop { // Pattern to test LEDs.
            let mut d = 0x3f;
            while d != 0 {
                hold_display(d, 2);
                d <<= 8;
            }
        }
    }

    blink_in();
    hold_display(0, 1);

    let mut count = 0u32;
    loop {
        count = count.saturating_add(1);
        // Probability of exception ramps from 0 at <5 to ⅔ at 25.
        let normal = count <= 5
            || random::RANDOM.random_n(30) >= count.min(25) - 5;

        if normal {
            common_display();
            hold_display(0, 2);
        }
        else {
            blink_in();
            cycles();
            hold_display(0, 1);
            match random::RANDOM.random_n(5) {
                0 => nice1(),
                1 => nice2(),
                2 => nice3(),
                3 => naughty1(),
                4|_ => naughty2(),
            }
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
        gpioa.BSRR.write(|w| w.BS9().set_bit());
        gpioa.MODER.write(|w| w.MODER9().bits(1));
    }
    else {
        debug::init();
    }

    for i in 0 .. 4 {
        let gpio = leds::gpio(i);
        let bits = leds::PORT_BITS[i];
        let bit2 = leds::PORT_BIT2[i];
        gpio.BSRR.write(|w| w.bits(bits));
        gpio.MODER.modify(|r, w| w.bits(r.bits() & !(bit2 * 2) | bit2));
    }

    pendsv::init();
    pulse::init();

    run();
}
