use stm_common::{interrupt::VectorTable, utils::barrier};

use crate::CONFIG;

pub fn init() {
    let pwr = unsafe {&*stm32g030::PWR::ptr()};
    let rcc = unsafe {&*stm32g030::RCC::ptr()};

    // Set the HSI16 divider...
    if CONFIG.clk < 16_000_000 {
        const DIV: u32 = 16_000_000 / CONFIG.clk;
        const {assert!(16_000_000 % CONFIG.clk == 0)};
        const {assert!(DIV.is_power_of_two())};
        const HSIDIV: u32 = DIV.ilog2();
        const {assert!(HSIDIV < 8)};
        rcc.CR.modify(|_, w| w.HSIDIV().bits(HSIDIV as u8));
    }

    if CONFIG.clk <= 2_000_000 {
        // Enter LP run mode, voltage range 2.
        pwr.CR1.modify(|_, w| w.LPR().set_bit().VOS().bits(2));
    }

    // Clear the BSS.
    if !cfg!(test) {
        barrier();
        // The rustc memset is hideous.
        let mut p = (&raw mut __bss_start) as *mut u32;
        loop {
            unsafe {*p = 0};
            p = p.wrapping_add(1);
            if p as *mut u8 >= &raw mut __bss_end {
                break;
            }
        }
        barrier();
    }
}

#[derive(Clone, Copy)]
pub struct Config {
    pub clk: u32,
    /// Turn off debug...
    pub no_debug: bool,
    pub vectors: VectorTable,
}

#[used]
#[unsafe(link_section = ".vectors")]
pub static VECTORS: VectorTable = CONFIG.vectors;

unsafe extern "C" {
    static mut __bss_start: u8;
    static mut __bss_end: u8;
    #[cfg(target_os = "none")]
    static end_of_ram: u8;
}

#[cfg(not(target_os = "none"))]
#[allow(non_upper_case_globals)]
static end_of_ram: u8 = 0;

impl Config {
    pub const fn new(clk: u32) -> Config {
        Config {
            clk, no_debug: false,
            vectors: VectorTable::new(
                    &raw const end_of_ram, crate::main, bugger),
        }
    }
    pub const fn isr(&mut self,
                     isr: stm32g030::Interrupt, handler: fn()) -> &mut Self {
        self.vectors.isr[isr as usize] = handler;
        // self.interrupts |= 1 << isr as u32;
        self
    }
}

unsafe extern "C" {
    #[link_name = "llvm.frameaddress"]
    fn frameaddress(level: i32) -> *const u8;
}

fn bugger() {
    let fp = unsafe {frameaddress(0)};
    // The exception PC is at +0x18, but then LLVM pushes an additional 8
    // bytes to form the frame.
    let pcp = fp.wrapping_add(0x20);
    let pc = unsafe {*(pcp as *const u32)};
    if false { // FIXME crate::CONFIG.low_power {
        let tamp = unsafe {&*stm32g030::TAMP::ptr()};
        tamp.BKPR[8].write(|w| w.bits(pc));
    }
    else {
        stm_common::dbgln!("Crash @ {pc:#010x}");
        stm_common::debug::flush::<crate::debug::DebugMeta>();
    }
    stm_common::utils::reboot();
}

#[inline(always)]
pub fn nothing() {
    unsafe {core::arch::asm!("", options(nomem))}
}
