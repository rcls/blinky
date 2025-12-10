//! LED pwm sequencing.  We use TIM3 which is a 16-bit timer with 4
//! capture/compare registers.  The LEDs are PWM'd in two banks, with independent
//! timing.  This allows two colours of LED to be driven with different duty
//! cycles.  We drive the LEDs at approx 1kHz with approx 1000 counts on the
//! duty.  (Assumes 1MHz clock).
//!
//! Double buffering is used...
//!
//! RELOAD:
//! # Turn off odd leds from the previous cycle.
//! # Turn on even leds.
//! # Trigger lower priority actions:
//!   - compute next duty cycles and update double buffered registers
//!   - trigger the per pulse state machines.
//!
//! CCR-EVEN-OFF:
//! - Turn off even leds.
//!
//! CCR-OFF-ON
//! - Turn on even leds (not that this may happen either before or after
//!   EVEN-OFF).

use stm_common::vcell::UCell;
use stm32g030::TIM3 as TIM;
use stm32g030::Interrupt::TIM3 as INTERRUPT;

use crate::leds;

/// Set LEDs via the 4-GPIO bit mask.  `on` and `off` refer to the GPIO level,
/// not the negative logic GPIO drive.  `on` takes precedence over `off`.
fn set(on: u64, off: u64) {
    let set1 = |i| {
        let on  = on  >> i * 16 & 0xffff;
        let off = off >> i * 16 & 0xffff;
        crate::leds::gpio(i).BSRR.write(
            |w| w.bits((off as u32) << 16 | on as u32));
    };
    set1(0);
    set1(1);
    set1(2);
    set1(3);
}

fn reset(off: u64) {
    let reset1 = |i| {
        let off = off >> i * 16 & 0xffff;
        crate::leds::gpio(i).BRR.write(|w| w.bits(off as u32));
    };
    reset1(0);
    reset1(1);
    reset1(2);
    reset1(3);
}

static DISPLAY: UCell<u64> = UCell::new(0);

static VBAT_OK_AVERAGE: UCell<u32> = UCell::new(0);

/// Full scale duty of even LEDs, out of 1024 PWM counts.
const EVEN_DUTY: u32 = 500;

/// Full scale duty of odd LEDs, out of 1024 PWM counts.
const ODD_DUTY: u32 = 500;

fn pwm_controller_update() {
    // We use a simple proportional controller.  We track the VBAT_OK signal
    // and use a decaying average with a time constant of ≈4s.  That then
    // becomes the PWM duty cycle.
    let gpiof = unsafe {&*stm32g030::GPIOF::PTR};
    let vbat = unsafe {VBAT_OK_AVERAGE.as_mut()};
    let ok = gpiof.IDR.read().IDR1().bit();
    *vbat = *vbat - (*vbat >> 12) + if ok {1 << 20} else {0};

    let pwm16 = *vbat >> 16;
    let tim = unsafe {&*TIM::PTR};
    tim.CCR1.write(|w| w.bits(pwm16 * EVEN_DUTY >> 16));
    tim.CCR2.write(|w| w.bits(1024 - (pwm16 * ODD_DUTY >> 16)));
}

fn isr() {
    let tim = unsafe {&*TIM::PTR};
    let sr = tim.SR.read();
    // First, update the LEDs, we want low timing jitter on this.
    let display = *DISPLAY;
    if sr.UIF().bit() {
        // Active evens asserted low, odds deasserted high.
        set(leds::LED_ODD, display);
        // FIXME - trigger the world!
    }
    if sr.CC1OF().bit() {
        // Active evens deasserted high.
        set(leds::LED_EVEN, 0);
    }
    if sr.CC2OF().bit() {
        // Active odds asserted low.
        reset(display & leds::LED_ODD);
    }
    if sr.UIF().bit() {
        pwm_controller_update();
        crate::pendsv::trigger();
    }
}

impl crate::cpu::Config {
    pub const fn pulse(&mut self) -> &mut Self {
        self.isr(INTERRUPT, isr)
    }
}
