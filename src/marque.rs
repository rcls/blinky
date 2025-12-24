use crate::pendsv::{self, FIFTH, hold_display};

fn marque_display(current: &mut u64, mut new: u64, wait: u32) {
    for _ in 0 .. 6 {
        *current += (new & 255) << 48;
        *current >>= 8;
        new >>= 8;
        pendsv::hold_display(*current, wait);
    }
}

pub fn marque_string(current: &mut u64, s: &[u8], wait: u32) {
    for &c in s {
        marque_display(current, crate::chars::COLUMNS[c as usize], wait);
    }
}

fn vmarque_display(current: &mut u64, mut new: u64, wait: u32) {
    *current = shr_parts(*current, 1);
    pendsv::hold_display(*current, wait);

    for _ in 0 .. 6 {
        *current |= shl_parts(new & 0x0101_0101_0101, 6);
        *current = shr_parts(*current, 1);
        *current &= 0x3f3f_3f3f_3f3f_3f3f;
        new = shr_parts(new, 1);
        pendsv::hold_display(*current, wait);
    }
}

fn vmarque_string(current: &mut u64, s: &[u8], wait: u32) {
    for &c in s {
        vmarque_display(current, crate::chars::COLUMNS[c as usize], wait);
    }
}

pub fn rmarque_string(s: &[u8], wait: u32) {
    let r = crate::random::RANDOM.random_n(4);
    let mut tail = s;
    let mut current = 0;
    if r & 1 != 0 {
        // Blink the first character...
        let c = crate::chars::COLUMNS[s[0] as usize];
        for _ in 0 .. 3 {
            hold_display(c, FIFTH);
            hold_display(0, FIFTH);
        }
        current = c;
        hold_display(c, FIFTH);
        tail = &s[1 ..];
    }
    if r & 2 != 0 {
        vmarque_string(&mut current, tail, wait);
    }
    else {
        marque_string(&mut current, tail, wait);
    }
}

#[inline]
pub const fn shr_parts(val: u64, n: usize) -> u64 {
    let (lo, hi) = (val as u32, (val >> 32) as u32);
    let (lo, hi) = (lo >> n, hi >> n);
    lo as u64 | (hi as u64) << 32
}

#[inline]
pub const fn shl_parts(val: u64, n: usize) -> u64 {
    let (lo, hi) = (val as u32, (val >> 32) as u32);
    let (lo, hi) = (lo << n, hi << n);
    lo as u64 | (hi as u64) << 32
}
