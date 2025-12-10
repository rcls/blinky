
const _: [u64; NUM_CHARS] = COLUMNS;
const _: &[u8] = CHARS;

pub type Picture = [&'static str; 6];

pub const NUM_CHARS: usize = PICTURES.len();

const COLUMNS: [u64; NUM_CHARS] = {
    let mut value = [0; _];
    let mut i = 0;
    while i < NUM_CHARS {
        value[i] = parse(&PICTURES[i]);
        i += 1;
    }
    value
};

const fn parse(p: &Picture) -> u64 {
    let mut columns = 0;
    assert!(p.len() == 6);
    let mut r = 0;
    while r < p.len() {
        let row = p[r].as_bytes();
        assert!(row.len() == 6);
        let mut c = 0;
        while c < row.len() {
            let ch = row[c];
            assert!(ch == b' ' || ch == b'*');
            if ch == b'*' {
                columns |= 1 << c * 8 + r;
            }
            c += 1;
        }
        r += 1;
    }
    columns
}

const fn map_str<const N: usize>(s: &[u8; N]) -> [u8; N] {
    let mut result = [0; _];
    let i = 0;
    while i < s.len() {
        let c = s[i];
        let mut low = 0;
        let mut high = CHARS.len();
        while high - low > 1 {
            let mid = (high + low) / 2;
            if c < CHARS[mid] {
                high = mid;
            }
            else {
                low = mid;
            }
        }
        assert!(CHARS[low] == c);
        result[i] = low as u8;
    }
    result
}

pub const CHARS: &[u8] = b" 0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const PICTURES: &'static [Picture] = &[
    SPC, D0, D1, D2, D3, D4, D5, D6, D7, D8, D9,
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
];

const SPC: Picture = [
    "      ",
    "      ",
    "      ",
    "      ",
    "      ",
    "      ",
];

const D0: Picture = [
    " ***  ",
    "*   * ",
    "*   * ",
    "*   * ",
    "*   * ",
    " ***  ",
];

const D1: Picture = [
    "  *   ",
    " **   ",
    "* *   ",
    "  *   ",
    "  *   ",
    "***** ",
];

const D2: Picture = [
    " ***  ",
    "*   * ",
    "   *  ",
    "  *   ",
    " *    ",
    "***** ",
];

const D3: Picture = [
    " ***  ",
    "*   * ",
    "  **  ",
    "    * ",
    "*   * ",
    " ***  ",
];

const D4: Picture = [
    "   *  ",
    "  *   ",
    " *    ",
    "*  *  ",
    "***** ",
    "   *  ",
];

const D5: Picture = [
    "***** ",
    "*     ",
    "****  ",
    "    * ",
    "*   * ",
    " ***  ",
];

const D6: Picture = [
    " **** ",
    "*     ",
    "****  ",
    "*   * ",
    "*   * ",
    " ***  ",
];

const D7: Picture = [
    "***** ",
    "    * ",
    "   *  ",
    "  *   ",
    " *    ",
    "*     ",
];

const D8: Picture = [
    " ***  ",
    "*   * ",
    " ***  ",
    "*   * ",
    "*   * ",
    " ***  ",
];

const D9: Picture = [
    " ***  ",
    "*   * ",
    "*   * ",
    " **** ",
    "   *  ",
    "  *   ",
];

const A: Picture = [
    "  *   ", 
    " * *  ",
    "*   * ",
    "***** ",
    "*   * ",
    "*   * ",
];

const B: Picture = [
    "****  ", 
    "*   * ",
    "****  ",
    "*   * ",
    "*   * ",
    "****  ",
];

const C: Picture = [
    " ***  ",
    "*   * ",
    "*     ",
    "*     ",
    "*   * ",
    " ***  ",
];

const D: Picture = [
    "****  ",
    "*   * ",
    "*   * ",
    "*   * ",
    "*   * ",
    "****  ",
];

const E: Picture = [
    "***** ",
    "*     ",
    "***   ",
    "*     ",
    "*     ",
    "***** ",
];

const F: Picture = [
    "***** ",
    "*     ",
    "****  ",
    "*     ",
    "*     ",
    "*     ",
];

const G: Picture = [
    " ***  ",
    "*   * ",
    "*     ",
    "*  ** ",
    "*   * ",
    " ***  ",
];

const H: Picture = [
    "*   * ",
    "*   * ",
    "***** ",
    "*   * ",
    "*   * ",
    "*   * ",
];

const I: Picture = [
    "***** ",
    "  *   ",
    "  *   ",
    "  *   ",
    "  *   ",
    "***** ",
];

const J: Picture = [
    "***** ",
    "   *  ",
    "   *  ",
    "   *  ",
    "*  *  ",
    " **   ",
];

const K: Picture = [
    "*   * ",
    "*  *  ",
    "* *   ",
    "**    ",
    "* *   ",
    "*  *  ",
];

const L: Picture = [
    "*     ",
    "*     ",
    "*     ",
    "*     ",
    "*     ",
    "***** ",
];

const M: Picture = [
    "*   * ",
    "** ** ",
    "* * * ",
    "*   * ",
    "*   * ",
    "*   * ",
];

const N: Picture = [
    "*   * ",
    "**  * ",
    "* * * ",
    "*  ** ",
    "*   * ",
    "*   * ",
];

const O: Picture = [
    " ***  ",
    "*   * ",
    "*   * ",
    "*   * ",
    "*   * ",
    " ***  ",
];

const P: Picture = [
    "****  ",
    "*   * ",
    "*   * ",
    "****  ",
    "*     ",
    "*     ",
];

const Q: Picture = [
    " ***  ",
    "*   * ",
    "*   * ",
    "*   * ",
    "*  *  ",
    " ** * ",
];

const R: Picture = [
    "****  ",
    "*   * ",
    "*   * ",
    "****  ",
    "* *   ",
    "*  *  ",
];

const S: Picture = [
    " ***  ",
    "*   * ",
    " **   ",
    "   *  ",
    "*   * ",
    " ***  ",
];

const T: Picture = [
    "***** ",
    "  *   ",
    "  *   ",
    "  *   ",
    "  *   ",
    "  *   ",
];

const U: Picture = [
    "*   * ",
    "*   * ",
    "*   * ",
    "*   * ",
    "*   * ",
    " ***  ",
];

const V: Picture = [
    "*   * ",
    "*   * ",
    "*   * ",
    "*   * ",
    " * *  ",
    "  *   ",
];

const W: Picture = [
    "*   * ",
    "*   * ",
    "*   * ",
    "* * * ",
    "* * * ",
    " * *  ",
];

const X: Picture = [
    "*   * ",
    "*   * ",
    " * *  ",
    "  *   ",
    " * *  ",
    "*   * ",
];

const Y: Picture = [
    "*   * ",
    "*   * ",
    " * *  ",
    "  *   ",
    "  *   ",
    "  *   ",
];

const Z: Picture = [
    "***** ",
    "   *  ",
    "  *   ",
    " *    ",
    "*     ",
    "***** ",
];

#[test]
fn chars_in_order() {
    // We do binary searches so make sure we get it right...
    for (i, &b) in CHARS[1 ..].iter().enumerate() {
        assert!(CHARS[i] < b, "{i} '{b}'");
    }
}