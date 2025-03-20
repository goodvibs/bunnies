use crate::utils::Bitboard;

/// Generate a 64-bit random number with all zeros in the upper 60 bits
fn gen_lower_bits_random(rng: &mut fastrand::Rng) -> Bitboard {
    rng.u64(..) & 0xFFFF
}

/// Generate a 64-bit random number with a generally uniform distribution of set bits
fn gen_uniform_random(rng: &mut fastrand::Rng) -> Bitboard {
    gen_lower_bits_random(rng) | (gen_lower_bits_random(rng) << 16) | (gen_lower_bits_random(rng) << 32) | (gen_lower_bits_random(rng) << 48)
}

/// Generate a 64-bit random number likely to be suitable as a magic number
pub fn gen_random_magic_number(rng: &mut fastrand::Rng) -> Bitboard {
    gen_uniform_random(rng) & gen_uniform_random(rng) & gen_uniform_random(rng)
}