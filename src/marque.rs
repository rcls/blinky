use crate::pendsv;

#[derive_const(Default)]
pub struct Display {
    current: u64
}

impl Display {
    pub const fn _new(current: u64) -> Display {Display{current}}

    pub fn marque_display(&mut self, mut new: u64, wait: u32) {
        for _ in 0 .. 6 {
            self.current += (new & 255) << 48;
            self.current = shr8(self.current);
            new = shr8(new);
            pendsv::hold_display(self.current, wait);
        }
    }

    pub fn marque_string(&mut self, s: &[u8], wait: u32) {
        for &c in s {
            self.marque_display(crate::chars::COLUMNS[c as usize], wait);
        }
    }
}

/// rustc generates bloated out-of-line code.  We don't want that.
#[inline]
pub const fn shr8(val: u64) -> u64 {
    let (lo, hi) = (val as u32, (val >> 32) as u32);
    let (lo, hi) = (lo >> 8 | hi << 24, hi >> 8);
    lo as u64 | (hi as u64) << 32
}

#[test]
fn test_shr8() {
    for i in 0 .. 64 {
        for j in 0 .. 64 {
            let x = 1 << i | 1 << j;
            assert_eq!(shr8(x), x >> 8);
        }
    }
}
