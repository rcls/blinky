//! Poor man's random number generator - efficient on a Cortex-M0+.

use stm_common::vcell::VCell;

#[derive_const(Default)]
pub struct Random {
    state: VCell<u32>
}

pub static RANDOM: Random = Random::default();

impl Random {
    pub fn stir(&self, info: u32) -> u32 {
        let mut stir = self.state.read();
        stir = stir << 16 | stir >> 16;
        stir = stir.wrapping_add(info);
        stir = stir.wrapping_mul(2654435769);
        self.state.write(stir);
        stir
    }

    pub fn random_n(&self, n: u32) -> u32 {
        let s = self.stir(0) >> 16;
        s * n >> 16
    }
}
