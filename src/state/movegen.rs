//! Move generation functions for the state struct

use crate::attacks::{multi_pawn_attacks, multi_pawn_moves, single_bishop_attacks, single_king_attacks, single_knight_attacks, single_rook_attacks};
use crate::utils::{get_set_bit_mask_iter, get_squares_from_mask_iter, Bitboard, SetBitMaskIterator};
use crate::utils::Color;
use crate::utils::masks::{FILE_A, RANK_1, RANK_3, RANK_4, RANK_5, RANK_6, RANK_8};
use crate::utils::PieceType;
use crate::r#move::{Move, MoveFlag};
use crate::utils::Square;
use crate::state::{State};

fn generate_pawn_promotions(src_square: Square, dst_square: Square) -> [Move; 4] {
    PieceType::PROMOTION_PIECES.map(|promotion_piece| {
        Move::new_promotion(dst_square, src_square, promotion_piece)
    })
}

impl State {
    fn add_normal_pawn_captures_pseudolegal(&self, moves: &mut Vec<Move>, pawn_srcs: SetBitMaskIterator, attacks_mask: &mut Bitboard) {
        let opposite_color = self.side_to_move.flip();
        let opposite_color_bb = self.board.color_masks[opposite_color as usize];

        let promotion_rank = match self.side_to_move {
            Color::White => RANK_8,
            Color::Black => RANK_1
        };

        for src in pawn_srcs {
            let move_src = unsafe { Square::from_bitboard(src) };

            let pawn_attacks = multi_pawn_attacks(src, self.side_to_move);
            *attacks_mask |= pawn_attacks;

            let pawn_captures = pawn_attacks & opposite_color_bb;

            for dst in get_set_bit_mask_iter(pawn_captures) {
                let move_dst = unsafe { Square::from_bitboard(dst) };
                if dst & promotion_rank != 0 {
                    moves.extend_from_slice(&generate_pawn_promotions(move_src, move_dst));
                }
                else {
                    moves.push(Move::new_non_promotion(move_dst, move_src, MoveFlag::NormalMove));
                }
            }
        }
    }

    fn add_en_passant_pseudolegal(&self, moves: &mut Vec<Move>) {
        let context = self.context.borrow();
        let same_color_bb = self.board.color_masks[self.side_to_move as usize];
        let pawns_bb = self.board.piece_type_masks[PieceType::Pawn as usize] & same_color_bb;

        let (src_rank_bb, src_rank_first_square, dst_rank_first_square) = match self.side_to_move {
            Color::White => (RANK_5, Square::A5, Square::A6),
            Color::Black => (RANK_4, Square::A4, Square::A3),
        };

        if context.double_pawn_push != -1 { // if en passant is possible
            for direction in [-1, 1] { // left and right
                let double_pawn_push_file = context.double_pawn_push as i32 + direction;

                if double_pawn_push_file >= 0 && double_pawn_push_file <= 7 { // if within bounds
                    let double_pawn_push_file_mask = FILE_A >> double_pawn_push_file;

                    if pawns_bb & double_pawn_push_file_mask & src_rank_bb != 0 {
                        let move_src = unsafe { Square::from(src_rank_first_square as u8 + double_pawn_push_file as u8) };
                        let move_dst = unsafe { Square::from(dst_rank_first_square as u8 + context.double_pawn_push as u8) };

                        moves.push(Move::new_non_promotion(move_dst, move_src, MoveFlag::EnPassant));
                    }
                }
            }
        }
    }
    
    fn add_pawn_push_pseudolegal(&self, moves: &mut Vec<Move>, pawn_srcs: SetBitMaskIterator) {
        let all_occupancy_bb = self.board.piece_type_masks[PieceType::AllPieceTypes as usize];

        let promotion_rank = RANK_8 >> (self.side_to_move as u8 * 7 * 8); // RANK_8 for white, RANK_1 for black

        // pawn pushes
        let single_push_rank = match self.side_to_move {
            Color::White => RANK_3,
            Color::Black => RANK_6
        };
        for src_bb in pawn_srcs {
            let src_square = unsafe { Square::from_bitboard(src_bb) };

            // single moves
            let single_move_dst = multi_pawn_moves(src_bb, self.side_to_move) & !all_occupancy_bb;
            if single_move_dst == 0 { // if no single moves
                continue;
            }

            let single_move_dst_square = unsafe { Square::from_bitboard(single_move_dst) };

            // double push
            if single_move_dst & single_push_rank != 0 {
                let double_move_dst = multi_pawn_moves(single_move_dst, self.side_to_move) & !all_occupancy_bb;
                if double_move_dst != 0 {
                    unsafe {
                        let double_move_dst_square = Square::from_bitboard(double_move_dst);
                        moves.push(Move::new_non_promotion(double_move_dst_square, src_square, MoveFlag::NormalMove));
                    }
                }
            }
            else if single_move_dst & promotion_rank != 0 { // promotion
                moves.extend_from_slice(&generate_pawn_promotions(src_square, single_move_dst_square));
                continue;
            }

            // single push (non-promotion)
            moves.push(Move::new_non_promotion(single_move_dst_square, src_square, MoveFlag::NormalMove));
        }
    }
    
    fn add_all_pawn_pseudolegal(&self, moves: &mut Vec<Move>, attacks_mask: &mut Bitboard) {
        let same_color_bb = self.board.color_masks[self.side_to_move as usize];
        let pawns_bb = self.board.piece_type_masks[PieceType::Pawn as usize] & same_color_bb;
        let pawn_srcs = get_set_bit_mask_iter(pawns_bb);

        self.add_normal_pawn_captures_pseudolegal(moves, pawn_srcs.clone(), attacks_mask);
        self.add_en_passant_pseudolegal(moves);
        self.add_pawn_push_pseudolegal(moves, pawn_srcs);
    }

    fn add_knight_pseudolegal(&self, moves: &mut Vec<Move>, attacks_mask: &mut Bitboard) {
        let same_color_bb = self.board.color_masks[self.side_to_move as usize];
        let knights_bb = self.board.piece_type_masks[PieceType::Knight as usize] & same_color_bb;

        for src_square in get_squares_from_mask_iter(knights_bb) {
            let knight_attacks = single_knight_attacks(src_square);
            *attacks_mask |= knight_attacks;

            let knight_moves = knight_attacks & !same_color_bb;

            for dst_square in get_squares_from_mask_iter(knight_moves) {
                moves.push(Move::new_non_promotion(dst_square, src_square, MoveFlag::NormalMove));
            }
        }
    }

    fn add_bishop_pseudolegal(&self, moves: &mut Vec<Move>, attacks_mask: &mut Bitboard) {
        let same_color_bb = self.board.color_masks[self.side_to_move as usize];
        let all_occupancy_bb = self.board.piece_type_masks[PieceType::AllPieceTypes as usize];

        let bishops_bb = self.board.piece_type_masks[PieceType::Bishop as usize] & same_color_bb;

        for src_square in get_squares_from_mask_iter(bishops_bb) {
            let bishop_attacks = single_bishop_attacks(src_square, all_occupancy_bb);
            *attacks_mask |= bishop_attacks;

            let bishop_moves = bishop_attacks & !same_color_bb;

            for dst_square in get_squares_from_mask_iter(bishop_moves) {
                moves.push(Move::new_non_promotion(dst_square, src_square, MoveFlag::NormalMove));
            }
        }
    }

    fn add_rook_pseudolegal(&self, moves: &mut Vec<Move>, attacks_mask: &mut Bitboard) {
        let same_color_bb = self.board.color_masks[self.side_to_move as usize];
        let all_occupancy_bb = self.board.piece_type_masks[PieceType::AllPieceTypes as usize];

        let rooks_bb = self.board.piece_type_masks[PieceType::Rook as usize] & same_color_bb;

        for src_square in get_squares_from_mask_iter(rooks_bb) {
            let rook_attacks = single_rook_attacks(src_square, all_occupancy_bb);
            *attacks_mask |= rook_attacks;

            let rook_moves = rook_attacks & !same_color_bb;

            for dst_square in get_squares_from_mask_iter(rook_moves) {
                moves.push(Move::new_non_promotion(dst_square, src_square, MoveFlag::NormalMove));
            }
        }
    }

    fn add_queen_pseudolegal(&self, moves: &mut Vec<Move>, attacks_mask: &mut Bitboard) {
        let same_color_bb = self.board.color_masks[self.side_to_move as usize];
        let all_occupancy_bb = self.board.piece_type_masks[PieceType::AllPieceTypes as usize];

        let queens_bb = self.board.piece_type_masks[PieceType::Queen as usize] & same_color_bb;

        for src_square in get_squares_from_mask_iter(queens_bb) {
            let queen_attacks = single_rook_attacks(src_square, all_occupancy_bb) | single_bishop_attacks(src_square, all_occupancy_bb);
            *attacks_mask |= queen_attacks;

            let queen_moves = queen_attacks & !same_color_bb;

            for dst_square in get_squares_from_mask_iter(queen_moves) {
                moves.push(Move::new_non_promotion(dst_square, src_square, MoveFlag::NormalMove));
            }
        }
    }

    fn add_king_pseudolegal(&self, moves: &mut Vec<Move>, attacks_mask: &mut Bitboard) {
        let same_color_bb = self.board.color_masks[self.side_to_move as usize];
        self.board.piece_type_masks[PieceType::AllPieceTypes as usize];

        let king_src_bb = self.board.piece_type_masks[PieceType::King as usize] & same_color_bb;
        let king_src_square = unsafe { Square::from_bitboard(king_src_bb) };

        let king_attacks = single_king_attacks(king_src_square);
        *attacks_mask |= king_attacks;

        let king_moves = king_attacks & !same_color_bb;

        for dst_square in get_squares_from_mask_iter(king_moves) {
            moves.push(Move::new_non_promotion(dst_square, king_src_square, MoveFlag::NormalMove));
        }
    }
    
    fn add_castling_pseudolegal(&self, moves: &mut Vec<Move>) {
        let king_src_square = match self.side_to_move {
            Color::White => Square::E1,
            Color::Black => Square::E8
        };

        if self.can_legally_castle_short() {
            let king_dst_square = unsafe { Square::from(king_src_square as u8 + 2) };
            moves.push(Move::new_non_promotion(king_dst_square, king_src_square, MoveFlag::Castling));
        }
        if self.can_legally_castle_long() {
            let king_dst_square = unsafe { Square::from(king_src_square as u8 - 2) };
            moves.push(Move::new_non_promotion(king_dst_square, king_src_square, MoveFlag::Castling));
        }
    }

    /// Returns a vector of pseudolegal moves.
    pub fn calc_pseudolegal_moves(&self, attacks_mask: &mut Bitboard) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();

        self.add_all_pawn_pseudolegal(&mut moves, attacks_mask);
        self.add_knight_pseudolegal(&mut moves, attacks_mask);
        self.add_bishop_pseudolegal(&mut moves, attacks_mask);
        self.add_rook_pseudolegal(&mut moves, attacks_mask);
        self.add_queen_pseudolegal(&mut moves, attacks_mask);
        self.add_king_pseudolegal(&mut moves, attacks_mask);
        self.add_castling_pseudolegal(&mut moves);

        moves
    }

    /// Returns a vector of legal moves.
    /// For each pseudolegal move, it makes the move, checks if the state is probably valid,
    /// and if so, adds the move to the vector.
    /// The state then unmakes the move before moving on to the next move.
    pub fn calc_legal_moves(&self, attacks_mask: &mut Bitboard) -> Vec<Move> {
        assert!(self.result.is_none());
        
        let pseudolegal_moves = self.calc_pseudolegal_moves(attacks_mask);
        let mut filtered_moves = Vec::new();
        
        // let self_keepsake = self.clone();
        
        let mut state = self.clone();
        for move_ in pseudolegal_moves {
            state.make_move(move_, *attacks_mask);
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