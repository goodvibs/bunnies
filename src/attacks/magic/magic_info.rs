use crate::bitboard::Bitboard;

/// Struct to store all magic-related information for a square
#[derive(Copy, Clone)]
pub struct MagicInfo {
    pub relevant_mask: Bitboard,
    pub magic_number: Bitboard,
    pub right_shift_amount: u8,
    pub offset: u32
}

impl MagicInfo {
    pub fn calc_key(&self, occupied_mask: Bitboard) -> usize {
        self.calc_key_without_offset(occupied_mask) + self.offset as usize
    }

    pub fn calc_key_without_offset(&self, occupied_mask: Bitboard) -> usize {
        let blockers = occupied_mask & self.relevant_mask;
        let mut hash = blockers.wrapping_mul(self.magic_number);
        hash >>= self.right_shift_amount;
        hash as usize
    }
}

impl Default for MagicInfo {
    fn default() -> Self {
        MagicInfo {
            relevant_mask: 0,
            magic_number: 0,
            right_shift_amount: 0,
            offset: 0
        }
    }
}