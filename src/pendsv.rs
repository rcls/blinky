use stm_common::vcell::{UCell, VCell};

mod text;

/// Number of application wake-ups per second.
pub const SECOND: u32 = 5;
/// Fifth of a second...
pub const FIFTH: u32 = 1;
/// Number of PWM cycles per tick.
pub const CYCLES_PER_TICK: u32 = crate::pulse::RATE / SECOND;

/// Trigger count from PWM.
static COUNT: VCell<i32> = VCell::new(0);

/// Application count (/8) on the trigger count.
static APP_COUNT: VCell<i32> = VCell::new(0);

/// LEDs to display next tick.
static NEXT_LEDS: UCell<u64> = UCell::new(0);

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

#[inline(never)]
pub fn hold_display(display: u64, wait: u32) {
    set_display(display);
    sleep(wait);
}

pub fn animate(list: &[u64], wait: u32) {
    for &display in list {
        hold_display(display, wait);
    }
}

pub fn set_display(display: u64) {
    let mut leds = 0;
    let mut d = display;
    for i in 0 .. 6 {
        leds |= crate::leds::COLUMNS[i][d as usize & 0x3f];
        d >>= 8;
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

#[cold]
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

        const {assert!(CYCLES_PER_TICK.is_power_of_two())};
        let phase = *alloc as u32 & (CYCLES_PER_TICK - 1);
        const ADC2: u32 = CYCLES_PER_TICK / 2;
        match phase {
            1 => {
                // Trigger the app.
                APP_COUNT.write(APP_COUNT.read().wrapping_add(1));
            }
            0|ADC2 => {
                crate::adc::power_up();
                // We are already past the point in the tick where we use the
                // LED setting.  So set the next one.
                if phase == 0 {
                    crate::pulse::apply_leds(*NEXT_LEDS);
                }
                // Run the ADC conversion.
                crate::adc::start();
            }
            _ => ()
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
