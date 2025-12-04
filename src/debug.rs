
pub use stm32g030::USART1 as UART;
pub use stm32g030::Interrupt::USART1 as INTERRUPT;

#[path = "../stm-common/debug_core.rs"]
pub mod debug_core;

use debug_core::{Debug, debug_isr};

/// Flag for global enable/disable of debugging.
pub const ENABLE: bool = !crate::CONFIG.no_debug;

/// State for debug logging.  We mark this as no-init and initialize the cells
/// ourselves, to avoid putting the buffer into BSS.
#[unsafe(link_section = ".noinit")]
static DEBUG: Debug = Debug::default();

const BAUD: u32 = 9600;

pub struct DebugMarker;
pub fn debug_marker() -> DebugMarker {DebugMarker}

fn lazy_init() {
    if !crate::CONFIG.is_lazy_debug() {
        return;
    }

    // Lazy initialization.
    let rcc = unsafe {&*stm32g030::RCC::ptr()};
    if !rcc.APBENR2.read().USART1EN().bit() {
        init();
    }
}

fn is_init() -> bool {
    let rcc = unsafe {&*stm32g030::RCC::ptr()};
    ENABLE && rcc.APBENR2.read().USART1EN().bit()
}

pub fn init() {
    check_vtors();
    let gpioa = unsafe {&*stm32g030::GPIOA::ptr()};
    let rcc   = unsafe {&*stm32g030::RCC::ptr()};
    let uart  = unsafe {&*UART::ptr()};

    rcc.APBENR2.modify(|_, w| w.USART1EN().set_bit());

    DEBUG.w.write(0);
    DEBUG.r.write(0);

    // Configure UART lines.
    gpioa.AFRH.modify(|_, w| w.AFSEL9().bits(8)); // FIXME.
    gpioa.MODER.modify(|_, w| w.MODER9().bits(2));

    // Set-up the UART TX.  TODO - we should enable RX at some point.  The dbg*
    // macros will work after this.

    const BRR: u32 = (crate::CONFIG.clk * 2 + BAUD) / 2 / BAUD;
    const {assert!(BRR > 100)};
    const {assert!(BRR < 65536)};
    uart.BRR.write(|w| w.bits(BRR)); // FIXME
    // uart.PRESC.write(|w| w.bits(0));
    uart.CR1.write(|w| w.FIFOEN().set_bit().TE().set_bit().UE().set_bit());

    if false {
        crate::dbg!("");
        crate::dbgln!();
        crate::dbgln!("");
    }
}

#[cfg(target_os = "none")]
#[panic_handler]
fn ph(info: &core::panic::PanicInfo) -> ! {
    crate::dbgln!("{info}");
    debug_core::flush();
    crate::cpu::reboot();
}

#[allow(dead_code)]
impl crate::cpu::Config {
    pub const fn debug(&mut self) -> &mut Self {
        self.lazy_debug() // .clocks(0, 1 << 20, 0)
    }
    pub const fn lazy_debug(&mut self) -> &mut Self {
        self.isr(INTERRUPT, debug_isr)
    }
    pub const fn no_debug(&mut self) -> &mut Self {
        self.no_debug = true;
        self
    }
    pub const fn is_lazy_debug(&self) -> bool {
        false
        //self.apb2_clocks & 1 << 20 == 0
    }
}

#[inline(always)]
#[cfg_attr(test, test)]
fn check_vtors() {
    // FIXME use crate::link_assert;
    if ENABLE {
        // FIXME link_assert! not assert!
        assert!(crate::cpu::VECTORS.isr[INTERRUPT as usize] == debug_isr);    }
}
