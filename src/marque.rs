use crate::pendsv;

#[derive_const(Default)]
pub struct Display {
    current: u64
}

impl Display {
    pub const fn _new(current: u64) -> Display {Display{current}}

    pub fn marque_display(&mut self, mut new: u64, wait: u32) {
        for _ in 0 .. 6 {
            pendsv::sleep(wait);
            self.current += (new & 255) << 48;
            self.current = pendsv::shr8(self.current);
            new = pendsv::shr8(new);
            pendsv::set_display(self.current);
        }
    }

    pub fn marque_string(&mut self, s: &[u8], wait: u32) {
        for &c in s {
            self.marque_display(crate::chars::COLUMNS[c as usize], wait);
        }
    }
}
