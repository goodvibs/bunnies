//! Move generation functions for the state struct

use crate::attacks::{multi_pawn_attacks, multi_pawn_moves, single_king_attacks, single_knight_attacks, sliding_piece_attacks};
use crate::masks::{FILE_A, RANK_1, RANK_3, RANK_4, RANK_5, RANK_6, RANK_8};
use crate::position::Position;
use crate::r#move::{Move, MoveFlag};
use crate::utilities::MaskBitsIterator;
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
        pawn_srcs: MaskBitsIterator,
    ) {
        let opposite_color = self.side_to_move.other();
        let opposite_color_bb = self.board.color_masks[opposite_color as usize];

        let promotion_rank = match self.side_to_move {
            Color::White => RANK_8,
            Color::Black => RANK_1,
        };

        for src in pawn_srcs {
            let move_src = unsafe { Square::from_bitboard(src) };

            let pawn_attacks = multi_pawn_attacks(src, self.side_to_move);

            let pawn_captures = pawn_attacks & opposite_color_bb;

            for dst in pawn_captures.iter_set_bits_as_masks() {
                let move_dst = unsafe { Square::from_bitboard(dst) };
                if dst & promotion_rank != 0 {
                    moves.extend_from_slice(&generate_pawn_promotions(move_src, move_dst));
                } else {
                    moves.push(Move::new_non_promotion(
                        move_dst,
                        move_src,
                        MoveFlag::NormalMove,
                    ));
                }
            }
        }
    }

    fn add_en_passant_pseudolegal(&self, moves: &mut Vec<Move>) {
        let context = unsafe { &*self.context };
        let same_color_bb = self.board.color_masks[self.side_to_move as usize];
        let pawns_bb = self.board.piece_type_masks[PieceType::Pawn as usize] & same_color_bb;

        let (src_rank_bb, src_rank_first_square, dst_rank_first_square) = match self.side_to_move {
            Color::White => (RANK_5, Square::A5, Square::A6),
            Color::Black => (RANK_4, Square::A4, Square::A3),
        };

        if context.double_pawn_push != -1 {
            // if en passant is possible
            for direction in [-1, 1] {
                // left and right
                let double_pawn_push_file = context.double_pawn_push as i32 + direction;

                if (0..=7).contains(&double_pawn_push_file) {
                    // if within bounds
                    let double_pawn_push_file_mask = FILE_A >> double_pawn_push_file;

                    if pawns_bb & double_pawn_push_file_mask & src_rank_bb != 0 {
                        let move_src = unsafe {
                            Square::from(src_rank_first_square as u8 + double_pawn_push_file as u8)
                        };
                        let move_dst = unsafe {
                            Square::from(
                                dst_rank_first_square as u8 + context.double_pawn_push as u8,
                            )
                        };

                        moves.push(Move::new_non_promotion(
                            move_dst,
                            move_src,
                            MoveFlag::EnPassant,
                        ));
                    }
                }
            }
        }
    }

    fn add_pawn_push_pseudolegal(&self, moves: &mut Vec<Move>, pawn_srcs: MaskBitsIterator) {
        let all_occupancy_bb = self.board.piece_type_masks[PieceType::ALL_PIECE_TYPES as usize];

        let promotion_rank = RANK_8 >> (self.side_to_move as u8 * 7 * 8); // RANK_8 for white, RANK_1 for black

        // pawn pushes
        let single_push_rank = match self.side_to_move {
            Color::White => RANK_3,
            Color::Black => RANK_6,
        };
        for src_bb in pawn_srcs {
            let src_square = unsafe { Square::from_bitboard(src_bb) };

            // single moves
            let single_move_dst = multi_pawn_moves(src_bb, self.side_to_move) & !all_occupancy_bb;
            if single_move_dst == 0 {
                // if no single moves
                continue;
            }

            let single_move_dst_square = unsafe { Square::from_bitboard(single_move_dst) };

            // double push
            if single_move_dst & single_push_rank != 0 {
                let double_move_dst =
                    multi_pawn_moves(single_move_dst, self.side_to_move) & !all_occupancy_bb;
                if double_move_dst != 0 {
                    unsafe {
                        let double_move_dst_square = Square::from_bitboard(double_move_dst);
                        moves.push(Move::new_non_promotion(
                            double_move_dst_square,
                            src_square,
                            MoveFlag::NormalMove,
                        ));
                    }
                }
            } else if single_move_dst & promotion_rank != 0 {
                // promotion
                moves.extend_from_slice(&generate_pawn_promotions(
                    src_square,
                    single_move_dst_square,
                ));
                continue;
            }

            // single push (non-promotion)
            moves.push(Move::new_non_promotion(
                single_move_dst_square,
                src_square,
                MoveFlag::NormalMove,
            ));
        }
    }

    fn add_all_pawn_pseudolegal(&self, moves: &mut Vec<Move>) {
        let pawns_bb = self.current_side_pawns();
        let pawn_srcs = pawns_bb.iter_set_bits_as_masks();

        self.add_normal_pawn_captures_pseudolegal(moves, pawn_srcs.clone());
        self.add_en_passant_pseudolegal(moves);
        self.add_pawn_push_pseudolegal(moves, pawn_srcs);
    }

    fn add_knight_pseudolegal(&self, moves: &mut Vec<Move>) {
        let knights_bb = self.current_side_knights() & !self.pinned_pieces();

        for src_square in knights_bb.iter_set_bits_as_squares() {
            let knight_attacks = single_knight_attacks(src_square);

            let knight_moves = knight_attacks & !self.current_side_pieces();

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
            let is_pinned = src_square.mask() & self.pinned_pieces() != 0;
            
            let attacks = sliding_piece_attacks(src_square, all_occupancy_bb, piece);
            let mut possible_moves = attacks & !same_color_bb;
            
            if is_pinned {
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
        let same_color_bb = self.board.color_masks[self.side_to_move as usize];

        let king_src_bb = self.board.piece_type_masks[PieceType::King as usize] & same_color_bb;
        let king_src_square = unsafe { Square::from_bitboard(king_src_bb) };

        let king_attacks = single_king_attacks(king_src_square);

        let king_moves = king_attacks & !same_color_bb;

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
