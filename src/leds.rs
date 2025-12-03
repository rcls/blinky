
const A: u8 = 0;
const B: u8 = 64;
const C: u8 = 128;
const D: u8 = 192;

/// Return GPIOA ..= GPIOB based on the top two bits of n.
pub fn gpio(n: u8) -> &'static stm32g030::GPIOB {
    let address = 0x5000_0000 + 0x400 * (n >> 6) as usize;
    unsafe {&* (address as *const stm32g030::GPIOB)}
}

#[inline(never)]
pub const fn bit(n: u8) -> u32 {
    1 << (n & 63)
}

pub const PORT_STUFF: ([u32; 4], [u32; 4], [[u32; 2]; 4]) = {
    let mut mask = [0; 4];
    let mut bit2 = [0; 4];
    let mut bit4 = [[0, 2]; 4];
    let mut i = 0;
    while i < LEDS.len() {
        let l = LEDS[i] as usize;
        mask[l / 64] |= 1 << l % 16;
        bit2[l / 64] |= 1 << l % 16 * 2;
        bit4[l / 64][l % 16 / 8] |= 1 << l % 8 * 2;
        i += 1;
    }
    (mask, bit2, bit4)
};

pub const PORT_BITS: [u32; 4] = PORT_STUFF.0;
pub const PORT_BIT2: [u32; 4] = PORT_STUFF.1;
// pub const PORT_BIT4: [[u32; 2]; 4] = PORT_STUFF.2;

pub static LEDS: [u8; 36] = [
    B +  6, B +  5, B + 4, B +  3, D +  3, D +  2,
    C + 13, B +  8, B + 7, D +  1, D +  0, A + 12,
    C + 15, C + 14, B + 9, A + 15, A + 11, A + 10,
    A +  8, A +  0, A + 2, B + 12, B + 14, B + 15,
    A +  1, A +  3, A + 4, B + 10, B + 11, B + 13,
    A +  5, A +  6, A + 7, B +  0, B +  1, B +  2,
];

macro_rules! row {
    ($n:expr,) => {};
    ($n:expr, $x:ident $($y:ident)*) => {
        #[allow(unused)]
        pub const $x: u8 = LEDS[$n];
        row!($n + 1, $($y)*);
    };
}

row!( 0, L11 L12 L13 L14 L15 L16 L21 L22 L23 L24 L25 L26);
row!(12, L31 L32 L33 L34 L35 L36 L41 L42 L43 L44 L45 L46);
row!(24, L51 L52 L53 L54 L55 L56 L61 L62 L63 L64 L65 L66);

#[test]
fn unique() {
    let mut leds = LEDS;
    leds.sort();
    for i in 1 .. leds.len() {
        assert_ne!(leds[i], leds[i - 1]);
    }
}