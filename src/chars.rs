
pub type Picture = [&'static str; 6];

pub const IDOTS: [u64; 3] = parse_array(&[IDOTS0, IDOTS1, IDOTS2]);
pub const ODOTPS: [u64; 5] = parse_array(
    &[ODOTS0, ODOTS1, ODOTS2, ODOTS3, ODOTS4]);

pub const NUM_CHARS: usize = PICTURES.len();
pub static COLUMNS: [u64; NUM_CHARS] = parse_array(PICTURES);

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

const fn parse_array<const N: usize>(p: &[Picture]) -> [u64; N] {
    assert!(N == p.len());
    let mut array = [0; _];
    let mut i = 0;
    while i < N {
        array[i] = parse(&p[i]);
        i += 1;
    }
    array
}

pub const fn map_str<const N: usize>(s: &[u8; N]) -> [u8; N] {
    let mut result = [0; _];
    let mut i = 0;
    while i < s.len() {
        result[i] = map_char(s[i]);
        i += 1;
    }
    result
}

pub const fn map_char(c: u8) -> u8 {
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
    low as u8
}

pub const fn picture(c: char) -> u64 {
    assert!(c == c as u8 as char);
    COLUMNS[map_char(c as u8) as usize]
}

pub const CHARS: &[u8] = b" 0123456789?ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const PICTURES: &'static [Picture] = &[
    SPC, D0, D1, D2, D3, D4, D5, D6, D7, D8, D9, QUESTION,
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

const QUESTION: Picture = [
    " ***  ",
    "*   * ",
    "   *  ",
    "  *   ",
    "      ",
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

pub const CDOT: u64 = parse(&[
    "      ",
    "      ",
    "  **  ",
    "  **  ",
    "      ",
    "      ",
]);

const IDOTS0: Picture = [
    "      ",
    "  *   ",
    "    * ",
    " *    ",
    "   *  ",
    "      ",
];

const IDOTS1: Picture = [
    "      ",
    "   *  ",
    " *    ",
    "    * ",
    "  *   ",
    "      ",
];
const IDOTS2: Picture = [
    "      ",
    " *  * ",
    "      ",
    "      ",
    " *  * ",
    "      ",
];
pub const MDOTS: u64 = parse(&[
    " *    ",
    " *  **",
    "      ",
    "      ",
    "**  * ",
    "    * ",
]);
const ODOTS0: Picture = [
    " **   ",
    "     *",
    "     *",
    "*     ",
    "*     ",
    "   ** ",
];
const ODOTS1: Picture = [
    "  **  ",
    "      ",
    "*    *",
    "*    *",
    "      ",
    "  **  ",
];
const ODOTS2: Picture = [
    "   ** ",
    "*     ",
    "*     ",
    "     *",
    "     *",
    " **   ",
];
const ODOTS3: Picture = [
    "*   **",
    "*     ",
    "      ",
    "      ",
    "     *",
    "**   *",
];
const ODOTS4: Picture = [
    "**   *",
    "     *",
    "      ",
    "      ",
    "*     ",
    "*   **",
];
pub const CORNERS: u64 = parse(&[
    "*    *",
    "      ",
    "      ",
    "      ",
    "      ",
    "*    *",
]);
pub const LOVE: u64 = parse(&[
    " * *  ",
    "* * * ",
    "*   * ",
    "*   * ",
    " * *  ",
    "  *   ",
]);


#[test]
fn chars_in_order() {
    // We do binary searches so make sure we get it right...
    for (i, &b) in CHARS[1 ..].iter().enumerate() {
        assert!(CHARS[i] < b, "{i} '{b}'");
    }
}
