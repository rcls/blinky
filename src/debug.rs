
pub use stm32g030::Interrupt::USART1 as INTERRUPT;

use stm_common::{debug, interrupt, link_assert};
use debug::{Debug, Meta};

use crate::{CONFIG, DEBUG_ENABLE};

#[derive_const(Default)]
pub struct DebugMeta;

impl Meta for DebugMeta {
    fn debug() -> &'static Debug<Self> {&DEBUG}

    fn uart(&self) -> &'static debug::UART {unsafe{&*stm32g030::USART1::PTR}}

    fn lazy_init(&self) {
        let rcc = unsafe {&*stm32g030::RCC::ptr()};
        if CONFIG.is_lazy_debug() && !rcc.APBENR2.read().USART1EN().bit() {
            init();
        }
    }

    fn is_init(&self) -> bool {
        let rcc = unsafe {&*stm32g030::RCC::ptr()};
        !CONFIG.is_lazy_debug()
            || DEBUG_ENABLE && rcc.APBENR2.read().USART1EN().bit()
    }

    fn interrupt(&self) -> u32 {INTERRUPT as u32}

    const ENABLE: bool = DEBUG_ENABLE;
}

/// State for debug logging.  We mark this as no-init and initialize the cells
/// ourselves, to avoid putting the buffer into BSS.
#[unsafe(link_section = ".noinit")]
static DEBUG: Debug<DebugMeta> = Default::default();

fn debug_isr() {
    DEBUG.isr();
}

const BAUD: u32 = 9600;

pub fn init() {
    check_vtors();
    let gpioa = unsafe {&*stm32g030::GPIOA::ptr()};
    let rcc   = unsafe {&*stm32g030::RCC::ptr()};
    let uart = DebugMeta.uart();

    rcc.APBENR2.modify(|_, w| w.USART1EN().set_bit());

    DEBUG.w.write(0);
    DEBUG.r.write(0);

    // Configure UART lines.
    gpioa.AFRH.modify(|_, w| w.AFSEL9().bits(8)); // FIXME.
    gpioa.MODER.modify(|_, w| w.MODER9().bits(2));

    // Set-up the UART TX.  TODO - we should enable RX at some point.  The dbg*
    // macros will work after this.

    const BRR: u32 = (CONFIG.clk * 2 + BAUD) / 2 / BAUD;
    const {assert!(BRR > 100)};
    const {assert!(BRR < 65536)};
    uart.BRR.write(|w| w.bits(BRR)); // FIXME
    // uart.PRESC.write(|w| w.bits(0));
    uart.CR1.write(|w| w.FIFOEN().set_bit().TE().set_bit().UE().set_bit());

    interrupt::enable(INTERRUPT);

    if false {
        stm_common::dbg!("{}", 1);
        stm_common::dbgln!();
        stm_common::dbgln!("{}", 1);
    }
}

#[cfg(target_os = "none")]
#[panic_handler]
fn ph(info: &core::panic::PanicInfo) -> ! {
    stm_common::dbgln!("{info}");
    debug::flush::<DebugMeta>();
    stm_common::utils::reboot();
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
    if DEBUG_ENABLE {
        link_assert!(crate::cpu::VECTORS.isr[INTERRUPT as usize] == debug_isr);
    }
}
