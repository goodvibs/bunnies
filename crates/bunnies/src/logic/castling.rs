//! Castling legality checks for kingside and queenside.

use crate::types::{Color, Flank, Piece, Position, ZobristPolicy};

impl<const N: usize, const STM: Color, Z: ZobristPolicy> Position<N, STM, Z> {
    /// Returns `true` if the side to move retains castling rights on `flank`.
    pub fn has_castling_rights(&self, flank: Flank) -> bool {
        self.context().castling_rights.has(flank, STM)
    }

    /// Returns `true` if no pieces block the king-to-rook path on `flank`.
    const fn has_castling_space(&self, flank: Flank) -> bool {
        flank.castling_gap_mask(STM) & self.board.piece_mask::<{ Piece::ALL_PIECES }>() == 0
    }

    /// Returns `true` if opponent doesn't attack any square the king crosses or lands on.
    fn can_castle_without_check(&self, flank: Flank) -> bool {
        !self
            .board
            .is_mask_attacked(flank.king_path_mask(STM), STM.other())
    }

    /// Full legality check for castling on `flank`.
    ///
    /// Requires: rights not forfeited, no blocking pieces, and king path not under attack.
    pub fn can_legally_castle(&self, flank: Flank) -> bool {
        self.has_castling_rights(flank)
            && self.has_castling_space(flank)
            && self.can_castle_without_check(flank)
    }
}
