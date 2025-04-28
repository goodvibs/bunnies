//! Board struct and methods

use crate::Piece;
use crate::Square;
use crate::attacks::*;
use crate::masks::*;
use crate::utilities::{Charboard, CharboardDisplay};
use crate::{Bitboard, Color};
use crate::{BitboardUtils, ColoredPiece};
use std::fmt::Display;

/// A struct representing the positions of all pieces on the board, for both colors,
/// and the zobrist hash of the position.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Board {
    pub piece_masks: [Bitboard; Piece::LIMIT as usize],
    pub color_masks: [Bitboard; 2],
    pub zobrist_hash: Bitboard,
}

impl Board {
    /// The board for the initial position.
    pub fn initial() -> Board {
        let mut res = Board {
            piece_masks: [
                STARTING_ALL,
                STARTING_WP | STARTING_BP,
                STARTING_WN | STARTING_BN,
                STARTING_WB | STARTING_BB,
                STARTING_WR | STARTING_BR,
                STARTING_WQ | STARTING_BQ,
                STARTING_WK | STARTING_BK,
            ],
            color_masks: [STARTING_WHITE, STARTING_BLACK],
            zobrist_hash: 0,
        };
        res.zobrist_hash = res.calc_zobrist_hash();
        res
    }

    /// The board for a blank position with no pieces on it.
    pub const fn blank() -> Board {
        Board {
            piece_masks: [0; Piece::LIMIT as usize],
            color_masks: [0; 2],
            zobrist_hash: 0,
        }
    }

    pub const fn piece_mask(&self, piece_type: Piece) -> Bitboard {
        self.piece_masks[piece_type as usize]
    }

    pub const fn color_mask(&self, color: Color) -> Bitboard {
        self.color_masks[color as usize]
    }

    pub const fn pieces(&self) -> Bitboard {
        self.piece_mask(Piece::ALL_PIECES)
    }

    pub const fn pawns(&self) -> Bitboard {
        self.piece_mask(Piece::Pawn)
    }

    pub const fn knights(&self) -> Bitboard {
        self.piece_mask(Piece::Knight)
    }

    pub const fn bishops(&self) -> Bitboard {
        self.piece_mask(Piece::Bishop)
    }

    pub const fn rooks(&self) -> Bitboard {
        self.piece_mask(Piece::Rook)
    }

    pub const fn queens(&self) -> Bitboard {
        self.piece_mask(Piece::Queen)
    }

    pub const fn kings(&self) -> Bitboard {
        self.piece_mask(Piece::King)
    }

    pub const fn diagonal_sliders(&self) -> Bitboard {
        self.bishops() | self.queens()
    }

    pub const fn orthogonal_sliders(&self) -> Bitboard {
        self.rooks() | self.queens()
    }

    pub fn is_mask_attacked(&self, mask: Bitboard, by_color: Color) -> bool {
        let attackers = self.color_mask(by_color);

        if (multi_pawn_attacks(mask, by_color.other()) & self.pawns() & attackers != 0)
            || (multi_knight_attacks(mask) & self.knights() & attackers != 0)
            || (multi_king_attacks(mask) & self.kings() & attackers != 0)
        {
            true
        } else {
            let diagonal_attackers = self.diagonal_sliders() & attackers;
            let orthogonal_attackers = self.orthogonal_sliders() & attackers;

            for defending_square in mask.iter_set_bits_as_squares() {
                let relevant_diagonals = defending_square.diagonals_mask();
                let relevant_orthogonals = defending_square.orthogonals_mask();

                let relevant_diagonal_attackers = diagonal_attackers & relevant_diagonals;
                let relevant_orthogonal_attackers = orthogonal_attackers & relevant_orthogonals;
                let relevant_sliding_attackers =
                    relevant_diagonal_attackers | relevant_orthogonal_attackers;

                let occupied = self.pieces();

                for attacker_square in relevant_sliding_attackers.iter_set_bits_as_squares() {
                    let blockers = Bitboard::between(defending_square, attacker_square) & occupied;
                    if blockers == 0 {
                        return true;
                    }
                }
            }

            false
        }
    }

    pub fn is_square_attacked(&self, square: Square, by_color: Color) -> bool {
        let attackers = self.color_mask(by_color);

        if (multi_pawn_attacks(square.mask(), by_color.other()) & self.pawns() & attackers != 0)
            || (single_knight_attacks(square) & self.knights() & attackers != 0)
            || (single_king_attacks(square) & self.kings() & attackers != 0)
        {
            true
        } else {
            let diagonal_attackers = self.diagonal_sliders() & attackers;
            let orthogonal_attackers = self.orthogonal_sliders() & attackers;

            let relevant_diagonals = square.diagonals_mask();
            let relevant_orthogonals = square.orthogonals_mask();

            let relevant_diagonal_attackers = diagonal_attackers & relevant_diagonals;
            let relevant_orthogonal_attackers = orthogonal_attackers & relevant_orthogonals;
            let relevant_sliding_attackers =
                relevant_diagonal_attackers | relevant_orthogonal_attackers;

            let occupied = self.pieces();

            for attacker_square in relevant_sliding_attackers.iter_set_bits_as_squares() {
                let blockers = Bitboard::between(square, attacker_square) & occupied;
                if blockers == 0 {
                    return true;
                }
            }

            false
        }
    }

    pub fn is_square_attacked_after_king_move(
        &self,
        square: Square,
        by_color: Color,
        king_move_src_dst: Bitboard,
    ) -> bool {
        let attackers = self.color_mask(by_color) & !king_move_src_dst;

        if (multi_pawn_attacks(square.mask(), by_color.other()) & self.pawns() & attackers != 0)
            || (single_knight_attacks(square) & self.knights() & attackers != 0)
            || (single_king_attacks(square) & self.kings() & attackers != 0)
        {
            true
        } else {
            let diagonal_attackers = self.diagonal_sliders() & attackers;
            let orthogonal_attackers = self.orthogonal_sliders() & attackers;

            let relevant_diagonals = square.diagonals_mask();
            let relevant_orthogonals = square.orthogonals_mask();

            let relevant_diagonal_attackers = diagonal_attackers & relevant_diagonals;
            let relevant_orthogonal_attackers = orthogonal_attackers & relevant_orthogonals;
            let relevant_sliding_attackers =
                relevant_diagonal_attackers | relevant_orthogonal_attackers;

            let occupied = self.pieces() ^ king_move_src_dst;

            for attacker_square in relevant_sliding_attackers.iter_set_bits_as_squares() {
                let blockers = Bitboard::between(square, attacker_square) & occupied;
                if blockers == 0 {
                    return true;
                }
            }

            false
        }
    }

    pub fn calc_attacks(&self, by_color: Color) -> Bitboard {
        let attacking_color_mask = self.color_mask(by_color);
        let occupied_mask = self.pieces();

        let queens_mask = self.queens();

        let mut attacks = multi_pawn_attacks(self.pawns() & attacking_color_mask, by_color);

        attacks |= multi_knight_attacks(self.knights() & attacking_color_mask);

        for src_square in
            ((self.bishops() | queens_mask) & attacking_color_mask).iter_set_bits_as_squares()
        {
            attacks |= single_bishop_attacks(src_square, occupied_mask);
        }

        for src_square in
            ((self.rooks() | queens_mask) & attacking_color_mask).iter_set_bits_as_squares()
        {
            attacks |= single_rook_attacks(src_square, occupied_mask);
        }

        attacks |= multi_king_attacks(self.kings() & attacking_color_mask);

        attacks
    }

    /// Populates a square with `color`, but no piece type.
    /// Does not update the zobrist hash.
    pub fn put_color_at(&mut self, color: Color, square: Square) {
        let mask = square.mask();
        self.color_masks[color as usize] |= mask;
    }

    /// Populates a square with `piece_type`, but no color.
    /// Updates the zobrist hash.
    pub fn put_piece_at(&mut self, piece_type: Piece, square: Square) {
        let mask = square.mask();
        self.piece_masks[piece_type as usize] |= mask;
        self.piece_masks[Piece::ALL_PIECES as usize] |= mask;
        self.xor_piece_zobrist_hash(square, piece_type);
    }

    /// Populates a square with `colored_piece`.
    /// Updates the zobrist hash.
    pub fn put_colored_piece_at(&mut self, colored_piece: ColoredPiece, square: Square) {
        let piece_type = colored_piece.piece();
        let color = colored_piece.color();

        self.put_color_at(color, square);
        self.put_piece_at(piece_type, square);
    }

    /// Removes `color` from a square, but not piece type.
    /// Does not update the zobrist hash.
    pub fn remove_color_at(&mut self, color: Color, square: Square) {
        let mask = square.mask();
        self.color_masks[color as usize] &= !mask;
    }

    /// Removes `piece_type` from a square, but not color.
    /// Updates the zobrist hash.
    pub fn remove_piece_at(&mut self, piece_type: Piece, square: Square) {
        let mask = square.mask();
        self.piece_masks[piece_type as usize] &= !mask;
        self.piece_masks[Piece::ALL_PIECES as usize] &= !mask;
        self.xor_piece_zobrist_hash(square, piece_type);
    }

    /// Removes `colored_piece` from a square.
    /// Updates the zobrist hash.
    pub fn remove_colored_piece_at(&mut self, colored_piece: ColoredPiece, square: Square) {
        let piece_type = colored_piece.piece();
        let color = colored_piece.color();

        self.remove_color_at(color, square);
        self.remove_piece_at(piece_type, square);
    }

    /// Moves `piece_type` from `src_square` to `dst_square`.
    /// Does not update color.
    /// Updates the zobrist hash.
    pub fn move_piece(
        &mut self,
        piece_type: Piece,
        dst_square: Square,
        src_square: Square,
    ) {
        let dst_mask = dst_square.mask();
        let src_mask = src_square.mask();
        let src_dst_mask = src_mask | dst_mask;

        self.piece_masks[piece_type as usize] ^= src_dst_mask;
        self.piece_masks[Piece::ALL_PIECES as usize] ^= src_dst_mask;

        self.xor_piece_zobrist_hash(dst_square, piece_type);
        self.xor_piece_zobrist_hash(src_square, piece_type);
    }

    /// Moves `color` from `src_square` to `dst_square`.
    /// Does not update color.
    /// Does not update the zobrist hash.
    pub fn move_color(&mut self, color: Color, dst_square: Square, src_square: Square) {
        let dst_mask = dst_square.mask();
        let src_mask = src_square.mask();
        let src_dst_mask = src_mask | dst_mask;

        self.color_masks[color as usize] ^= src_dst_mask;
    }

    /// Moves a `colored_piece` from `src_square` to `dst_square`.
    /// Updates the zobrist hash.
    pub fn move_colored_piece(
        &mut self,
        colored_piece: ColoredPiece,
        dst_square: Square,
        src_square: Square,
    ) {
        let piece_type = colored_piece.piece();
        let color = colored_piece.color();

        self.move_color(color, dst_square, src_square);
        self.move_piece(piece_type, dst_square, src_square);
    }

    /// Returns the piece type at `square`.
    pub fn piece_at(&self, square: Square) -> Piece {
        let mask = square.mask();
        for piece_type in Piece::PIECES {
            if self.piece_masks[piece_type as usize] & mask != 0 {
                return piece_type;
            }
        }
        Piece::Null
    }

    pub fn is_occupied_at(&self, square: Square) -> bool {
        let mask = square.mask();
        self.piece_masks[Piece::ALL_PIECES as usize] & mask != 0
    }

    /// Returns the color at `square`.
    pub fn color_at(&self, square: Square) -> Color {
        let mask = square.mask();
        Color::from_is_black(self.color_masks[Color::Black as usize] & mask != 0)
    }

    /// Returns the colored piece at `square`.
    pub fn get_colored_piece_at(&self, square: Square) -> ColoredPiece {
        let piece_type = self.piece_at(square);
        let color = self.color_at(square);
        ColoredPiece::new(color, piece_type)
    }

    /// Checks if the board is consistent (color masks, individual piece type masks, all occupancy).
    pub fn is_consistent(&self) -> bool {
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

        for piece in Piece::PIECES {
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
        }

        all_occupancy_bb_reconstructed == all_occupancy_bb
    }

    /// Checks if the board has one king of each color.
    pub const fn has_valid_kings(&self) -> bool {
        let white_bb = self.color_masks[Color::White as usize];
        let kings_bb = self.piece_masks[Piece::King as usize];

        kings_bb.count_ones() == 2 && (white_bb & kings_bb).count_ones() == 1
    }

    /// Checks if the zobrist hash is correctly calculated.
    pub fn is_zobrist_valid(&self) -> bool {
        self.zobrist_hash == self.calc_zobrist_hash()
    }

    /// Rigorous check for the validity and consistency of the board.
    pub fn is_unequivocally_valid(&self) -> bool {
        self.has_valid_kings() && self.is_consistent() && self.is_zobrist_valid()
    }

    /// Prints the board to the console.
    pub fn print(&self) {
        println!("{}", self);
    }

    pub fn ascii_charboard(&self) -> Charboard {
        let mut cb: Charboard = [[' '; 8]; 8];
        for (i, square) in Square::ALL.into_iter().enumerate() {
            let colored_piece = self.get_colored_piece_at(square);
            cb[i / 8][i % 8] = colored_piece.ascii();
        }
        cb
    }

    pub fn unicode_charboard(&self) -> Charboard {
        let mut cb: Charboard = [[' '; 8]; 8];
        for (i, square) in Square::ALL.into_iter().enumerate() {
            let colored_piece = self.get_colored_piece_at(square);
            cb[i / 8][i % 8] = colored_piece.unicode();
        }
        cb
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.unicode_charboard().to_string())
    }
}
