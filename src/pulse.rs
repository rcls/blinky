//! LED pwm sequencing.  We use TIM3 which is a 16-bit timer with 4
//! capture/compare registers.  The LEDs are PWM'd in two banks.
//!  We drive the LEDs at approx 80Hz with 3125 PWM clocks per PWM cycle.
//!
//! Double buffering is used.

use stm_common::vcell::UCell;
use stm32g030::TIM3 as TIM;
use stm32g030::Interrupt::TIM3 as INTERRUPT;

use crate::leds::LED_ALL;

// Number of PWM pulses per second.
pub const RATE: u32 = 80;

/// Prescaler for the PWM timer.
const PWM_PRESCALE: u32 = (crate::CONFIG.clk / 250_000).max(1);
/// Number of PWM clocks per pwm cycle.
pub const PWM_DIV: u32 = crate::CONFIG.clk / PWM_PRESCALE / RATE;

const _: () = assert!(PWM_PRESCALE <= 65536);
const _: () = assert!(PWM_DIV <= 65536);
const _: () = assert!(RATE * PWM_DIV * PWM_PRESCALE == crate::CONFIG.clk);
const _: () = assert!(PWM_DIV >= 500);

// #[derive_const(Default)]
#[derive(Copy, Clone)]
struct Leds {
    leds: [u16; 4],
}

/// Currently displaying LEDs.
static LEDS: UCell<Leds> = UCell::new(Leds{leds: [0; _]});

pub fn init() {
    let rcc = unsafe {&*stm32g030::RCC::PTR};
    let tim = unsafe {&*TIM::PTR};

    rcc.APBENR1.modify(|_, w| w.TIM3EN().set_bit());

    tim.PSC.write(|w| w.bits(PWM_PRESCALE - 1));
    tim.ARR.write(|w| w.bits(PWM_DIV - 1));
    tim.DIER.write(|w| w.UIE().set_bit().CC1IE().set_bit());
    tim.CCMR1_Output().write(
        |w| w.OC1PE().set_bit().OC1M().bits(1).OC1PE().set_bit());
    tim.CR1.write(|w| w.CEN().set_bit());

    stm_common::interrupt::enable_priority(INTERRUPT, crate::cpu::PRIO_PULSE);
}

/// Set the LED pattern for future PWM cycles.
pub fn apply_leds(pos: u64) {
    let leds = unsafe {LEDS.as_mut()};
    stm_common::interrupt::disable_all();
    leds.leds[0] = pos as u16;
    leds.leds[1] = (pos >> 16) as u16;
    leds.leds[2] = (pos >> 32) as u16;
    leds.leds[3] = (pos >> 48) as u16;
    stm_common::interrupt::enable_all();
}

pub fn set_duty(duty: u32) {
    let tim = unsafe {&*TIM::PTR};
    tim.CCR1.write(|w| w.bits(duty));
}

fn isr() {
    let tim = unsafe {&*TIM::PTR};
    let sr = tim.SR.read();
    tim.SR.write(|w| w.bits(!sr.bits()));
    // First, update the LEDs, we want low timing jitter on this.
    if sr.UIF().bit() {
        reset(&LEDS.as_ref().leds);
    }
    if sr.CC1IF().bit() {
        set(LED_ALL, 0);
        crate::pendsv::trigger();
    }
}

/// Set LEDs via the 4-GPIO bit mask.  `on` and `off` refer to the GPIO level,
/// not the negative logic GPIO drive.  `on` takes precedence over `off`.
fn set(on: u64, off: u64) {
    fn set1(on: u64, off: u64, i: usize) {
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

/// Reset LED GPIOs via the 4-GPIO bit mask.  This is negative logic so it turns
/// LEDs on.
fn reset(bits: &[u16; 4]) {
    fn reset1(bits: &[u16; 4], i: usize) {
        crate::leds::gpio(i).BRR.write(|w| w.bits(bits[i] as u32));
    }
    reset1(bits, 0);
    reset1(bits, 1);
    reset1(bits, 2);
    reset1(bits, 3);
}

impl crate::cpu::Config {
    pub const fn pulse(&mut self) -> &mut Self {
        self.isr(INTERRUPT, isr)
    }
}
