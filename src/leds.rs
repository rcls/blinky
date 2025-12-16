

pub const _: () = {let _ = &COLUMNS;};

pub static COLUMNS: [[u64; 64]; 6] = gen_columns();

const fn gen_columns() -> [[u64; 64]; 6] {
    let mut value = [[0; _]; _];
    let mut column = 0;
    while column < 6 {
        let mut bits = 0;
        while bits < 64 {
            let val = &mut value[column][bits];
            let mut row = 0;
            while row < 6 {
                if bits & (1 << row) != 0 {
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

/// Return GPIOA ..= GPIOD based on n.
pub fn gpio(n: u8) -> &'static stm32g030::gpiob::RegisterBlock {
    let address = 0x5000_0000 + 0x400 * n as usize;
    unsafe {&* (address as *const stm32g030::gpiob::RegisterBlock)}
}

pub const PORT_STUFF: ([u32; 4], [u32; 4], [[u32; 2]; 4]) = {
    let mut mask = [0; 4];
    let mut bit2 = [0; 4];
    let mut bit4 = [[0, 2]; 4];
    let mut i = 0;
    while i < LEDS.len() {
        let led = LEDS[i] as usize;
        mask[led / 16] |= 1 << led % 16;
        bit2[led / 16] |= 1 << led % 16 * 2;
        bit4[led / 16][led % 16 / 8] |= 1 << led % 8 * 4;
        i += 1;
    }
    (mask, bit2, bit4)
};

pub const LED_EVEN: u64 = LED_EVEN_ODD.0;
pub const LED_ODD : u64 = LED_EVEN_ODD.1;
pub const LED_ALL : u64 = LED_EVEN | LED_ODD;

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

// First itema in the array are D16 .. D11, note the reverse column order.
pub static LEDS: [u8; 36] = [
    D +  2, D +  3, B +  3, B + 4, B +  5, B +  6,
    A + 12, D +  0, D +  1, B + 7, B +  8, C + 13,
    A + 10, A + 11, A + 15, B + 9, C + 14, C + 15,
    B + 15, B + 14, B + 12, A + 2, A +  0, A +  8,
    B + 13, B + 11, B + 10, A + 4, A +  3, A +  1,
    B +  2, B +  1, B +  0, A + 7, A +  6, A +  5,
];

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
    assert_eq!(LED_ALL .count_ones(), 36);
    assert_eq!(LED_ODD & LED_EVEN, 0);
}

#[test]
fn sane_columns() {
    for column in COLUMNS {
        for (i, x) in column.iter().enumerate() {
            assert_eq!(i.count_ones(), x.count_ones());
            for (j, y) in column.iter().enumerate() {
                assert_eq!(column[i | j], x | y);
            }
        }
    }
}