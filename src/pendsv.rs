use stm_common::vcell::{UCell, VCell};

use crate::{leds::{LED_EVEN, LED_ODD}, pulse::{CLOCKS_PER_TICK, PWM_DIV}};

mod text;

/// Number of application wake-ups per second.
pub const SECOND: u32 = 5;
pub const PWM_PER_TICK: u32 = crate::pulse::RATE / SECOND;

/// Trigger count from PWM.
static COUNT: VCell<i32> = VCell::new(0);

/// Application count (/8) on the trigger count.
static APP_COUNT: VCell<i32> = VCell::new(0);

/// LEDs to display next tick.
static NEXT_LEDS: UCell<u64> = UCell::new(0);

#[derive(Copy, Clone)]
#[derive_const(Default)]
struct PendingDuty {
    pending: u32,
    combine: bool,
}
/// Duty cycle to apply next tick.
static PENDING_DUTY: UCell<PendingDuty> = UCell::default();

macro_rules! dbgln {($($tt: tt)*) => {if false {stm_common::dbgln!($($tt)*)}}}

pub fn init() {
    // We use the PENDSV exception to dispatch some work at lower priority.
    let scb = unsafe {&*cortex_m::peripheral::SCB::PTR};
    let pendsv_prio = &scb.shpr[1];
    // Cortex-M crate has two different ideas of what the SHPR is, make sure we
    // are built with the correct one.
    stm_common::link_assert!(pendsv_prio as *const _ as usize == 0xe000ed20);
    #[cfg(target_os = "none")]
    unsafe {pendsv_prio.write(crate::cpu::PRIO_PENDSV as u32 * 65536)};
}

pub fn set_display(display: u64) {
    let mut leds = 0;
    let mut d = display;
    for i in 0 .. 6 {
        leds |= crate::leds::COLUMNS[i][d as usize & 0xff];
        d = shr8(d);
    }
    stm_common::interrupt::disable_all();
    *unsafe {NEXT_LEDS.as_mut()} = leds;
    stm_common::interrupt::enable_all();
    if crate::DEBUG_ENABLE {
        let strings = text::blocks(display);
        dbgln!("{}{}{}\n{}{}{}",
               strings[0], strings[1], strings[2],
               strings[3], strings[4], strings[5]);
    }
}

pub fn trigger() {
    COUNT.write(COUNT.read().wrapping_add(1));
    cortex_m::peripheral::SCB::set_pendsv();
}

pub fn sleep(wait: u32) {
    static ALLOC: UCell<i32> = UCell::new(0);
    let target = ALLOC.wrapping_add(wait as i32);
    unsafe {*ALLOC.as_mut() = target};
    dbgln!("Sleep for {target}");
    while APP_COUNT.read().wrapping_sub(target) < 0 {
        stm_common::utils::WFE();
    }
    dbgln!("Wakes");
}

pub fn store_duty(total: u32) {
    let pending = unsafe {PENDING_DUTY.as_mut()};
    let neg = !*NEXT_LEDS & (LED_EVEN | LED_ODD);
    if total > CLOCKS_PER_TICK / 8 {
        pending.pending = total / PWM_PER_TICK;
        pending.combine = false;
        crate::pulse::apply_leds(neg | LED_ODD, neg | LED_EVEN);
    }
    else {
        pending.pending = total;
        pending.combine = true;
        crate::pulse::apply_leds(!0, neg);
    }
}

fn update_duty() {
    let pending = unsafe {PENDING_DUTY.as_mut()};
    if pending.combine {
        let todo = pending.pending.min(PWM_DIV);
        pending.pending -= todo;
        crate::pulse::set_duty(0, todo);
    }
    else {
        crate::pulse::set_duty(pending.pending, pending.pending * 2);
    }
}

fn pendsv_handler() {
    static ALLOC: UCell<i32> = UCell::new(0);
    let alloc = unsafe {ALLOC.as_mut()};
    // We loop just in case we miss a tick.  Or get a spurious wake-up.
    while alloc.wrapping_sub(COUNT.read()) < 0 {
        *alloc += 1;
        update_duty();

        const {assert!(PWM_PER_TICK.is_power_of_two())};
        match *alloc as u32 & (PWM_PER_TICK - 1) {
            1 => {
                // Trigger the app.
                APP_COUNT.write(APP_COUNT.read().wrapping_add(1));
            }
            0 => {
                // Prep. for the next app tick.
                crate::adc::power_up();
                crate::adc::start();
            }
            _ => ()
        }
    }
}

/// rustc generates bloated out-of-line code.  We don't want that.
#[inline]
pub const fn shr8(val: u64) -> u64 {
    let (lo, hi) = (val as u32, (val >> 32) as u32);
    let (lo, hi) = (lo >> 8 | hi << 24, hi >> 8);
    lo as u64 | (hi as u64) << 32
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

#[test]
fn test_shr8() {
    for i in 0 .. 64 {
        for j in 0 .. 64 {
            let x = 1 << i | 1 << j;
            assert_eq!(shr8(x), x >> 8);
        }
    }
}
