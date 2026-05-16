#[derive(Clone, Copy, Debug)]
pub struct Prng {
    state: u64,
}

impl Prng {
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub const fn generate(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);

        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}
