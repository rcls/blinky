use crate::{chars, pendsv, pulse};

#[derive_const(Default)]
pub struct Display {
    current: u64
}

impl Display {
    pub const fn _new(current: u64) -> Display {Display{current}}

    pub async fn marque_display(&mut self, mut new: u64, wait: u32) {
        for _ in 0 .. 6 {
            pendsv::sleep(wait).await;
            self.current |= (new & 255) << 36;
            self.current >>= 8;
            new >>= 8;
            pulse::set_display(self.current);
        }
    }

    pub async fn marque_string(&mut self, s: &[u8], wait: u32) {
        for &c in s {
            self.marque_display(chars::COLUMNS[c as usize], wait).await;
        }
    }
}
