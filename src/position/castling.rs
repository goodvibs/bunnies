use crate::masks::STARTING_KING_ROOK_GAP;
use crate::position::Position;
use crate::{Flank, Piece};

impl<const N: usize> Position<N> {
    /// Returns whether the current side to move has castling rights on `flank`.
    pub fn has_castling_rights(&self, flank: Flank) -> bool {
        self.context().castling_rights & flank.rights_mask(self.side_to_move) != 0
    }

    /// Returns true if there are no pieces between king and rook on `flank`.
    const fn has_castling_space(&self, flank: Flank) -> bool {
        STARTING_KING_ROOK_GAP[self.side_to_move as usize][flank as usize]
            & self.board.piece_mask::<{ Piece::ALL_PIECES }>()
            == 0
    }

    /// Opponent cannot attack squares the king crosses or lands on.
    fn can_castle_without_check(&self, flank: Flank) -> bool {
        !self.board.is_mask_attacked(
            flank.king_path_mask(self.side_to_move),
            self.side_to_move.other(),
        )
    }

    /// Legal castling on `flank` for the side to move.
    pub fn can_legally_castle(&self, flank: Flank) -> bool {
        self.has_castling_rights(flank)
            && self.has_castling_space(flank)
            && self.can_castle_without_check(flank)
    }
}
