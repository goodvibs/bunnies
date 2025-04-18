//! Move generation functions for the state struct

use crate::attacks::{multi_pawn_attacks, multi_pawn_moves, single_king_attacks, single_knight_attacks, sliding_piece_attacks};
use crate::masks::{RANK_3, RANK_6};
use crate::position::Position;
use crate::r#move::{Move, MoveFlag};
use crate::Square;
use crate::{Bitboard, Color};
use crate::{BitboardUtils, PieceType};

fn generate_pawn_promotions(src_square: Square, dst_square: Square) -> [Move; 4] {
    PieceType::PROMOTION_PIECES
        .map(|promotion_piece| Move::new_promotion(dst_square, src_square, promotion_piece))
}

impl Position {
    fn add_normal_pawn_captures_pseudolegal(
        &self,
        moves: &mut Vec<Move>,
    ) {
        let pawn_srcs = self.current_side_pawns().iter_set_bits_as_masks();

        let opposite_side_pieces = self.opposite_side_pieces();

        let promotion_rank = self.current_side_promotion_rank();

        for src in pawn_srcs {
            let src_square = unsafe { Square::from_bitboard(src) };

            let mut possible_captures = multi_pawn_attacks(src, self.side_to_move) & opposite_side_pieces;

            if src_square.mask() & self.pinned_pieces() != 0 {
                let possible_move_ray = Bitboard::edge_to_edge_ray(src_square, unsafe { Square::from_bitboard(self.current_side_king()) });
                possible_captures &= possible_move_ray;
            }

            for dst_square in possible_captures.iter_set_bits_as_squares() {
                if dst_square.rank() == promotion_rank {
                    moves.extend(generate_pawn_promotions(src_square, dst_square));
                } else {
                    moves.push(Move::new_non_promotion(
                        dst_square,
                        src_square,
                        MoveFlag::NormalMove,
                    ));
                }
            }
        }
    }
    
    const fn get_possible_en_passant_src_squares(&self, double_pawn_push: i8) -> [Option<Square>; 2] {
        assert!(double_pawn_push >= 0 && double_pawn_push <= 7);
        
        let double_pawn_push_dst = match self.side_to_move {
            Color::White => unsafe { Square::from_rank_file(4, double_pawn_push as u8) },
            Color::Black => unsafe { Square::from_rank_file(3, double_pawn_push as u8) }
        };
        
        [double_pawn_push_dst.left(), double_pawn_push_dst.right()]
    }

    const fn get_en_passant_dst_square(&self, double_pawn_push: i8) -> Square {
        assert!(double_pawn_push >= 0 && double_pawn_push <= 7);

        match self.side_to_move {
            Color::White => unsafe { Square::from_rank_file(5, double_pawn_push as u8) },
            Color::Black => unsafe { Square::from_rank_file(2, double_pawn_push as u8) }
        }

    }

    fn add_en_passant_pseudolegal(&self, moves: &mut Vec<Move>) {
        let double_pawn_push = self.double_pawn_push();
        let current_side_pawns = self.current_side_pawns();

        if double_pawn_push != -1 {
            let dst_square = self.get_en_passant_dst_square(double_pawn_push);

            for src_square in self.get_possible_en_passant_src_squares(double_pawn_push).into_iter().flatten() {
                if src_square.mask() & self.pinned_pieces() != 0 {
                    let possible_move_ray = Bitboard::edge_to_edge_ray(src_square, unsafe { Square::from_bitboard(self.current_side_king()) });
                    if possible_move_ray & dst_square.mask() == 0 {
                        continue;
                    }
                }
                
                if src_square.mask() & current_side_pawns != 0 {
                    moves.push(Move::new_non_promotion(dst_square, src_square, MoveFlag::EnPassant));
                }
            }
        }
    }

    const unsafe fn get_pawn_push_origin(&self, dst_square: Square) -> Square {
        match self.side_to_move {
            Color::White => unsafe { dst_square.down().unwrap_unchecked() },
            Color::Black => unsafe { dst_square.up().unwrap_unchecked() }
        }
    }

    const unsafe fn get_pawn_double_push_origin(&self, dst_square: Square) -> Square {
        match self.side_to_move {
            Color::White => unsafe { dst_square.down().unwrap_unchecked().down().unwrap_unchecked() },
            Color::Black => unsafe { dst_square.up().unwrap_unchecked().up().unwrap_unchecked() }
        }
    }

    const fn get_additional_pawn_push_rank_mask(&self) -> Bitboard {
        match self.side_to_move {
            Color::White => RANK_3,
            Color::Black => RANK_6,
        }
    }

    fn add_pawn_push_pseudolegal(&self, moves: &mut Vec<Move>) {
        let occupied_mask = self.board.pieces();

        let mut movable_pawns = self.current_side_pawns();

        let pinned_pawns = self.pinned_pieces() & movable_pawns;
        if pinned_pawns != 0 {
            let current_side_king_file = unsafe { Square::from_bitboard(self.current_side_king()) }.file();

            for pinned_pawn_square in pinned_pawns.iter_set_bits_as_squares() {
                if pinned_pawn_square.file() != current_side_king_file {
                    movable_pawns &= !pinned_pawn_square.mask();
                }
            }
        }

        let single_push_dsts = multi_pawn_moves(movable_pawns, self.side_to_move) & !occupied_mask;
        for dst_square in single_push_dsts.iter_set_bits_as_squares() {
            let src_square = unsafe { self.get_pawn_push_origin(dst_square) };

            if dst_square.rank() == self.current_side_promotion_rank() {
                moves.extend(generate_pawn_promotions(src_square, dst_square));
            } else {
                moves.push(Move::new_non_promotion(dst_square, src_square, MoveFlag::NormalMove));
            }
        }

        let double_push_dsts = multi_pawn_moves(single_push_dsts & self.get_additional_pawn_push_rank_mask(), self.side_to_move) & !occupied_mask;
        for dst_square in double_push_dsts.iter_set_bits_as_squares() {
            let src_square = unsafe { self.get_pawn_double_push_origin(dst_square) };
            moves.push(Move::new_non_promotion(dst_square, src_square, MoveFlag::NormalMove));
        }
    }

    fn add_all_pawn_pseudolegal(&self, moves: &mut Vec<Move>) {
        self.add_normal_pawn_captures_pseudolegal(moves);
        self.add_en_passant_pseudolegal(moves);
        self.add_pawn_push_pseudolegal(moves);
    }

    fn add_knight_pseudolegal(&self, moves: &mut Vec<Move>) {
        let current_side_pieces = self.current_side_pieces();
        let movable_knights = self.board.knights() & current_side_pieces & !self.pinned_pieces();

        for src_square in movable_knights.iter_set_bits_as_squares() {
            let knight_attacks = single_knight_attacks(src_square);
            let knight_moves = knight_attacks & !current_side_pieces;

            for dst_square in knight_moves.iter_set_bits_as_squares() {
                moves.push(Move::new_non_promotion(
                    dst_square,
                    src_square,
                    MoveFlag::NormalMove,
                ));
            }
        }
    }

    fn add_sliding_piece_pseudolegal(&self, piece: PieceType, moves: &mut Vec<Move>) {
        let same_color_bb = self.current_side_pieces();
        let all_occupancy_bb = self.board.pieces();

        let piece_mask = self.board.piece_mask(piece) & same_color_bb;

        for src_square in piece_mask.iter_set_bits_as_squares() {
            let attacks = sliding_piece_attacks(src_square, all_occupancy_bb, piece);
            let mut possible_moves = attacks & !same_color_bb;

            if src_square.mask() & self.pinned_pieces() != 0 {
                let possible_move_ray = Bitboard::edge_to_edge_ray(src_square, unsafe { Square::from_bitboard(self.current_side_king()) });
                possible_moves &= possible_move_ray;
            }

            for dst_square in possible_moves.iter_set_bits_as_squares() {
                moves.push(Move::new_non_promotion(
                    dst_square,
                    src_square,
                    MoveFlag::NormalMove,
                ));
            }
        }
    }

    fn add_king_pseudolegal(&self, moves: &mut Vec<Move>) {
        let current_side_mask = self.current_side_pieces();

        let king_src_bb = self.board.kings() & current_side_mask;
        let king_src_square = unsafe { Square::from_bitboard(king_src_bb) };

        let king_attacks = single_king_attacks(king_src_square);
        let king_moves = king_attacks & !current_side_mask;

        for dst_square in king_moves.iter_set_bits_as_squares() {
            moves.push(Move::new_non_promotion(
                dst_square,
                king_src_square,
                MoveFlag::NormalMove,
            ));
        }
    }

    fn add_castling_pseudolegal(&self, moves: &mut Vec<Move>) {
        let king_src_square = match self.side_to_move {
            Color::White => Square::E1,
            Color::Black => Square::E8,
        };

        if self.can_legally_castle_short() {
            let king_dst_square = unsafe { Square::from(king_src_square as u8 + 2) };
            moves.push(Move::new_non_promotion(
                king_dst_square,
                king_src_square,
                MoveFlag::Castling,
            ));
        }
        if self.can_legally_castle_long() {
            let king_dst_square = unsafe { Square::from(king_src_square as u8 - 2) };
            moves.push(Move::new_non_promotion(
                king_dst_square,
                king_src_square,
                MoveFlag::Castling,
            ));
        }
    }

    /// Returns a vector of pseudolegal moves.
    pub fn calc_pseudolegal_moves(&self) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();

        self.add_all_pawn_pseudolegal(&mut moves);
        self.add_knight_pseudolegal(&mut moves);
        self.add_sliding_piece_pseudolegal(PieceType::Bishop, &mut moves);
        self.add_sliding_piece_pseudolegal(PieceType::Rook, &mut moves);
        self.add_sliding_piece_pseudolegal(PieceType::Queen, &mut moves);
        self.add_king_pseudolegal(&mut moves);
        self.add_castling_pseudolegal(&mut moves);

        moves
    }

    /// Returns a vector of legal moves.
    /// For each pseudolegal move, it makes the move, checks if the state is probably valid,
    /// and if so, adds the move to the vector.
    /// The state then unmakes the move before moving on to the next move.
    pub fn calc_legal_moves(&self) -> Vec<Move> {
        assert!(self.result.is_none());

        let pseudolegal_moves = self.calc_pseudolegal_moves();
        let mut filtered_moves = Vec::new();

        // let self_keepsake = self.clone();

        let mut state = self.clone();
        for move_ in pseudolegal_moves {
            state.make_move(move_);
            if state.is_probably_valid() {
                filtered_moves.push(move_);
            }
            state.unmake_move(move_);
            // assert!(state.is_valid());
            // assert!(self_keepsake.eq(&state));
        }
        filtered_moves
    }
}
