//! Board struct and methods

use crate::Piece;
use crate::Rank;
use crate::Square;
use crate::attacks::*;
use crate::utilities::{Charboard, CharboardDisplay};
use crate::{Bitboard, Color};
use crate::{BitboardUtils, ColoredPiece, ConstBitboardGeometry};
use std::fmt::Display;

/// A struct representing the positions of all pieces on the board, for both colors.
///
/// [`Self::piece_masks`] are authoritative for attacks; [`Self::pieces`] is a mailbox (`pieces[square]`)
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
        let mut sq: u8 = 0;
        while sq < 64 {
            let square = Square::from_u8(sq);
            let mask = square.mask();
            let mut i = 0;
            while i < Piece::PIECES.len() {
                let piece_type = Piece::PIECES[i];
                if piece_masks[piece_type as usize] & mask != 0 {
                    out[sq as usize] = piece_type;
                    break;
                }
                i += 1;
            }
            sq += 1;
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
        const PM: [Bitboard; Piece::LIMIT as usize] = [
            STARTING_ALL,
            WP | BP,
            WN | BN,
            WB | BB,
            WR | BR,
            WQ | BQ,
            WK | BK,
        ];
        Board {
            piece_masks: PM,
            color_masks: [STARTING_WHITE, STARTING_BLACK],
            pieces: Self::mailbox_from_piece_masks(&PM),
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

    pub const fn piece_mask<const P: Piece>(&self) -> Bitboard {
        self.piece_masks[P as usize]
    }

    /// When the piece type is only known at runtime (e.g. loop variable), use this instead of [`Self::piece_mask`].
    pub const fn piece_mask_at(&self, piece_type: Piece) -> Bitboard {
        self.piece_masks[piece_type as usize]
    }

    pub const fn color_mask<const C: Color>(&self) -> Bitboard {
        self.color_masks[C as usize]
    }

    /// When the color is only known at runtime, use this instead of [`Self::color_mask`].
    pub const fn color_mask_at(&self, color: Color) -> Bitboard {
        self.color_masks[color as usize]
    }

    pub const fn pieces(&self) -> Bitboard {
        self.piece_mask::<{ Piece::ALL_PIECES }>()
    }

    pub(crate) const fn diagonal_sliders(&self) -> Bitboard {
        self.piece_mask::<{ Piece::Bishop }>() | self.piece_mask::<{ Piece::Queen }>()
    }

    pub(crate) const fn orthogonal_sliders(&self) -> Bitboard {
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

    pub fn is_mask_attacked(&self, mask: Bitboard, by_color: Color) -> bool {
        let attackers = self.color_mask_at(by_color);

        if (multi_pawn_attacks(mask, by_color.other())
            & self.piece_mask::<{ Piece::Pawn }>()
            & attackers
            != 0)
            || (multi_knight_attacks(mask) & self.piece_mask::<{ Piece::Knight }>() & attackers
                != 0)
            || (multi_king_attacks(mask) & self.piece_mask::<{ Piece::King }>() & attackers != 0)
        {
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

    pub fn is_square_attacked(&self, square: Square, by_color: Color) -> bool {
        let attackers = self.color_mask_at(by_color);

        if (multi_pawn_attacks(square.mask(), by_color.other())
            & self.piece_mask::<{ Piece::Pawn }>()
            & attackers
            != 0)
            || (single_knight_attacks(square) & self.piece_mask::<{ Piece::Knight }>() & attackers
                != 0)
            || (single_king_attacks(square) & self.piece_mask::<{ Piece::King }>() & attackers != 0)
        {
            true
        } else {
            self.is_square_attacked_by_sliding(square, self.pieces(), attackers)
        }
    }

    pub fn is_square_attacked_after_king_move(
        &self,
        square: Square,
        by_color: Color,
        king_move_src_dst: Bitboard,
    ) -> bool {
        let attackers = self.color_mask_at(by_color) & !king_move_src_dst;

        attackers
            & ((multi_pawn_attacks(square.mask(), by_color.other())
                & self.piece_mask::<{ Piece::Pawn }>())
                | (single_knight_attacks(square) & self.piece_mask::<{ Piece::Knight }>())
                | (single_king_attacks(square) & self.piece_mask::<{ Piece::King }>()))
            != 0
            || self.is_square_attacked_by_sliding(
                square,
                self.pieces() ^ king_move_src_dst,
                attackers,
            )
    }

    /// Whether `square` is attacked by `by_color` after a side makes en passant on
    /// `src -> dst` removing the opponent pawn at `capture_square`.
    ///
    /// `occupied` must be the all-pieces occupancy **after** the EP capture:
    /// `self.pieces() ^ src.mask() ^ capture_square.mask() ^ dst.mask()`.
    /// Slider rays use `occupied`; knights and kings use current piece boards; opponent pawns
    /// exclude `capture_square` (the captured pawn).
    pub fn is_square_attacked_after_en_passant(
        &self,
        square: Square,
        by_color: Color,
        occupied: Bitboard,
        capture_square: Square,
    ) -> bool {
        let opp_pawns = self.piece_mask::<{ Piece::Pawn }>()
            & self.color_mask_at(by_color)
            & !capture_square.mask();
        let attackers = self.color_mask_at(by_color);

        if (multi_pawn_attacks(square.mask(), by_color.other()) & opp_pawns != 0)
            || (single_knight_attacks(square) & self.piece_mask::<{ Piece::Knight }>() & attackers
                != 0)
            || (single_king_attacks(square) & self.piece_mask::<{ Piece::King }>() & attackers != 0)
        {
            true
        } else {
            self.is_square_attacked_by_sliding(square, occupied, attackers)
        }
    }

    pub fn calc_attacks(&self, by_color: Color) -> Bitboard {
        let attacking_color_mask = self.color_mask_at(by_color);
        let occupied_mask = self.pieces();

        let queens_mask = self.piece_mask::<{ Piece::Queen }>();

        let mut attacks = multi_pawn_attacks(
            self.piece_mask::<{ Piece::Pawn }>() & attacking_color_mask,
            by_color,
        );

        for src_square in (self.piece_mask::<{ Piece::Knight }>() & attacking_color_mask)
            .iter_set_bits_as_squares()
        {
            attacks |= non_pawn_piece_attacks(src_square, occupied_mask, Piece::Knight);
        }

        for src_square in ((self.piece_mask::<{ Piece::Bishop }>() | queens_mask)
            & attacking_color_mask)
            .iter_set_bits_as_squares()
        {
            attacks |= single_bishop_attacks(src_square, occupied_mask);
        }

        for src_square in ((self.piece_mask::<{ Piece::Rook }>() | queens_mask)
            & attacking_color_mask)
            .iter_set_bits_as_squares()
        {
            attacks |= single_rook_attacks(src_square, occupied_mask);
        }

        for src_square in
            (self.piece_mask::<{ Piece::King }>() & attacking_color_mask).iter_set_bits_as_squares()
        {
            attacks |= non_pawn_piece_attacks(src_square, occupied_mask, Piece::King);
        }

        attacks
    }

    /// Populates a square with `color`, but no piece type.
    pub const fn put_color_at(&mut self, color: Color, square: Square) {
        let mask = square.mask();
        self.color_masks[color as usize] |= mask;
    }

    /// Populates a square with `piece_type`, but no color.
    pub const fn put_piece_at(&mut self, piece_type: Piece, square: Square) {
        let mask = square.mask();
        self.piece_masks[piece_type as usize] |= mask;
        self.piece_masks[Piece::ALL_PIECES as usize] |= mask;
        self.pieces[square as usize] = piece_type;
    }

    /// Populates a square with both `color` and `piece`.
    pub const fn put_piece_and_color(&mut self, color: Color, piece: Piece, square: Square) {
        self.put_color_at(color, square);
        self.put_piece_at(piece, square);
    }

    /// Removes `color` from a square, but not piece type.
    pub const fn remove_color_at(&mut self, color: Color, square: Square) {
        let mask = square.mask();
        self.color_masks[color as usize] &= !mask;
    }

    /// Removes `piece_type` from a square, but not color.
    pub const fn remove_piece_at(&mut self, piece_type: Piece, square: Square) {
        let mask = square.mask();
        self.piece_masks[piece_type as usize] &= !mask;
        self.piece_masks[Piece::ALL_PIECES as usize] &= !mask;
        self.pieces[square as usize] = Piece::Null;
    }

    /// Removes both `color` and `piece` from a square.
    pub const fn remove_piece_and_color(&mut self, color: Color, piece: Piece, square: Square) {
        self.remove_color_at(color, square);
        self.remove_piece_at(piece, square);
    }

    /// Moves `piece_type` from `src_square` to `dst_square`.
    /// Does not update color.
    pub const fn move_piece(&mut self, piece_type: Piece, dst_square: Square, src_square: Square) {
        let dst_mask = dst_square.mask();
        let src_mask = src_square.mask();
        let src_dst_mask = src_mask | dst_mask;

        self.piece_masks[piece_type as usize] ^= src_dst_mask;
        self.piece_masks[Piece::ALL_PIECES as usize] ^= src_dst_mask;

        self.pieces[src_square as usize] = Piece::Null;
        self.pieces[dst_square as usize] = piece_type;
    }

    /// Moves `color` from `src_square` to `dst_square`.
    /// Does not update color.
    pub const fn move_color(&mut self, color: Color, dst_square: Square, src_square: Square) {
        let dst_mask = dst_square.mask();
        let src_mask = src_square.mask();
        let src_dst_mask = src_mask | dst_mask;

        self.color_masks[color as usize] ^= src_dst_mask;
    }

    /// Moves both `color` and `piece` from `src_square` to `dst_square`.
    pub const fn move_piece_and_color(
        &mut self,
        color: Color,
        piece: Piece,
        dst_square: Square,
        src_square: Square,
    ) {
        self.move_color(color, dst_square, src_square);
        self.move_piece(piece, dst_square, src_square);
    }

    /// Returns the piece type at `square` (from the mailbox; kept in sync with [`Self::piece_masks`]).
    #[inline]
    pub const fn piece_at(&self, square: Square) -> Piece {
        self.pieces[square as usize]
    }

    pub const fn is_occupied_at(&self, square: Square) -> bool {
        (self.pieces[square as usize] as u8) != (Piece::Null as u8)
    }

    /// Returns the color at `square`.
    pub const fn color_at(&self, square: Square) -> Color {
        let mask = square.mask();
        Color::from_is_black(self.color_masks[Color::Black as usize] & mask != 0)
    }

    /// Checks if the board is consistent (color masks, individual piece type masks, all occupancy).
    pub const fn is_consistent(&self) -> bool {
        let white_bb = self.color_masks[Color::White as usize];
        let black_bb = self.color_masks[Color::Black as usize];
        if white_bb & black_bb != 0 {
            return false;
        }

        let all_occupancy_bb = self.piece_masks[Piece::ALL_PIECES as usize];

        if (white_bb | black_bb) != all_occupancy_bb {
            return false;
        }

        let mut all_occupancy_bb_reconstructed: Bitboard = 0;

        // Same rationale as `piece_at`: `for` over arrays in `const fn` is not yet usable on this
        // toolchain (const `IntoIterator` for `[T; N]`).
        let mut i = 0;
        while i < Piece::PIECES.len() {
            let piece = Piece::PIECES[i];
            let piece_bb = self.piece_masks[piece as usize];

            if piece_bb & all_occupancy_bb != piece_bb {
                return false;
            }

            if (piece_bb & white_bb) | (piece_bb & black_bb) != piece_bb {
                return false;
            }

            if piece_bb & all_occupancy_bb_reconstructed != 0 {
                return false;
            }
            all_occupancy_bb_reconstructed |= piece_bb;
            i += 1;
        }

        if all_occupancy_bb_reconstructed != all_occupancy_bb {
            return false;
        }

        let mut sq: u8 = 0;
        while sq < 64 {
            let square = Square::from_u8(sq);
            let mask = square.mask();
            let mut from_masks = Piece::Null;
            let mut i = 0;
            while i < Piece::PIECES.len() {
                let piece_type = Piece::PIECES[i];
                if self.piece_masks[piece_type as usize] & mask != 0 {
                    from_masks = piece_type;
                    break;
                }
                i += 1;
            }
            if (from_masks as u8) != (self.pieces[sq as usize] as u8) {
                return false;
            }
            sq += 1;
        }

        true
    }

    /// Checks if the board has one king of each color.
    pub const fn has_valid_kings(&self) -> bool {
        let white_bb = self.color_masks[Color::White as usize];
        let kings_bb = self.piece_masks[Piece::King as usize];

        kings_bb.count_ones() == 2 && (white_bb & kings_bb).count_ones() == 1
    }

    /// Rigorous check for the validity and consistency of the board.
    pub const fn is_unequivocally_valid(&self) -> bool {
        self.has_valid_kings() && self.is_consistent()
    }

    /// Prints the board to the console.
    pub fn print(&self) {
        println!("{}", self);
    }

    pub fn ascii_charboard(&self) -> Charboard {
        let mut cb: Charboard = [[' '; 8]; 8];
        for (i, square) in Square::ALL.into_iter().enumerate() {
            let piece = self.piece_at(square);
            let color = self.color_at(square);
            cb[i / 8][i % 8] = ColoredPiece::new(color, piece).ascii();
        }
        cb
    }

    pub fn unicode_charboard(&self) -> Charboard {
        let mut cb: Charboard = [[' '; 8]; 8];
        for (i, square) in Square::ALL.into_iter().enumerate() {
            let piece = self.piece_at(square);
            let color = self.color_at(square);
            cb[i / 8][i % 8] = ColoredPiece::new(color, piece).unicode();
        }
        cb
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.unicode_charboard().to_string())
    }
}

#[cfg(test)]
mod const_eval_smoke_tests {
    use crate::Board;
    use crate::Piece;
    use crate::Square;

    /// Compile-time use of `const fn` board API (fails to compile if a link breaks).
    const INITIAL: Board = Board::initial();
    const E1_HAS_PIECE: bool = INITIAL.is_occupied_at(Square::E1);
    const PAWN_MASK: crate::Bitboard = INITIAL.piece_mask::<{ Piece::Pawn }>();

    #[test]
    fn initial_board_const_matches_runtime() {
        assert_eq!(INITIAL, Board::initial());
        assert!(E1_HAS_PIECE);
        assert_eq!(INITIAL.piece_at(Square::E1), Piece::King);
        assert_eq!(PAWN_MASK, INITIAL.piece_mask::<{ Piece::Pawn }>());
    }
}
