#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

// For link_llvm_intrinsics.
#![allow(internal_features)]
// I value sane syntax over appeasing the Rust Gods.
#![allow(unpredictable_function_pointer_comparisons)]
#![feature(derive_const)]
#![feature(const_clone, const_cmp, const_default, const_index)]
#![feature(const_trait_impl)]
// Convenience for dbgln! implementation.
#![feature(format_args_nl)]
// For frameaddress() in the crash handler.
#![feature(link_llvm_intrinsics)]

mod cpu;
mod debug;
mod leds;
mod vcell;

pub const CONFIG: cpu::Config = *cpu::Config::new(16_000_000).debug();

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
