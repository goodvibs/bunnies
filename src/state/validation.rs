use crate::state::State;
use crate::utils::{Bitboard, Color, PieceType};
use crate::utils::masks::{FILES, RANK_4, STARTING_BK, STARTING_KING_SIDE_BR, STARTING_KING_SIDE_WR, STARTING_QUEEN_SIDE_BR, STARTING_QUEEN_SIDE_WR, STARTING_WK};

impl State {
    /// Rigorous check for whether the current positional information is consistent and valid.
    pub fn is_unequivocally_valid(&self) -> bool {
        self.board.is_unequivocally_valid() &&
            self.has_valid_side_to_move() &&
            self.has_valid_castling_rights() &&
            self.has_valid_double_pawn_push() &&
            self.has_valid_halfmove_clock() &&
            self.is_not_in_illegal_check() &&
            self.is_zobrist_consistent()
    }

    /// Quick check for whether the state is probably valid, should be used after making pseudo-legal moves.
    pub fn is_probably_valid(&self) -> bool {
        self.board.has_valid_kings() && self.is_not_in_illegal_check()
    }

    /// Checks if the zobrist hash in the board is consistent with the zobrist hash in the context.
    pub fn is_zobrist_consistent(&self) -> bool {
        self.board.zobrist_hash == unsafe { (*self.context).zobrist_hash }
    }

    /// Returns true if the opponent king is not in check.
    /// Else, returns false.
    pub fn is_not_in_illegal_check(&self) -> bool {
        !self.is_opposite_side_in_check()
    }

    /// Checks if the halfmove clock is valid and consistent with the halfmove counter.
    pub fn has_valid_halfmove_clock(&self) -> bool {
        let context = unsafe { &*self.context };
        context.has_valid_halfmove_clock() && context.halfmove_clock as u16 <= self.halfmove
    }

    /// Checks if the side to move is consistent with the halfmove counter.
    pub fn has_valid_side_to_move(&self) -> bool {
        self.halfmove % 2 == self.side_to_move as u16
    }

    /// Checks if the castling rights are consistent with the position of the rooks and kings.
    pub fn has_valid_castling_rights(&self) -> bool {
        let context = unsafe { &*self.context };

        let kings_bb = self.board.piece_type_masks[PieceType::King as usize];
        let rooks_bb = self.board.piece_type_masks[PieceType::Rook as usize];

        let white_bb = self.board.color_masks[Color::White as usize];
        let black_bb = self.board.color_masks[Color::Black as usize];

        let is_white_king_in_place = (kings_bb & white_bb & STARTING_WK) != 0;
        let is_black_king_in_place = (kings_bb & black_bb & STARTING_BK) != 0;

        if !is_white_king_in_place && context.castling_rights & 0b00001100 != 0 {
            return false;
        }

        if !is_black_king_in_place && context.castling_rights & 0b00000011 != 0 {
            return false;
        }

        let is_white_king_side_rook_in_place = (rooks_bb & white_bb & STARTING_KING_SIDE_WR) != 0;
        if !is_white_king_side_rook_in_place && (context.castling_rights & 0b00001000) != 0 {
            return false;
        }

        let is_white_queen_side_rook_in_place = (rooks_bb & white_bb & STARTING_QUEEN_SIDE_WR) != 0;
        if !is_white_queen_side_rook_in_place && (context.castling_rights & 0b00000100) != 0 {
            return false;
        }

        let is_black_king_side_rook_in_place = (rooks_bb & black_bb & STARTING_KING_SIDE_BR) != 0;
        if !is_black_king_side_rook_in_place && (context.castling_rights & 0b00000010) != 0 {
            return false;
        }

        let is_black_queen_side_rook_in_place = (rooks_bb & black_bb & STARTING_QUEEN_SIDE_BR) != 0;
        if !is_black_queen_side_rook_in_place && (context.castling_rights & 0b00000001) != 0 {
            return false;
        }

        true
    }

    /// Checks if the double pawn push is consistent with the position of the pawns.
    pub fn has_valid_double_pawn_push(&self) -> bool {
        match unsafe { (*self.context).double_pawn_push } {
            -1 => true,
            file if !(-1..=7).contains(&file) => false,
            file => {
                if self.halfmove < 1 {
                    return false;
                }
                let color_just_moved = self.side_to_move.flip();
                let pawns_bb = self.board.piece_type_masks[PieceType::Pawn as usize];
                let colored_pawns_bb = pawns_bb & self.board.color_masks[color_just_moved as usize];
                let file_mask = FILES[file as usize];
                let rank_mask = RANK_4 << (color_just_moved as Bitboard * 8); // 4 for white, 5 for black
                colored_pawns_bb & file_mask & rank_mask != 0
            }
        }
    }
}