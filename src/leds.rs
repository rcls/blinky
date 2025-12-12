

pub const _: () = {let _ = &COLUMNS;};

pub const COLUMNS: [[u64; 64]; 6] = gen_columns();

const fn gen_columns() -> [[u64; 64]; 6] {
    let mut value = [[0; _]; _];
    let mut column = 0;
    while column < 6 {
        let mut bits = 0;
        while bits < 64 {
            let val = &mut value[column][bits];
            let mut row = 0;
            while row < 6 {
                // The LED drive is negative logic.
                if bits & (1 << row) == 0 {
                    *val |= 1 << LEDS[column + row * 6];
                }
                row += 1;
            }
            bits += 1;
        }
        column += 1;
    }
    value
}

/// Bit mask of LED GPIOs.  One entry per-port, successive bits are LEDs.
pub const PORT_BITS: [u32; 4] = PORT_STUFF.0;
/// Like PORT_BITS, except only even numbered bits are used.
pub const PORT_BIT2: [u32; 4] = PORT_STUFF.1;
/// Like PORT_BITS, except only every fourth bit is used.
pub const _PORT_BIT4: [[u32; 2]; 4] = PORT_STUFF.2;

/// Return GPIOA ..= GPIOB based on the top two bits of n.
pub fn gpio(n: u8) -> &'static stm32g030::GPIOB {
    let address = 0x5000_0000 + 0x400 * (n >> 4) as usize;
    unsafe {&* (address as *const stm32g030::GPIOB)}
}

pub const PORT_STUFF: ([u32; 4], [u32; 4], [[u32; 2]; 4]) = {
    let mut mask = [0; 4];
    let mut bit2 = [0; 4];
    let mut bit4 = [[0, 2]; 4];
    let mut i = 0;
    while i < LEDS.len() {
        let l = LEDS[i] as usize;
        mask[l / 16] |= 1 << l % 16;
        bit2[l / 16] |= 1 << l % 16 * 2;
        bit4[l / 16][l % 16 / 8] |= 1 << l % 8 * 2;
        i += 1;
    }
    (mask, bit2, bit4)
};

pub const LED_EVEN: u64 = LED_EVEN_ODD.0;
pub const LED_ODD : u64 = LED_EVEN_ODD.1;

const LED_EVEN_ODD: (u64, u64) = {
    let (mut a, mut b) = (0, 0);
    let mut i = 0;
    while i < 6 {
        let mut j = 0;
        while j < 6 {
            (a, b) = (b, a | 1 << LEDS[i * 6 + j]);
            j += 1;
        }
        (a, b) = (b, a);
        i += 1;
    }
    (a, b)
};

const A: u8 = 0;
const B: u8 = 16;
const C: u8 = 32;
const D: u8 = 48;

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

#[test]
fn bit_counts() {
    assert_eq!(LED_ODD .count_ones(), 18);
    assert_eq!(LED_EVEN.count_ones(), 18);
    let led_all = LED_ODD | LED_EVEN;
    assert_eq!(led_all .count_ones(), 36);
}
