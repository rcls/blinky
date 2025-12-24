use crate::pendsv::{self, FIFTH, hold_display};

#[derive_const(Default)]
pub struct Display {
    current: u64
}

impl Display {
    pub const fn _new(current: u64) -> Display {Display{current}}

    fn marque_display(&mut self, mut new: u64, wait: u32) {
        for _ in 0 .. 6 {
            self.current += (new & 255) << 48;
            self.current >>= 8;
            new >>= 8;
            pendsv::hold_display(self.current, wait);
        }
    }

    pub fn marque_string(&mut self, s: &[u8], wait: u32) {
        for &c in s {
            self.marque_display(crate::chars::COLUMNS[c as usize], wait);
        }
    }

    fn vmarque_display(&mut self, mut new: u64, wait: u32) {
        self.current = shr_parts(self.current, 1);
        pendsv::hold_display(self.current, wait);

        for _ in 0 .. 6 {
            self.current |= shl_parts(new & 0x0101_0101_0101, 6);
            self.current = shr_parts(self.current, 1);
            self.current &= 0x3f3f_3f3f_3f3f_3f3f;
            new = shr_parts(new, 1);
            pendsv::hold_display(self.current, wait);
        }
    }

    pub fn vmarque_string(&mut self, s: &[u8], wait: u32) {
        for &c in s {
            self.vmarque_display(crate::chars::COLUMNS[c as usize], wait);
        }
    }

    pub fn rmarque_string(&mut self, s: &[u8], wait: u32) {
        let r = crate::random::RANDOM.random_n(4);
        let mut tail = s;
        if r & 1 != 0 {
            // Blink the first character...
            let c = crate::chars::COLUMNS[s[0] as usize];
            for _ in 0 .. 3 {
                hold_display(c, FIFTH);
                hold_display(0, FIFTH);
            }
            self.current = c;
            hold_display(c, FIFTH);
            tail = &s[1 ..];
        }
        if r & 2 != 0 {
            self.vmarque_string(tail, wait);
        }
        else {
            self.marque_string(tail, wait);
        }
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
