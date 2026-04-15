use crate::masks::{
    FILES, RANK_4, STARTING_BK, STARTING_KING_SIDE_BR, STARTING_KING_SIDE_WR,
    STARTING_QUEEN_SIDE_BR, STARTING_QUEEN_SIDE_WR, STARTING_WK,
};
use crate::position::Position;
use crate::{Bitboard, Color, Flank, Piece, Square};

impl<const N: usize> Position<N> {
    /// Rigorous check for whether the current positional information is consistent and valid.
    pub fn is_unequivocally_valid(&self) -> bool {
        self.board.is_unequivocally_valid()
            && self.has_valid_side_to_move()
            && self.has_valid_castling_rights()
            && self.has_valid_double_pawn_push()
            && self.has_valid_halfmove_clock()
            && !self.is_opposite_side_in_check()
            && self.is_zobrist_consistent()
    }

    /// Quick check for whether the state is probably valid, should be used after making pseudo-legal moves.
    pub fn is_probably_valid(&self) -> bool {
        self.board.has_valid_kings() && !self.is_opposite_side_in_check()
    }

    /// Checks if the zobrist hash in the context matches the board piece placement hash.
    pub fn is_zobrist_consistent(&self) -> bool {
        self.context().zobrist_hash == crate::calc_zobrist_hash(&self.board)
    }

    pub fn is_opposite_side_in_check(&self) -> bool {
        let opp = self.side_to_move.other();
        let opp_king = self.board.piece_mask(Piece::King) & self.board.color_mask(opp);
        self.board.is_square_attacked(
            unsafe { Square::from_bitboard(opp_king) },
            self.side_to_move,
        )
    }

    /// Checks if the halfmove clock is valid and consistent with the halfmove counter.
    pub fn has_valid_halfmove_clock(&self) -> bool {
        let context = self.context();
        context.has_valid_halfmove_clock() && context.halfmove_clock as u16 <= self.halfmove
    }

    /// Checks if the side to move is consistent with the halfmove counter.
    pub fn has_valid_side_to_move(&self) -> bool {
        self.halfmove % 2 == self.side_to_move as u16
    }

    /// Checks if the castling rights are consistent with the position of the rooks and kings.
    pub fn has_valid_castling_rights(&self) -> bool {
        let context = self.context();

        let kings_bb = self.board.piece_mask(Piece::King);
        let rooks_bb = self.board.piece_mask(Piece::Rook);

        let white_bb = self.board.color_mask(Color::White);
        let black_bb = self.board.color_mask(Color::Black);

        let is_white_king_in_place = (kings_bb & white_bb & STARTING_WK) != 0;
        let is_black_king_in_place = (kings_bb & black_bb & STARTING_BK) != 0;

        let white_castle =
            Flank::Kingside.rights_mask(Color::White) | Flank::Queenside.rights_mask(Color::White);
        let black_castle =
            Flank::Kingside.rights_mask(Color::Black) | Flank::Queenside.rights_mask(Color::Black);

        if !is_white_king_in_place && context.castling_rights & white_castle != 0 {
            return false;
        }

        if !is_black_king_in_place && context.castling_rights & black_castle != 0 {
            return false;
        }

        let is_white_king_side_rook_in_place = (rooks_bb & white_bb & STARTING_KING_SIDE_WR) != 0;
        if !is_white_king_side_rook_in_place
            && (context.castling_rights & Flank::Kingside.rights_mask(Color::White)) != 0
        {
            return false;
        }

        let is_white_queen_side_rook_in_place = (rooks_bb & white_bb & STARTING_QUEEN_SIDE_WR) != 0;
        if !is_white_queen_side_rook_in_place
            && (context.castling_rights & Flank::Queenside.rights_mask(Color::White)) != 0
        {
            return false;
        }

        let is_black_king_side_rook_in_place = (rooks_bb & black_bb & STARTING_KING_SIDE_BR) != 0;
        if !is_black_king_side_rook_in_place
            && (context.castling_rights & Flank::Kingside.rights_mask(Color::Black)) != 0
        {
            return false;
        }

        let is_black_queen_side_rook_in_place = (rooks_bb & black_bb & STARTING_QUEEN_SIDE_BR) != 0;
        if !is_black_queen_side_rook_in_place
            && (context.castling_rights & Flank::Queenside.rights_mask(Color::Black)) != 0
        {
            return false;
        }

        true
    }

    /// Checks if the double pawn push is consistent with the position of the pawns.
    pub fn has_valid_double_pawn_push(&self) -> bool {
        match self.context().double_pawn_push {
            -1 => true,
            file if !(-1..=7).contains(&file) => false,
            file => {
                if self.halfmove < 1 {
                    return false;
                }
                let color_just_moved = self.side_to_move.other();
                let pawns_bb = self.board.piece_mask(Piece::Pawn);
                let colored_pawns_bb = pawns_bb & self.board.color_mask(color_just_moved);
                let file_mask = FILES[file as usize];
                let rank_mask = RANK_4 << (color_just_moved as Bitboard * 8); // 4 for white, 5 for black
                colored_pawns_bb & file_mask & rank_mask != 0
            }
        }
    }
}
