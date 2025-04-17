//! Board struct and methods

use crate::{BitboardUtils, ColoredPieceType};
use crate::PieceType;
use crate::Square;
use crate::attacks::*;
use crate::masks::*;
use crate::utilities::{Charboard, CharboardDisplay};
use crate::{Bitboard, Color};
use std::fmt::Display;

/// A struct representing the positions of all pieces on the board, for both colors,
/// and the zobrist hash of the position.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Board {
    pub piece_type_masks: [Bitboard; PieceType::LIMIT as usize],
    pub color_masks: [Bitboard; 2],
    pub zobrist_hash: Bitboard,
}

impl Board {
    /// The board for the initial position.
    pub fn initial() -> Board {
        let mut res = Board {
            piece_type_masks: [
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
            piece_type_masks: [0; PieceType::LIMIT as usize],
            color_masks: [0; 2],
            zobrist_hash: 0,
        }
    }
    
    pub const fn piece_mask(&self, piece_type: PieceType) -> Bitboard {
        self.piece_type_masks[piece_type as usize]
    }
    
    pub const fn color_mask(&self, color: Color) -> Bitboard {
        self.color_masks[color as usize]
    }
    
    pub const fn pieces(&self) -> Bitboard {
        self.piece_mask(PieceType::ALL_PIECE_TYPES)
    }
    
    pub const fn pawns(&self) -> Bitboard {
        self.piece_mask(PieceType::Pawn)
    }

    pub const fn knights(&self) -> Bitboard {
        self.piece_mask(PieceType::Knight)
    }

    pub const fn bishops(&self) -> Bitboard {
        self.piece_mask(PieceType::Bishop)
    }

    pub const fn rooks(&self) -> Bitboard {
        self.piece_mask(PieceType::Rook)
    }

    pub const fn queens(&self) -> Bitboard {
        self.piece_mask(PieceType::Queen)
    }

    pub const fn kings(&self) -> Bitboard {
        self.piece_mask(PieceType::King)
    }

    pub fn calc_attacks_mask(&self, by_color: Color) -> Bitboard {
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

        for src_square in ((self.rooks() | queens_mask) & attacking_color_mask).iter_set_bits_as_squares()
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
    pub fn put_piece_type_at(&mut self, piece_type: PieceType, square: Square) {
        let mask = square.mask();
        self.piece_type_masks[piece_type as usize] |= mask;
        self.piece_type_masks[PieceType::ALL_PIECE_TYPES as usize] |= mask;
        self.xor_piece_zobrist_hash(square, piece_type);
    }

    /// Populates a square with `colored_piece`.
    /// Updates the zobrist hash.
    pub fn put_colored_piece_at(&mut self, colored_piece: ColoredPieceType, square: Square) {
        let piece_type = colored_piece.piece_type();
        let color = colored_piece.color();

        self.put_color_at(color, square);
        self.put_piece_type_at(piece_type, square);
    }

    /// Removes `color` from a square, but not piece type.
    /// Does not update the zobrist hash.
    pub fn remove_color_at(&mut self, color: Color, square: Square) {
        let mask = square.mask();
        self.color_masks[color as usize] &= !mask;
    }

    /// Removes `piece_type` from a square, but not color.
    /// Updates the zobrist hash.
    pub fn remove_piece_type_at(&mut self, piece_type: PieceType, square: Square) {
        let mask = square.mask();
        self.piece_type_masks[piece_type as usize] &= !mask;
        self.piece_type_masks[PieceType::ALL_PIECE_TYPES as usize] &= !mask;
        self.xor_piece_zobrist_hash(square, piece_type);
    }

    /// Removes `colored_piece` from a square.
    /// Updates the zobrist hash.
    pub fn remove_colored_piece_at(&mut self, colored_piece: ColoredPieceType, square: Square) {
        let piece_type = colored_piece.piece_type();
        let color = colored_piece.color();

        self.remove_color_at(color, square);
        self.remove_piece_type_at(piece_type, square);
    }

    /// Moves `piece_type` from `src_square` to `dst_square`.
    /// Does not update color.
    /// Updates the zobrist hash.
    pub fn move_piece_type(
        &mut self,
        piece_type: PieceType,
        dst_square: Square,
        src_square: Square,
    ) {
        let dst_mask = dst_square.mask();
        let src_mask = src_square.mask();
        let src_dst_mask = src_mask | dst_mask;

        self.piece_type_masks[piece_type as usize] ^= src_dst_mask;
        self.piece_type_masks[PieceType::ALL_PIECE_TYPES as usize] ^= src_dst_mask;

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
        colored_piece: ColoredPieceType,
        dst_square: Square,
        src_square: Square,
    ) {
        let piece_type = colored_piece.piece_type();
        let color = colored_piece.color();

        self.move_color(color, dst_square, src_square);
        self.move_piece_type(piece_type, dst_square, src_square);
    }

    /// Returns the piece type at `square`.
    pub fn get_piece_type_at(&self, square: Square) -> PieceType {
        let mask = square.mask();
        for piece_type in PieceType::PIECES {
            if self.piece_type_masks[piece_type as usize] & mask != 0 {
                return piece_type;
            }
        }
        PieceType::NoPieceType
    }

    pub fn is_occupied_at(&self, square: Square) -> bool {
        let mask = square.mask();
        self.piece_type_masks[PieceType::ALL_PIECE_TYPES as usize] & mask != 0
    }

    /// Returns the color at `square`.
    pub fn get_color_at(&self, square: Square) -> Color {
        let mask = square.mask();
        Color::from_is_black(self.color_masks[Color::Black as usize] & mask != 0)
    }

    /// Returns the colored piece at `square`.
    pub fn get_colored_piece_at(&self, square: Square) -> ColoredPieceType {
        let piece_type = self.get_piece_type_at(square);
        let color = self.get_color_at(square);
        ColoredPieceType::new(color, piece_type)
    }

    /// Checks if the board is consistent (color masks, individual piece type masks, all occupancy).
    pub fn is_consistent(&self) -> bool {
        let white_bb = self.color_masks[Color::White as usize];
        let black_bb = self.color_masks[Color::Black as usize];
        if white_bb & black_bb != 0 {
            return false;
        }

        let all_occupancy_bb = self.piece_type_masks[PieceType::ALL_PIECE_TYPES as usize];

        if (white_bb | black_bb) != all_occupancy_bb {
            return false;
        }

        let mut all_occupancy_bb_reconstructed: Bitboard = 0;

        for piece_type in PieceType::PIECES {
            let piece_bb = self.piece_type_masks[piece_type as usize];

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
        let kings_bb = self.piece_type_masks[PieceType::King as usize];

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

    pub fn to_cb(&self) -> Charboard {
        let mut cb: Charboard = [[' '; 8]; 8];
        for (i, square) in Square::ALL.into_iter().enumerate() {
            let colored_piece = self.get_colored_piece_at(square);
            cb[i / 8][i % 8] = colored_piece.ascii();
        }
        cb
    }

    pub fn to_cb_pretty(&self) -> Charboard {
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
        write!(f, "{}", &self.to_cb_pretty().to_string())
    }
}
