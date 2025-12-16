
use stm32g030::Interrupt::ADC as INTERRUPT;

use crate::pulse::PWM_DIV;

pub const OVER3: u32 = 273;
pub const UNDER3: u32 = 273;

macro_rules! dbgln {($($tt: tt)*) => {if false {stm_common::dbgln!($($tt)*)}}}

pub fn power_up() {
    let adc = unsafe {&*stm32g030::ADC::PTR};
    let rcc = unsafe {&*stm32g030::RCC::PTR};
    rcc.APBENR2.modify(|_, w| w.ADCEN().set_bit());

    // Enable the voltage regulator.  After this we need to wait ≈20µs.
    // At 250kHz, this is 5 clock cycles...
    adc.CR.write(|w| w.ADVREGEN().set_bit());

    // Turn ADC off once we're done.  FIXME or manual?
    //adc.CFGR1.write(|w| w.AUTOFF().set_bit());
    // Set the ADC clock source to APB.
    adc.CFGR2.write(|w| w.CKMODE().set(3).LFTRIG().set_bit());
}

pub fn start() {
    if crate::CONFIG.clk > 250_000 {
        // If we're running fast then waste some time waiting for the ADC
        // power-up.
        for _ in 0 .. crate::CONFIG.clk / 250_000 {
            stm_common::utils::nothing();
        }
    }
    let adc = unsafe {&*stm32g030::ADC::PTR};
    adc.CCR.write(|w| w.VREFEN().set_bit());
    adc.CHSELR_0().write(|w| w.CHSEL13().set_bit());
    adc.IER.write(
        |w| w.EOSIE().set_bit().EOCALIE().set_bit().ADRDYIE().set_bit());
    // Start the calibration....
    adc.CR.write(|w| w.ADVREGEN().set_bit().ADCAL().set_bit());

    // Enable the interrupt.
    stm_common::interrupt::enable_priority(INTERRUPT, crate::cpu::PRIO_PENDSV);
}

pub fn isr() {
    let adc = unsafe {&*stm32g030::ADC::PTR};
    let rcc = unsafe {&*stm32g030::RCC::PTR};

    let isr = adc.ISR.read();
    dbgln!("ADC ISR {:#x}", isr.bits());
    // Clear interrupts.
    adc.ISR.write(|w| w.set(isr.bits()));
    if isr.EOCAL().bit() {
        dbgln!("Cal done, enable");
        // Enable the ADC!!!
        adc.CR.write(|w| w.ADVREGEN().set_bit().ADEN().set_bit());
    }
    if isr.ADRDY().bit() {
        // Start a calibration.
        dbgln!("Ready.  Start.");
        adc.CR.write(
            |w| w.ADVREGEN().set_bit().ADEN().set_bit().ADSTART().set_bit());
    }
    if isr.EOS().bit() {
        dbgln!("Conv done, off");
        // Turn off the ADC.
        adc.CCR.write(|w| w.VREFEN().clear_bit());
        adc.CR.write(|w| w.ADDIS().set_bit());
        adc.CR.write(|w| w.bits(0));
        // Get the result.
        let counts = adc.DR.read().bits();
        // Stop the ADC clock.
        rcc.APBENR2.modify(|_, w| w.ADCEN().clear_bit());
        // Compare with flash config & convert to mV offset.
        let cal = unsafe{*(0x1fff75aa as *const u16)} as u32;
        let top = cal + OVER3;
        let max = OVER3 + UNDER3;
        let delta = if top > counts {(top - counts).min(max)} else {0};
        let duty = calc_duty(delta);
        crate::pulse::set_duty(duty);

        let delta = cal as i32 - counts as i32;
        const SCALE_F: f64 = 3000.0 * 3000.0 / 1212.0 / 4096.0;
        const SCALE: i32 = (SCALE_F * 65536.0 + 0.5) as i32;
        const {assert!(SCALE > 65536)};
        const {assert!(SCALE < 1000000)};
        let offset = (delta * SCALE) >> 16;
        // Log the counts...
        dbgln!("ADC {counts} {delta} {offset}");
    }
}

/// Go from 50% duty at delta==0 to 2.5% duty at delta ≈ OVER3 + UNDER3.
fn calc_duty(delta: u32) -> u32 {
    const MAX: u32 = crate::adc::OVER3 + crate::adc::UNDER3;
    const RANGE: u32 = PWM_DIV / 2 - PWM_DIV / 40;
    const SCALE: u32 = (RANGE * 65536).div_ceil(MAX);
    (SCALE * delta >> 16) + PWM_DIV / 40
}

#[test]
fn test_pwm_duty() {
    let max = crate::adc::OVER3 + crate::adc::UNDER3;
    assert!(calc_duty(max) == PWM_DIV / 2);
    assert!(calc_duty(0) == PWM_DIV / 40);
    assert!(calc_duty(0) > 100);
}

impl crate::cpu::Config {
    pub const fn adc(&mut self) -> &mut Self {
        self.vectors.isr(INTERRUPT, isr);
        self
    }
}

#[test]
fn check_isr() {
    assert!(crate::cpu::VECTORS.isr[INTERRUPT as usize] == isr);
}
