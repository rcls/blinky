//! LED pwm sequencing.  We use TIM3 which is a 16-bit timer with 4
//! capture/compare registers.  The LEDs are PWM'd in two banks, with independent
//! timing.  This allows two colours of LED to be driven with different duty
//! cycles.  We drive the LEDs at approx 1kHz with approx 1000 counts on the
//! duty.  (Assumes 1MHz clock).
//!
//! Double buffering is used...

use stm_common::vcell::UCell;
use stm32g030::TIM3 as TIM;
use stm32g030::Interrupt::TIM3 as INTERRUPT;

use crate::leds;

mod text;

// Number of PWM pulses per second.
pub const RATE: u32 = 40;

const PWM_PRESCALE: u32 = (crate::CONFIG.clk / 50_000).max(1);
const PWM_DIV: u32 = crate::CONFIG.clk / PWM_PRESCALE / RATE;
const _: () = assert!(PWM_PRESCALE <= 65536);
const _: () = assert!(PWM_DIV <= 65536);
const _: () = assert!(RATE * PWM_DIV * PWM_PRESCALE == crate::CONFIG.clk);
const _: () = assert!(PWM_DIV >= 1000);

/// Currently displaying LEDs.
static LEDS: UCell<u64> = UCell::new(0);

/// LEDs to display starting at next PWM cycle.
static NEXT_LEDS: UCell<u64> = UCell::new(0);

macro_rules! dbgln {($($tt: tt)*) => {if false {stm_common::dbgln!($($tt)*)}}}

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

pub fn init() {
    let rcc = unsafe {&*stm32g030::RCC::PTR};
    let tim = unsafe {&*TIM::PTR};

    rcc.APBENR1.modify(|_, w| w.TIM3EN().set_bit());

    tim.PSC.write(|w| w.bits(PWM_PRESCALE - 1));
    tim.ARR.write(|w| w.bits(PWM_DIV - 1));
    tim.DIER.write(|w| w.UIE().set_bit().CC1IE().set_bit().CC2IE().set_bit());
    tim.CCMR1_Output().write(
        |w|w.OC1CE().set_bit().OC1M().bits(1).OC1PE().set_bit()
            .OC2CE().set_bit().OC2M().bits(1).OC2PE().set_bit());
    tim.CCER.write(|w| w.CC1E().set_bit().CC2E().set_bit());
    tim.CR1.write(|w| w.CEN().set_bit());

    stm_common::interrupt::enable_priority(INTERRUPT, crate::cpu::PRIO_PULSE);
}

pub fn set_leds(leds: u64) {
    stm_common::interrupt::disable_all();
    *unsafe {NEXT_LEDS.as_mut()} = leds;
    stm_common::interrupt::enable_all();
}

pub fn apply_leds() {
    stm_common::interrupt::disable_all();
    *unsafe {LEDS.as_mut()} = *NEXT_LEDS;
    stm_common::interrupt::enable_all();
}

#[inline]
pub const fn shr8(val: u64) -> u64 {
    let (lo, hi) = (val as u32, (val >> 32) as u32);
    let (lo, hi) = (lo >> 8 | hi << 24, hi >> 8);
    lo as u64 | (hi as u64) << 32
}

pub fn set_display(display: u64) {
    let mut leds = 0;
    let mut d = display;
    for i in 0 .. 6 {
        leds |= crate::leds::COLUMNS[i][d as usize & 0xff];
        d = shr8(d);
    }
    set_leds(leds);
    if crate::DEBUG_ENABLE {
        let strings = text::blocks(display);
        dbgln!("{}{}{}\n{}{}{}",
               strings[0], strings[1], strings[2],
               strings[3], strings[4], strings[5]);
    }
}

/// Go from 50% duty at delta==0 to 2.5% duty at delta ≈ OVER3 + UNDER3.
fn calc_duty(delta: u32) -> u32 {
    const MAX: u32 = crate::adc::OVER3 + crate::adc::UNDER3;
    const RANGE: f64 = PWM_DIV as f64 * (0.5 - 0.025);
    const SCALE_F: f64 = RANGE * 65536.0 / MAX as f64;
    const SCALE: u32 = (SCALE_F + 0.5) as u32;
    (SCALE * delta >> 16) + PWM_DIV / 40
}

pub fn update_duty(delta: u32) {
    let tim = unsafe {&*TIM::PTR};
    let pwm16 = calc_duty(delta);
    tim.CCR1.write(|w| w.bits(pwm16));
    tim.CCR2.write(|w| w.bits(2 * pwm16));
}

fn isr() {
    let tim = unsafe {&*TIM::PTR};
    let sr = tim.SR.read();
    tim.SR.write(|w| w.bits(!sr.bits()));
    // First, update the LEDs, we want low timing jitter on this.
    let leds = *LEDS;
    match sr.bits() & 7 {
        1 | 5 => // UIF with or without CC2.
            // Active evens asserted low, all others deasserted high.
            set(leds::LED_EVEN & !leds | leds::LED_ODD, leds::LED_EVEN),
        2 | 3 | 7 => // CC1 with or without UIF.
            // Also includes UIF+CC1+CC2.  This probably means that the previous
            // PWM cycle was near-full-duty (CC2 at end) and this PWM cycle is
            // low (CC1 at start).
            // Active odds asserted low.
            set(leds::LED_ODD & !leds | leds::LED_EVEN, leds::LED_ODD),
        4 | 6 => // CC2 with or without CC1.
            // Everything deasserted high.
            set(leds::LED_ODD | leds::LED_EVEN, 0),
        0 => (), // Unexpected wake-up!
        _ => stm_common::utils::unreachable(),

    }
    if sr.UIF().bit() {
        crate::pendsv::trigger();
    }
}

impl crate::cpu::Config {
    pub const fn pulse(&mut self) -> &mut Self {
        self.isr(INTERRUPT, isr)
    }
}

#[test]
fn test_shr8() {
    for i in 0 .. 64 {
        for j in 0 .. 64 {
            let x = 1 << i | 1 << j;
            assert_eq!(shr8(x), x >> 8);
        }
    }
}

#[test]
fn test_pwm_duty() {
    let max = crate::adc::OVER3 + crate::adc::UNDER3;
    assert!(calc_duty(max) <= PWM_DIV / 2);
    assert!(calc_duty(max) >= PWM_DIV / 2 - 1);
    assert!(calc_duty(0) >= PWM_DIV / 40);
    assert!(calc_duty(0) <= PWM_DIV / 40 + 1);
    assert!(calc_duty(0) > 10);
}
