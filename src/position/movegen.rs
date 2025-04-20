//! Move generation functions for the state struct

use static_init::dynamic;
use crate::attacks::{multi_pawn_attacks, multi_pawn_moves, single_king_attacks, single_knight_attacks, sliding_piece_attacks};
use crate::masks::{RANK_3, RANK_6};
use crate::position::Position;
use crate::r#move::{Move, MoveFlag};
use crate::Square;
use crate::{Bitboard, Color};
use crate::{BitboardUtils, PieceType};
use crate::utilities::SquaresTwoToOneMapping;

#[dynamic]
static PAWN_PROMOTIONS_LOOKUP: SquaresTwoToOneMapping<[Move; 4]> = SquaresTwoToOneMapping::init(generate_pawn_promotions);

fn generate_pawn_promotions(src_square: Square, dst_square: Square) -> [Move; 4] {
    PieceType::PROMOTION_PIECES
        .map(|promotion_piece| Move::new_promotion(dst_square, src_square, promotion_piece))
}

impl Position {
    fn add_legal_non_ep_pawn_captures(&self, possible_dsts: Bitboard, moves: &mut Vec<Move>) {
        let opposite_side_pieces = self.opposite_side_pieces();

        let promotion_rank = self.current_side_promotion_rank();

        for src in self.current_side_pawns().iter_set_bits_as_masks() {
            let src_square = unsafe { Square::from_bitboard(src) };

            let mut possible_captures = multi_pawn_attacks(src, self.side_to_move) & opposite_side_pieces & possible_dsts;

            if src_square.mask() & self.context().pinned != 0 {
                let possible_move_ray = Bitboard::edge_to_edge_ray(src_square, unsafe { Square::from_bitboard(self.current_side_king()) });
                possible_captures &= possible_move_ray;
            }

            for dst_square in possible_captures.iter_set_bits_as_squares() {
                if dst_square.rank() == promotion_rank {
                    moves.extend(PAWN_PROMOTIONS_LOOKUP.get(src_square, dst_square));
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

    fn add_pseudolegal_en_passant(&self, moves: &mut Vec<Move>) {
        let double_pawn_push = self.context().double_pawn_push;
        let current_side_pawns = self.current_side_pawns();

        if double_pawn_push != -1 {
            let dst_square = self.get_en_passant_dst_square(double_pawn_push);

            for src_square in self.get_possible_en_passant_src_squares(double_pawn_push).into_iter().flatten() {
                if src_square.mask() & self.context().pinned != 0 {
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

    fn add_legal_pawn_pushes(&self, possible_dsts: Bitboard, moves: &mut Vec<Move>) {
        let occupied_mask = self.board.pieces();

        let mut movable_pawns = self.current_side_pawns();

        let pinned_pawns = self.context().pinned & movable_pawns;
        if pinned_pawns != 0 {
            let current_side_king_file = unsafe { Square::from_bitboard(self.current_side_king()) }.file();

            for pinned_pawn_square in pinned_pawns.iter_set_bits_as_squares() {
                if pinned_pawn_square.file() != current_side_king_file {
                    movable_pawns &= !pinned_pawn_square.mask();
                }
            }
        }

        let single_push_dsts = multi_pawn_moves(movable_pawns, self.side_to_move) & !occupied_mask;
        let single_push_dsts_without_check = single_push_dsts & possible_dsts;
        for dst_square in single_push_dsts_without_check.iter_set_bits_as_squares() {
            let src_square = unsafe { self.get_pawn_push_origin(dst_square) };

            if dst_square.rank() == self.current_side_promotion_rank() {
                moves.extend(PAWN_PROMOTIONS_LOOKUP.get(src_square, dst_square));
            } else {
                moves.push(Move::new_non_promotion(dst_square, src_square, MoveFlag::NormalMove));
            }
        }

        let double_push_dsts = multi_pawn_moves(single_push_dsts & self.get_additional_pawn_push_rank_mask(), self.side_to_move) & !occupied_mask & possible_dsts;
        for dst_square in double_push_dsts.iter_set_bits_as_squares() {
            let src_square = unsafe { self.get_pawn_double_push_origin(dst_square) };
            moves.push(Move::new_non_promotion(dst_square, src_square, MoveFlag::NormalMove));
        }
    }

    fn add_pawn_moves(&self, possible_dsts: Bitboard, moves: &mut Vec<Move>) {
        self.add_legal_non_ep_pawn_captures(possible_dsts, moves);
        self.add_pseudolegal_en_passant(moves);
        self.add_legal_pawn_pushes(possible_dsts, moves);
    }

    fn add_legal_knight_moves(&self, possible_dsts: Bitboard, moves: &mut Vec<Move>) {
        let movable_knights = self.board.knights() & self.current_side_pieces() & !self.context().pinned;

        for src_square in movable_knights.iter_set_bits_as_squares() {
            let knight_attacks = single_knight_attacks(src_square);
            let knight_moves = knight_attacks & possible_dsts;

            for dst_square in knight_moves.iter_set_bits_as_squares() {
                moves.push(Move::new_non_promotion(
                    dst_square,
                    src_square,
                    MoveFlag::NormalMove,
                ));
            }
        }
    }

    fn add_legal_sliding_piece_moves(&self, piece: PieceType, possible_dsts: Bitboard, moves: &mut Vec<Move>) {
        let all_occupancy_bb = self.board.pieces();

        let piece_mask = self.board.piece_mask(piece) & self.current_side_pieces();

        for src_square in piece_mask.iter_set_bits_as_squares() {
            let attacks = sliding_piece_attacks(src_square, all_occupancy_bb, piece);
            let mut possible_moves = attacks & possible_dsts;

            if src_square.mask() & self.context().pinned != 0 {
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

    fn add_legal_king_moves(&self, moves: &mut Vec<Move>) {
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
    
    const fn get_castling_king_src_square(&self) -> Square {
        match self.side_to_move {
            Color::White => Square::E1,
            Color::Black => Square::E8,
        }
    }

    fn add_legal_castling_moves(&self, moves: &mut Vec<Move>) {
        let king_src_square = self.get_castling_king_src_square();

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
        let mut moves: Vec<Move> = Vec::with_capacity(35);

        let mut possible_non_king_dsts = !self.current_side_pieces();
        
        match self.context().checkers {
            0 => {
                self.add_pawn_moves(possible_non_king_dsts, &mut moves);
                self.add_legal_knight_moves(possible_non_king_dsts, &mut moves);
                self.add_legal_sliding_piece_moves(PieceType::Bishop, possible_non_king_dsts, &mut moves);
                self.add_legal_sliding_piece_moves(PieceType::Rook, possible_non_king_dsts, &mut moves);
                self.add_legal_sliding_piece_moves(PieceType::Queen, possible_non_king_dsts, &mut moves);
                self.add_legal_king_moves(&mut moves);
                self.add_legal_castling_moves(&mut moves);
            },
            checkers if checkers.count_ones() == 1 => {
                let checker_square = unsafe { Square::from_bitboard(checkers) };
                let is_checker_a_slider = self.board.get_piece_type_at(checker_square).is_sliding_piece();

                if is_checker_a_slider {
                    let possible_move_ray = Bitboard::edge_to_edge_ray(checker_square, unsafe { Square::from_bitboard(self.current_side_king()) });
                    possible_non_king_dsts &= possible_move_ray;
                } else {
                    possible_non_king_dsts = checker_square.mask();
                }

                self.add_pawn_moves(possible_non_king_dsts, &mut moves);
                self.add_legal_knight_moves(possible_non_king_dsts, &mut moves);
                self.add_legal_sliding_piece_moves(PieceType::Bishop, possible_non_king_dsts, &mut moves);
                self.add_legal_sliding_piece_moves(PieceType::Rook, possible_non_king_dsts, &mut moves);
                self.add_legal_sliding_piece_moves(PieceType::Queen, possible_non_king_dsts, &mut moves);
                self.add_legal_king_moves(&mut moves);
            },
            _ => {
                self.add_legal_king_moves(&mut moves);
            }
        }
        
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
