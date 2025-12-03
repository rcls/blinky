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
#[derive_const(Default)]
pub struct Config {
    pub clk: u32,
    pub vectors: VectorTable,
    /// Turn off debug...
    pub no_debug: bool,
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
            clk, .. Config::default()
        }
    }
    pub const fn isr(&mut self,
                     isr: stm32g030::Interrupt, handler: fn()) -> &mut Self {
        self.vectors.isr[isr as usize] = handler;
        // self.interrupts |= 1 << isr as u32;
        self
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct VectorTable {
    pub stack     : *const u8,
    pub reset     : fn() -> !,
    pub nmi       : fn(),
    pub hard_fault: fn(),
    pub reserved1 : [u32; 7],
    pub svcall    : fn(),
    pub reserved2 : [u32; 2],
    pub pendsv    : fn(),
    pub systick   : fn(),
    pub isr       : [fn(); 32],
}

/// !@#$!@$#
unsafe impl Sync for VectorTable {}

impl const Default for VectorTable {
    fn default() -> Self {
        VectorTable{
            stack     : &raw const end_of_ram,
            reset     : crate::main,
            nmi       : bugger,
            hard_fault: bugger,
            reserved1 : [0; 7],
            svcall    : bugger,
            reserved2 : [0; 2],
            pendsv    : bugger,
            systick   : bugger,
            isr       : [bugger; 32]}
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
        crate::dbgln!("Crash @ {pc:#010x}");
        crate::debug::debug_core::flush();
    }
    reboot();
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn WFE() {
    if cfg!(target_arch = "arm") {
        unsafe {
            core::arch::asm!("wfe", options(nomem, preserves_flags, nostack))};
    }
    else {
        panic!("wfe!");
    }
}

pub fn reboot() -> ! {
    loop {
        unsafe {(*cortex_m::peripheral::SCB::PTR).aircr.write(0x05fa0004)};
    }
}

#[inline(always)]
pub fn barrier() {
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}

#[inline(always)]
pub fn nothing() {
    unsafe {core::arch::asm!("", options(nomem))}
}

pub mod interrupt {
    // We don't use disabling interrupts to transfer ownership, so no need for
    // the enable to be unsafe.
    #[cfg(target_arch = "arm")]
    #[allow(unused)]
    pub fn enable_all() {unsafe{cortex_m::interrupt::enable()}}
    #[cfg(target_arch = "arm")]
    #[allow(unused)]
    pub fn disable_all() {cortex_m::interrupt::disable()}
    #[cfg(not(target_arch = "arm"))]
    #[allow(unused)]
    pub fn enable_all() { }
    #[cfg(not(target_arch = "arm"))]
    #[allow(unused)]
    pub fn disable_all() { }
}
