//! Board representation and low-level piece/color occupancy operations.

use super::{
    bitboard::{Bitboard, BitboardUtils},
    color::Color,
    piece::Piece,
    rank::Rank,
    square::Square,
};
use crate::{
    logic::attacks::*,
    utilities::{Array, IterableEnum},
};

/// A struct representing the positions of all pieces on the board, for both colors.
///
/// `piece_masks` are authoritative for attacks; `pieces` is a mailbox (`pieces[square]`)
/// mirroring piece placement for O(1) [`Self::piece_at`].
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Board {
    piece_masks: [Bitboard; Piece::LIMIT as usize],
    color_masks: [Bitboard; 2],
    /// Piece type per square (`Square` index 0..64); empty squares are [`Piece::Null`].
    pieces: [Piece; 64],
}

impl Board {
    const fn mailbox_from_piece_masks(
        piece_masks: &[Bitboard; Piece::LIMIT as usize],
    ) -> [Piece; 64] {
        let mut out = [Piece::Null; 64];
        for square in Square::ALL {
            let mask = square.mask();
            for piece in Piece::PIECES {
                if piece_masks[piece as usize] & mask != 0 {
                    out[square as usize] = piece;
                    break;
                }
            }
        }
        out
    }

    /// The board for the initial position.
    pub const fn initial() -> Board {
        const WP: Bitboard = Rank::Two.mask();
        const BP: Bitboard = Rank::Seven.mask();
        const WN: Bitboard = Square::B1.mask() | Square::G1.mask();
        const BN: Bitboard = Square::B8.mask() | Square::G8.mask();
        const WB: Bitboard = Square::C1.mask() | Square::F1.mask();
        const BB: Bitboard = Square::C8.mask() | Square::F8.mask();
        const WR: Bitboard = Square::A1.mask() | Square::H1.mask();
        const BR: Bitboard = Square::A8.mask() | Square::H8.mask();
        const WQ: Bitboard = Square::D1.mask();
        const BQ: Bitboard = Square::D8.mask();
        const WK: Bitboard = Square::E1.mask();
        const BK: Bitboard = Square::E8.mask();
        const STARTING_WHITE: Bitboard = WP | WN | WB | WR | WQ | WK;
        const STARTING_BLACK: Bitboard = BP | BN | BB | BR | BQ | BK;
        const STARTING_ALL: Bitboard = STARTING_WHITE | STARTING_BLACK;
        const PM: Array<Bitboard, { Piece::LIMIT as usize }> = Array([
            STARTING_ALL,
            WP | BP,
            WN | BN,
            WB | BB,
            WR | BR,
            WQ | BQ,
            WK | BK,
        ]);
        Board {
            piece_masks: PM.0,
            color_masks: [STARTING_WHITE, STARTING_BLACK],
            pieces: Self::mailbox_from_piece_masks(&PM.0),
        }
    }

    /// The board for a blank position with no pieces on it.
    pub const fn blank() -> Board {
        Board {
            piece_masks: [0; Piece::LIMIT as usize],
            color_masks: [0; 2],
            pieces: [Piece::Null; 64],
        }
    }

    #[inline]
    /// Returns the occupancy mask for piece type `P`.
    pub const fn piece_mask<const P: Piece>(&self) -> Bitboard {
        self.piece_masks[P as usize]
    }

    /// When the piece type is only known at runtime (e.g. loop variable), use this instead of [`Self::piece_mask`].
    #[inline]
    pub const fn piece_mask_at(&self, piece: Piece) -> Bitboard {
        self.piece_masks[piece as usize]
    }

    #[inline]
    /// Returns the occupancy mask for color `C`.
    pub const fn color_mask<const C: Color>(&self) -> Bitboard {
        self.color_masks[C as usize]
    }

    /// When the color is only known at runtime, use this instead of [`Self::color_mask`].
    #[inline]
    pub const fn color_mask_at(&self, color: Color) -> Bitboard {
        self.color_masks[color as usize]
    }

    #[inline]
    /// Returns occupancy of all pieces (both colors).
    pub const fn pieces(&self) -> Bitboard {
        self.piece_mask::<{ Piece::ALL_PIECES }>()
    }

    #[inline]
    /// Returns occupancy of bishops and queens.
    pub const fn diagonal_sliders(&self) -> Bitboard {
        self.piece_mask::<{ Piece::Bishop }>() | self.piece_mask::<{ Piece::Queen }>()
    }

    #[inline]
    /// Returns occupancy of rooks and queens.
    pub const fn orthogonal_sliders(&self) -> Bitboard {
        self.piece_mask::<{ Piece::Rook }>() | self.piece_mask::<{ Piece::Queen }>()
    }

    /// True if any sliding attacker in `attackers` sees `square` along a ray with `occupied` blockers.
    fn is_square_attacked_by_sliding(
        &self,
        square: Square,
        occupied: Bitboard,
        attackers: Bitboard,
    ) -> bool {
        let relevant_sliding_attackers = ((self.diagonal_sliders() & square.diagonals_mask())
            | (self.orthogonal_sliders() & square.orthogonals_mask()))
            & attackers;

        for attacker_square in relevant_sliding_attackers.iter_set_bits_as_squares() {
            if Bitboard::between(square, attacker_square) & occupied == 0 {
                return true;
            }
        }
        false
    }

    #[inline]
    /// Returns non-sliding attackers (pawn/knight/king) on any square in `mask`.
    pub fn non_sliding_attacks_on_mask(&self, mask: Bitboard, by: Color) -> Bitboard {
        (multi_pawn_attacks(mask, by.other()) & self.piece_mask::<{ Piece::Pawn }>())
            | (multi_knight_attacks(mask) & self.piece_mask::<{ Piece::Knight }>())
            | (multi_king_attacks(mask) & self.piece_mask::<{ Piece::King }>())
    }

    #[inline]
    /// Returns non-sliding attackers (pawn/knight/king) on `square`.
    pub fn non_sliding_attacks_on_square(&self, square: Square, by: Color) -> Bitboard {
        (multi_pawn_attacks(square.mask(), by.other()) & self.piece_mask::<{ Piece::Pawn }>())
            | (single_knight_attacks(square) & self.piece_mask::<{ Piece::Knight }>())
            | (single_king_attacks(square) & self.piece_mask::<{ Piece::King }>())
    }

    /// Returns whether any square in `mask` is attacked by `by_color`.
    pub fn is_mask_attacked(&self, mask: Bitboard, by_color: Color) -> bool {
        let attackers = self.color_mask_at(by_color);

        if attackers & self.non_sliding_attacks_on_mask(mask, by_color) != 0 {
            true
        } else {
            for defending_square in mask.iter_set_bits_as_squares() {
                if self.is_square_attacked_by_sliding(defending_square, self.pieces(), attackers) {
                    return true;
                }
            }
            false
        }
    }

    #[inline]
    /// Returns whether `square` is attacked by `by_color`.
    pub fn is_square_attacked(&self, square: Square, by_color: Color) -> bool {
        self.is_square_attacked_after_move(square, by_color, 0)
    }

    #[inline]
    /// Returns whether `square` is attacked by `by_color` after applying `move_mask` occupancy delta.
    pub fn is_square_attacked_after_move(
        &self,
        square: Square,
        by_color: Color,
        move_mask: Bitboard,
    ) -> bool {
        let attackers = self.color_mask_at(by_color) & !move_mask;

        attackers & self.non_sliding_attacks_on_square(square, by_color) != 0
            || self.is_square_attacked_by_sliding(square, self.pieces() ^ move_mask, attackers)
    }

    /// Populates a square with `color`, but no piece type.
    #[inline]
    pub const fn put_color_at(&mut self, color: Color, square: Square) {
        let mask = square.mask();
        self.color_masks[color as usize] |= mask;
    }

    /// Populates a square with `piece`, but no color.
    #[inline]
    pub const fn put_piece_at(&mut self, piece: Piece, square: Square) {
        let mask = square.mask();
        self.piece_masks[piece as usize] |= mask;
        self.piece_masks[Piece::ALL_PIECES as usize] |= mask;
        self.pieces[square as usize] = piece;
    }

    /// Populates a square with both `color` and `piece`.
    #[inline]
    pub const fn put_piece_and_color(&mut self, color: Color, piece: Piece, square: Square) {
        self.put_color_at(color, square);
        self.put_piece_at(piece, square);
    }

    /// Removes `color` from a square, but not piece type.
    #[inline]
    pub const fn remove_color_at(&mut self, color: Color, square: Square) {
        let mask = square.mask();
        self.color_masks[color as usize] &= !mask;
    }

    /// Removes `piece` from a square, but not color.
    #[inline]
    pub const fn remove_piece_at(&mut self, piece: Piece, square: Square) {
        let mask = square.mask();
        self.piece_masks[piece as usize] &= !mask;
        self.piece_masks[Piece::ALL_PIECES as usize] &= !mask;
        self.pieces[square as usize] = Piece::Null;
    }

    /// Removes both `color` and `piece` from a square.
    #[inline]
    pub const fn remove_piece_and_color(&mut self, color: Color, piece: Piece, square: Square) {
        self.remove_color_at(color, square);
        self.remove_piece_at(piece, square);
    }

    /// Moves `piece` from `from` to `to`.
    /// Does not update color.
    #[inline]
    pub const fn move_piece(&mut self, piece: Piece, from: Square, to: Square) {
        let from_mask = from.mask();
        let to_mask = to.mask();
        let from_to_mask = from_mask | to_mask;

        self.piece_masks[piece as usize] ^= from_to_mask;
        self.piece_masks[Piece::ALL_PIECES as usize] ^= from_to_mask;

        self.pieces[from as usize] = Piece::Null;
        self.pieces[to as usize] = piece;
    }

    /// Moves `color` from `from` to `to`.
    /// Does not update color.
    #[inline]
    pub const fn move_color(&mut self, color: Color, from: Square, to: Square) {
        let from_mask = from.mask();
        let to_mask = to.mask();
        let from_to_mask = from_mask | to_mask;

        self.color_masks[color as usize] ^= from_to_mask;
    }

    /// Moves both `color` and `piece` from `from` to `to`.
    #[inline]
    pub const fn move_piece_and_color(
        &mut self,
        color: Color,
        piece: Piece,
        from: Square,
        to: Square,
    ) {
        self.move_color(color, from, to);
        self.move_piece(piece, from, to);
    }

    /// Returns the piece type at `square` (from the mailbox; kept in sync with `piece_masks`).
    #[inline]
    pub const fn piece_at(&self, square: Square) -> Piece {
        self.pieces[square as usize]
    }

    #[inline]
    /// Returns whether `square` currently contains any non-null piece.
    pub const fn is_occupied_at(&self, square: Square) -> bool {
        (self.pieces[square as usize] as u8) != (Piece::Null as u8)
    }

    /// Returns the color at `square`.
    #[inline]
    pub const fn color_at(&self, square: Square) -> Color {
        let mask = square.mask();
        Color::from_is_black(self.color_masks[Color::Black as usize] & mask != 0)
    }

    /// Checks if the board is consistent (color masks, individual piece type masks, all occupancy).
    pub const fn is_consistent(&self) -> bool {
        let white_mask = self.color_masks[Color::White as usize];
        let black_mask = self.color_masks[Color::Black as usize];
        if white_mask & black_mask != 0 {
            return false;
        }

        let all_occupancy_mask = self.piece_masks[Piece::ALL_PIECES as usize];

        if (white_mask | black_mask) != all_occupancy_mask {
            return false;
        }

        let mut all_occupancy_mask_reconstructed: Bitboard = 0;

        for piece in Piece::PIECES {
            let piece_mask = self.piece_masks[piece as usize];

            if piece_mask & all_occupancy_mask != piece_mask {
                return false;
            }

            if (piece_mask & white_mask) | (piece_mask & black_mask) != piece_mask {
                return false;
            }

            if piece_mask & all_occupancy_mask_reconstructed != 0 {
                return false;
            }
            all_occupancy_mask_reconstructed |= piece_mask;
        }

        if all_occupancy_mask_reconstructed != all_occupancy_mask {
            return false;
        }

        for square in Square::ALL {
            let mask = square.mask();
            let mut from_masks = Piece::Null;
            for piece in Piece::PIECES {
                if self.piece_masks[piece as usize] & mask != 0 {
                    from_masks = piece;
                    break;
                }
            }
            if (from_masks as u8) != (self.pieces[square as usize] as u8) {
                return false;
            }
        }

        true
    }

    /// Checks if the board has one king of each color.
    pub const fn has_valid_kings(&self) -> bool {
        let white_mask = self.color_masks[Color::White as usize];
        let kings_mask = self.piece_masks[Piece::King as usize];

        kings_mask.count_ones() == 2 && (white_mask & kings_mask).count_ones() == 1
    }

    /// Rigorous check for the validity and consistency of the board.
    pub const fn is_unequivocally_valid(&self) -> bool {
        self.has_valid_kings() && self.is_consistent()
    }
}

#[cfg(test)]
mod const_eval_smoke_tests {
    use super::Board;
    use crate::types::{Bitboard, Piece, Square};

    /// Compile-time use of `const fn` board API (fails to compile if a link breaks).
    const INITIAL: Board = Board::initial();
    const E1_HAS_PIECE: bool = INITIAL.is_occupied_at(Square::E1);
    const PAWN_MASK: Bitboard = INITIAL.piece_mask::<{ Piece::Pawn }>();
    const _: () = assert!(E1_HAS_PIECE);

    #[test]
    fn initial_board_const_matches_runtime() {
        assert_eq!(INITIAL, Board::initial());
        assert_eq!(INITIAL.piece_at(Square::E1), Piece::King);
        assert_eq!(PAWN_MASK, INITIAL.piece_mask::<{ Piece::Pawn }>());
    }
}
