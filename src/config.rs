use stm_common::vcell::UCell;

use crate::pulse::PWM_DIV;

const PWM_MIN: u32 = 80;
const PWM_MAX: u32 = PWM_DIV / 2;

const CPU_ID: *const [UCell<u32>; 3] = 0x1fff7590 as _;

pub static CONFIG: UCell<Config> = UCell::default();

pub static CONFIGS: [([u32; 3], Config); 4] = [
    // Led test board.
    ([0x004c0072, 0x3245500b, 0x2031374c], Config::new(PWM_MAX)),
    // Slated for red.
    ([0, 0, 0], Config::new(PWM_MAX / 2)),
    // Orig white board.
    ([0x004c007b, 0x3245500b, 0x2031374c], Config::new(PWM_MAX)),
    // Orig blue board.
    ([0x004c0058, 0x3245500b, 0x2031374c], Config::new(PWM_MAX)),
];

pub static GENERIC: Config = Config::new(PWM_MAX);

#[derive(Clone, Copy)]
#[derive_const(Default)]
pub struct Config {
    pub adc_over: u16,
    pub adc_max: u16,
    pub pwm_scale: u32,
}

pub fn get() -> &'static Config {CONFIG.as_ref()}

pub fn generate_config() {
    let cpu_id = get_cpu_id();
    for (cpu, config) in &CONFIGS {
        if cpu_id[0] == cpu[0] && cpu_id[1] == cpu[1] && cpu_id[2] == cpu[2] {
            *unsafe {CONFIG.as_mut()} = *config;
            return;
        } 
    }
    *unsafe {CONFIG.as_mut()} = GENERIC;
}

fn get_cpu_id() -> [u32; 3] {
    let cpu_id = unsafe {&*CPU_ID};
    [*cpu_id[0], *cpu_id[1], *cpu_id[2]]
}

impl Config {
    const fn new(pwm_max: u32) -> Config {
        let max       = crate::adc::OVER3 + crate::adc::UNDER3;
        let adc_over  = crate::adc::OVER3;
        let adc_max   = crate::adc::OVER3 + crate::adc::UNDER3;
        let range     = pwm_max - PWM_MIN;
        let pwm_scale = (range * 65536).div_ceil(max);

        let Ok(adc_over) = adc_over.try_into() else {panic!()};
        let Ok(adc_max)  = adc_max .try_into() else {panic!()};

        let c = Config {adc_over, adc_max, pwm_scale};
        c.check(pwm_max);
        c
    }

    pub const fn calc_duty(&self, delta: u32) -> u32 {
        (self.pwm_scale * delta >> 16) + PWM_MIN
    }

    const fn check(&self, pwm_max: u32) {
        assert!(self.calc_duty(self.adc_max as u32) == pwm_max);
        assert!(self.calc_duty(0) == PWM_MIN);
        assert!(self.calc_duty(0) >= 78);
    }
}
