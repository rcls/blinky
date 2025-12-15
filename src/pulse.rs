//! LED pwm sequencing.  We use TIM3 which is a 16-bit timer with 4
//! capture/compare registers.  The LEDs are PWM'd in two banks, with independent
//! timing.  This allows two colours of LED to be driven with different duty
//! cycles.  We drive the LEDs at approx 1kHz with approx 1000 counts on the
//! duty.  (Assumes 1MHz clock).
//!
//! Double buffering is used...

use stm_common::vcell::{UCell, VCell};
use stm32g030::TIM3 as TIM;
use stm32g030::Interrupt::TIM3 as INTERRUPT;

use crate::leds::{LED_EVEN, LED_ODD};

// Number of PWM pulses per second.
pub const RATE: u32 = 80;

const PWM_PRESCALE: u32 = (crate::CONFIG.clk / 250_000).max(1);
pub const PWM_DIV: u32 = crate::CONFIG.clk / PWM_PRESCALE / RATE;
pub const CLOCKS_PER_TICK: u32 = PWM_DIV * crate::pendsv::PWM_PER_TICK;

const _: () = assert!(PWM_PRESCALE <= 65536);
const _: () = assert!(PWM_DIV <= 65536);
const _: () = assert!(RATE * PWM_DIV * PWM_PRESCALE == crate::CONFIG.clk);
const _: () = assert!(PWM_DIV >= 500);

#[derive_const(Default)]
#[derive(Copy, Clone)]
struct Leds {
    even: u64,
    odd: u64,
}

/// Currently displaying LEDs.
static LEDS: UCell<Leds> = UCell::default();
/// Which interrupt bit to use for application ticks.
static APP_MASK: VCell<u32> = VCell::new(0);

pub fn init() {
    let rcc = unsafe {&*stm32g030::RCC::PTR};
    let tim = unsafe {&*TIM::PTR};

    APP_MASK.write(1);
    *unsafe {LEDS.as_mut()} = Leds{
        even: LED_EVEN|LED_ODD, odd: LED_EVEN|LED_ODD};

    rcc.APBENR1.modify(|_, w| w.TIM3EN().set_bit());

    tim.PSC.write(|w| w.bits(PWM_PRESCALE - 1));
    tim.ARR.write(|w| w.bits(PWM_DIV - 1));
    tim.DIER.write(|w| w.UIE().set_bit().CC1IE().set_bit().CC2IE().set_bit());
    tim.CCMR1_Output().write(
        |w|w.OC1PE().set_bit().OC1M().bits(1).OC1PE().set_bit()
            .OC1PE().set_bit().OC2M().bits(1).OC2PE().set_bit());
    tim.CR1.write(|w| w.CEN().set_bit());

    stm_common::interrupt::enable_priority(INTERRUPT, crate::cpu::PRIO_PULSE);
}

/// LEDs should have been converted to negative logic by the caller.
pub fn apply_leds(even: u64, odd: u64) {
    stm_common::interrupt::disable_all();
    *unsafe {LEDS.as_mut()} = Leds{even, odd};
    stm_common::interrupt::enable_all();
}

pub fn set_duty(duty1: u32, duty2: u32) {
    let tim = unsafe {&*TIM::PTR};
    tim.CCR1.write(|w| w.bits(duty1));
    tim.CCR2.write(|w| w.bits(duty2));
    if duty2 <= PWM_DIV / 2 {
        // Trigger application update on CC2.
        APP_MASK.write(4);
    }
    else {
        APP_MASK.write(2); // CC1.
    }
}

fn isr() {
    let tim = unsafe {&*TIM::PTR};
    let sr = tim.SR.read();
    tim.SR.write(|w| w.bits(!sr.bits()));
    // First, update the LEDs, we want low timing jitter on this.
    let leds = LEDS.as_ref();
    match sr.bits() & 7 {
        1 | 5 => // UIF with or without CC2.
            // Active evens asserted low, all others deasserted high.
            set(leds.even, LED_EVEN | LED_ODD),
        2 | 3 => // CC1 with or without UIF.
            // Active odds asserted low.
            set(leds.odd, LED_EVEN | LED_ODD),
        4 | 6 | 7 => // CC2 with or without CC1.
            // Everything deasserted high.
            set(LED_ODD | LED_EVEN, 0),
        0 => (), // Unexpected wake-up!
        _ => stm_common::utils::unreachable(),
    }
    if sr.bits() & APP_MASK.read() != 0 {
        crate::pendsv::trigger();
    }
}

/// Set LEDs via the 4-GPIO bit mask.  `on` and `off` refer to the GPIO level,
/// not the negative logic GPIO drive.  `on` takes precedence over `off`.
fn set(on: u64, off: u64) {
    fn set1(on: u64, off: u64, i: u8) {
        let on  = on  >> i * 16 & 0xffff;
        let off = off >> i * 16 & 0xffff;
        crate::leds::gpio(i).BSRR.write(
            |w| w.bits((off as u32) << 16 | on as u32));
    }
    set1(on, off, 0);
    set1(on, off, 1);
    set1(on, off, 2);
    set1(on, off, 3);
}

impl crate::cpu::Config {
    pub const fn pulse(&mut self) -> &mut Self {
        self.isr(INTERRUPT, isr)
    }
}
