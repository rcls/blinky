use stm_common::vcell::{UCell, VCell};

/// Number of wake-ups per second.  Note that the /8 is hardwired below...
pub const SECOND: u32 = crate::pulse::RATE / 8;

/// Trigger count from PWM.
static COUNT: VCell<i32> = VCell::new(0);

/// Application count (/8) on the trigger count.
static APP_COUNT: VCell<i32> = VCell::new(0);

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

fn pendsv_handler() {
    static ALLOC: UCell<i32> = UCell::new(0);
    let alloc = unsafe {ALLOC.as_mut()};
    // We loop just in case we miss a tick.  Or get a spurious wake-up.
    while alloc.wrapping_sub(COUNT.read()) < 0 {
        *alloc += 1;

        match *alloc & 7 {
            0 => {
                // Trigger the app in plenty of time for the next app. tick.
                APP_COUNT.write(APP_COUNT.read().wrapping_add(1));
            },

            7 => {
                crate::adc::power_up();
                crate::pulse::apply_leds();
                crate::adc::start();
            },
            _ => (),
        }
    }
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
