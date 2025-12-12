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

mod text;

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

static LEDS: UCell<u64> = UCell::new(0);

static VBAT_OK_AVERAGE: UCell<u32> = UCell::new(0);

const PWM_PRESCALE: u32 = (crate::CONFIG.clk / 1000_000).max(1);
const PWM_DIV: u32 = crate::CONFIG.clk / PWM_PRESCALE / 100;
const _: () = assert!(PWM_DIV <= 65536);

/// Full scale duty of even LEDs, out of 1024 PWM counts.
const EVEN_DUTY: u32 = PWM_DIV / 2;

/// Full scale duty of odd LEDs, out of 1024 PWM counts.
const ODD_DUTY: u32 = PWM_DIV / 2;

/// Lower bound (out of u32::MAX) on the VBAT_OK filter.
const VBAT_LOW: u32 = 0x1999999a;

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
    tim.CR1.write(|w| w.CEN().set_bit());

    stm_common::interrupt::enable(INTERRUPT);
}

fn set_leds(leds: u64) {
    stm_common::interrupt::disable_all();
    *unsafe {LEDS.as_mut()} = leds;
    stm_common::interrupt::enable_all();
}

pub fn set_display(display: u64) {
    let mut leds = 0;
    for i in 0 .. 6 {
        let bits = display >> i * 8 & 0x3f;
        leds |= crate::leds::COLUMNS[i][bits as usize];
    }
    set_leds(leds);
    if crate::DEBUG_ENABLE {
        let strings = text::blocks(display);
        crate::dbgln!("{}{}{}\n{}{}{}",
                      strings[0], strings[1], strings[2],
                      strings[3], strings[4], strings[5]);
    }
}

fn pwm_controller_update() {
    // We use a simple proportional controller.  We track the VBAT_OK signal
    // and use a decaying average with a time constant of ≈4s.  That then
    // becomes the PWM duty cycle.
    let gpiof = unsafe {&*stm32g030::GPIOF::PTR};
    let vbat = unsafe {VBAT_OK_AVERAGE.as_mut()};
    let ok = gpiof.IDR.read().IDR1().bit();
    *vbat = VBAT_LOW.max(*vbat - (*vbat >> 12) + if ok {1 << 20} else {0});

    let pwm16 = *vbat >> 16;
    let tim = unsafe {&*TIM::PTR};
    tim.CCR1.write(|w| w.bits(pwm16 * EVEN_DUTY >> 16));
    tim.CCR2.write(|w| w.bits(PWM_DIV - (pwm16 * ODD_DUTY >> 16)));
}

fn isr() {
    let tim = unsafe {&*TIM::PTR};
    let sr = tim.SR.read();
    tim.SR.write(|w| w.bits(!sr.bits()));
    // First, update the LEDs, we want low timing jitter on this.
    let display = *LEDS;
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
