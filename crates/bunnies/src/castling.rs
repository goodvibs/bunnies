use crate::{Color, Flank, Piece, Position};

impl<const N: usize, const STM: Color> Position<N, STM> {
    /// Returns whether the current side to move has castling rights on `flank`.
    pub fn has_castling_rights(&self, flank: Flank) -> bool {
        self.context().castling_rights.has(flank, STM)
    }

    /// Returns true if there are no pieces between king and rook on `flank`.
    const fn has_castling_space(&self, flank: Flank) -> bool {
        flank.castling_gap_mask(STM) & self.board.piece_mask::<{ Piece::ALL_PIECES }>() == 0
    }

    /// Opponent cannot attack squares the king crosses or lands on.
    fn can_castle_without_check(&self, flank: Flank) -> bool {
        !self
            .board
            .is_mask_attacked(flank.king_path_mask(STM), STM.other())
    }

    /// Legal castling on `flank` for the side to move.
    pub fn can_legally_castle(&self, flank: Flank) -> bool {
        self.has_castling_rights(flank)
            && self.has_castling_space(flank)
            && self.can_castle_without_check(flank)
    }
}
