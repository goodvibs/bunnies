//! Move generation functions for the state struct

use static_init::dynamic;
use crate::attacks::{multi_pawn_attacks, multi_pawn_moves, single_king_attacks, single_knight_attacks, sliding_piece_attacks};
use crate::masks::{FILE_A, FILE_H, RANK_3, RANK_6};
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
    /**
    * Adds all legal non-en-passant pawn capture moves to the provided moves vector.
    *
    * Iterates through all pawns of the current side and finds legal capturing moves based on:
    * - Attacks hitting opponent pieces within possible destination squares
    * - Handling pinned pawns by restricting their movement to the pin ray
    * - Creating proper promotion moves when captures land on the promotion rank
    *
    * @param possible_dsts Bitboard representing valid destination squares for moves
    * @param moves Mutable reference to a vector where generated moves will be added
    */
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
    
    const fn get_possible_en_passant_src_squares(&self, double_pawn_push: i8) -> Bitboard {
        assert!(double_pawn_push >= 0 && double_pawn_push <= 7);
        
        let double_pawn_push_dst = match self.side_to_move {
            Color::White => unsafe { Square::from_rank_file(4, double_pawn_push as u8).mask() },
            Color::Black => unsafe { Square::from_rank_file(3, double_pawn_push as u8).mask() }
        };
        
        ((double_pawn_push_dst << 1) & !FILE_H) | ((double_pawn_push_dst >> 1) & !FILE_A)
    }

    const fn get_en_passant_dst_square(&self, double_pawn_push: i8) -> Square {
        assert!(double_pawn_push >= 0 && double_pawn_push <= 7);

        match self.side_to_move {
            Color::White => unsafe { Square::from_rank_file(5, double_pawn_push as u8) },
            Color::Black => unsafe { Square::from_rank_file(2, double_pawn_push as u8) }
        }

    }

    const fn get_en_passant_capture_square(&self, double_pawn_push: i8) -> Square {
        assert!(double_pawn_push >= 0 && double_pawn_push <= 7);

        match self.side_to_move {
            Color::White => unsafe { Square::from_rank_file(4, double_pawn_push as u8) },
            Color::Black => unsafe { Square::from_rank_file(3, double_pawn_push as u8) }
        }

    }

    /**
    * Adds all legal en passant capture moves to the provided moves vector.
    *
    * Handles the complex logic of en passant captures including:
    * - Finding pawns that can perform the capture
    * - Validating that the move is legal (doesn't leave king in check)
    * - Special handling for discovered checks along ranks
    * - Filtering based on pin status of the capturing pawn
    *
    * @param moves Mutable reference to a vector where generated moves will be added
    */
    fn add_legal_en_passants(&self, moves: &mut Vec<Move>) {
        let double_pawn_push = self.context().double_pawn_push;
        let current_side_pawns = self.current_side_pawns();

        if double_pawn_push != -1 {
            let capture_square = self.get_en_passant_capture_square(double_pawn_push);
            let dst_square = self.get_en_passant_dst_square(double_pawn_push);

            for src_square in self.get_possible_en_passant_src_squares(double_pawn_push).iter_set_bits_as_squares() {
                if src_square.mask() & self.context().pinned != 0 {
                    let possible_move_ray = Bitboard::edge_to_edge_ray(src_square, unsafe { Square::from_bitboard(self.current_side_king()) });
                    if possible_move_ray & dst_square.mask() == 0 {
                        continue;
                    }
                }
                
                if src_square.mask() & current_side_pawns != 0 {
                    if self.context().checkers != 0 || self.current_side_king() & src_square.rank_mask() != 0 {
                        let mut board_copy = self.board.clone();

                        board_copy.piece_type_masks[PieceType::Pawn as usize] ^= src_square.mask() | dst_square.mask() | capture_square.mask();
                        board_copy.color_masks[self.side_to_move as usize] ^= src_square.mask() | dst_square.mask();
                        board_copy.color_masks[self.side_to_move.other() as usize] &= !capture_square.mask();
                        board_copy.piece_type_masks[PieceType::ALL_PIECE_TYPES as usize] ^= src_square.mask() | dst_square.mask() | capture_square.mask();

                        if !board_copy.is_square_attacked(unsafe { Square::from_bitboard(self.current_side_king()) }, self.side_to_move.other()) {
                            moves.push(Move::new_non_promotion(dst_square, src_square, MoveFlag::EnPassant));
                        }
                    } else {
                        moves.push(Move::new_non_promotion(dst_square, src_square, MoveFlag::EnPassant));
                    }
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

    /**
    * Adds all legal pawn push moves (non-captures) to the provided moves vector.
    *
    * Handles both single and double pawn pushes, including:
    * - Filtering for occupied squares that block pushes
    * - Handling pinned pawns (which can only move along file pins)
    * - Creating proper promotion moves for pushes that reach the promotion rank
    * - Ensuring all generated moves comply with check evasion requirements
    *
    * @param possible_dsts Bitboard representing valid destination squares for moves
    * @param moves Mutable reference to a vector where generated moves will be added
    */
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

    /**
    * Adds all legal knight moves to the provided moves vector.
    *
    * Generates moves for all knights of the current side that are not pinned
    * (since pinned knights cannot move legally). For each knight, calculates
    * attack squares and filters them by possible destinations.
    *
    * @param possible_dsts Bitboard representing valid destination squares for moves
    * @param moves Mutable reference to a vector where generated moves will be added
    */
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

    /**
    * Adds all legal moves for a specific sliding piece type to the provided moves vector.
    *
    * Handles bishops, rooks, and queens by calculating sliding piece attacks and filtering them:
    * - Respects pins by restricting moves to the pin ray if the piece is pinned
    * - Ensures moves comply with the possible destinations (for check evasion, etc.)
    *
    * @param piece The piece type (Bishop, Rook, or Queen)
    * @param possible_dsts Bitboard representing valid destination squares for moves
    * @param moves Mutable reference to a vector where generated moves will be added
    */
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

    /**
    * Adds all legal king moves (excluding castling) to the provided moves vector.
    *
    * Calculates king attacks and filters them to ensure:
    * - The king doesn't move to a square attacked by opponent pieces
    * - The king doesn't move to a square occupied by friendly pieces
    *
    * @param moves Mutable reference to a vector where generated moves will be added
    */
    fn add_legal_king_moves(&self, moves: &mut Vec<Move>) {
        let current_side_mask = self.current_side_pieces();

        let king_src_bb = self.board.kings() & current_side_mask;
        let king_src_square = unsafe { Square::from_bitboard(king_src_bb) };

        let king_attacks = single_king_attacks(king_src_square);
        let king_moves = king_attacks & !current_side_mask;

        for dst_square in king_moves.iter_set_bits_as_squares() {
            if !self.board.is_square_attacked_after_king_move(dst_square, self.side_to_move.other(), king_src_bb | dst_square.mask()) {
                moves.push(Move::new_non_promotion(
                    dst_square,
                    king_src_square,
                    MoveFlag::NormalMove,
                ));
            }
        }
    }
    
    const fn get_castling_king_src_square(&self) -> Square {
        match self.side_to_move {
            Color::White => Square::E1,
            Color::Black => Square::E8,
        }
    }

    /**
    * Adds all legal castling moves to the provided moves vector.
    *
    * Verifies castling legality and adds the appropriate king moves for:
    * - Kingside castling (short castling)
    * - Queenside castling (long castling)
    *
    * The castling legality checks (king not in check, path clear, etc.) are
    * performed in the can_legally_castle_* methods.
    *
    * @param moves Mutable reference to a vector where generated moves will be added
    */
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
                self.add_legal_non_ep_pawn_captures(possible_non_king_dsts, &mut moves);
                self.add_legal_en_passants(&mut moves);
                self.add_legal_pawn_pushes(possible_non_king_dsts, &mut moves);
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
                    possible_non_king_dsts &= checkers | Bitboard::between(checker_square, unsafe { Square::from_bitboard(self.current_side_king()) });
                } else {
                    possible_non_king_dsts = checker_square.mask();
                }

                self.add_legal_non_ep_pawn_captures(possible_non_king_dsts, &mut moves);
                self.add_legal_en_passants(&mut moves);
                self.add_legal_pawn_pushes(possible_non_king_dsts, &mut moves);
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use crate::{Move, MoveFlag, PieceType, Position, Square};

    fn expected_moves_test<const N: usize>(fen: &str, include_move: fn(Move, &Position) -> bool, expected_moves: [Move; N]) {
        let pos = Position::from_fen(fen).unwrap();
        let moves: Vec<Move> = pos.calc_pseudolegal_moves().into_iter()
            .filter(|mv| include_move(*mv, &pos))
            .collect();

        let expected_moves_set = HashSet::from(expected_moves);

        assert_eq!(moves.len(), expected_moves_set.len());
        assert!(moves.iter().all(|mv| expected_moves_set.contains(mv)));
    }

    #[test]
    fn test_knight_movegen() {
        let is_knight_move = |mv: Move, pos: &Position| pos.current_side_knights() & mv.get_source().mask() != 0;

        expected_moves_test("r5k1/pP1n2np/Q7/bbpnp1R1/Np6/1B6/RPPP2P1/4K1N1 b - - 5 12", is_knight_move,
                            [
                                Move::new_non_promotion(Square::F6, Square::D7, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::F8, Square::D7, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::B6, Square::D7, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::B8, Square::D7, MoveFlag::NormalMove)
                            ]);

        expected_moves_test("Rn3k2/pP1n2np/Q7/bbpnpR2/Np6/1B6/RPPP2P1/4K1N1 b - - 7 13", is_knight_move,
                            [
                                Move::new_non_promotion(Square::F5, Square::G7, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::F6, Square::D5, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::F6, Square::D7, MoveFlag::NormalMove)
                            ]);
    }

    #[test]
    fn test_sliding_piece_movegen() {
        let is_sliding_piece_move = |mv: Move, pos: &Position| (pos.current_side_bishops() | pos.current_side_rooks() | pos.current_side_queens()) & mv.get_source().mask() != 0;

        expected_moves_test("r2q1rk1/pP1q3p/Q4n2/bbp1p3/Np4q1/1B1r1NRn/pPbP1PPP/R3K2R b KQ - 0 1", is_sliding_piece_move,
                            [
                                Move::new_non_promotion(Square::F7, Square::F8, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::D5, Square::D7, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::E6, Square::D7, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::F7, Square::D7, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::C4, Square::B5, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::B3, Square::D3, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::D5, Square::D3, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::B3, Square::C2, MoveFlag::NormalMove)
                            ]);

        expected_moves_test("2B2rk1/pP5p/Q2p1n2/2p1p3/Npq3r1/1B1r1NRn/1P1P1PPP/R3K2R b KQ - 0 1", is_sliding_piece_move,
                            [
                                Move::new_non_promotion(Square::F7, Square::F8, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::E8, Square::F8, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::D8, Square::F8, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::C8, Square::F8, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::F3, Square::D3, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::E3, Square::D3, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::C3, Square::D3, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::B3, Square::D3, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::D2, Square::D3, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::D4, Square::D3, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::D5, Square::D3, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::G3, Square::G4, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::G5, Square::G4, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::G6, Square::G4, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::G7, Square::G4, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::B3, Square::C4, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::D5, Square::C4, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::E6, Square::C4, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::F7, Square::C4, MoveFlag::NormalMove),
                            ]);
    }

    #[test]
    fn test_white_pawn_push_movegen() {
        let is_pawn_push = |mv: Move, pos: &Position| pos.current_side_pawns() & mv.get_source().mask() != 0 && (mv.get_source() as i8 - mv.get_destination() as i8) % 8 == 0;

        expected_moves_test("2bb3k/P1Ppqp1P/bn2pnp1/3Pr3/1p5b/2NQ3p/PPPPPPPP/R3K2R w KQ - 0 1", is_pawn_push,
                            [
                                Move::new_non_promotion(Square::A3, Square::A2, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::A4, Square::A2, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::B3, Square::B2, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::E3, Square::E2, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::E4, Square::E2, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::G3, Square::G2, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::G4, Square::G2, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::D6, Square::D5, MoveFlag::NormalMove),
                                Move::new_promotion(Square::A8, Square::A7, PieceType::Knight),
                                Move::new_promotion(Square::A8, Square::A7, PieceType::Bishop),
                                Move::new_promotion(Square::A8, Square::A7, PieceType::Rook),
                                Move::new_promotion(Square::A8, Square::A7, PieceType::Queen),
                            ]);
    }

    #[test]
    fn test_white_non_ep_pawn_capture_movegen() {
        let is_non_ep_pawn_capture = |mv: Move, pos: &Position| pos.current_side_pawns() & mv.get_source().mask() != 0 && mv.get_flag() != MoveFlag::EnPassant && (mv.get_source() as i8 - mv.get_destination() as i8) % 8 != 0;

        expected_moves_test("1qbb3k/P1PpqP1P/bn2pnp1/3Pr3/1p5b/1nNQ3p/PPPPPPPP/Rqn1Kb1R w KQ - 0 1", is_non_ep_pawn_capture,
                            [
                                Move::new_non_promotion(Square::B3, Square::A2, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::B3, Square::C2, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::H3, Square::G2, MoveFlag::NormalMove),
                                Move::new_promotion(Square::B8, Square::A7, PieceType::Knight),
                                Move::new_promotion(Square::B8, Square::A7, PieceType::Bishop),
                                Move::new_promotion(Square::B8, Square::A7, PieceType::Rook),
                                Move::new_promotion(Square::B8, Square::A7, PieceType::Queen),
                                Move::new_promotion(Square::B8, Square::C7, PieceType::Knight),
                                Move::new_promotion(Square::B8, Square::C7, PieceType::Bishop),
                                Move::new_promotion(Square::B8, Square::C7, PieceType::Rook),
                                Move::new_promotion(Square::B8, Square::C7, PieceType::Queen),
                                Move::new_promotion(Square::D8, Square::C7, PieceType::Knight),
                                Move::new_promotion(Square::D8, Square::C7, PieceType::Bishop),
                                Move::new_promotion(Square::D8, Square::C7, PieceType::Rook),
                                Move::new_promotion(Square::D8, Square::C7, PieceType::Queen),
                                Move::new_non_promotion(Square::E6, Square::D5, MoveFlag::NormalMove),
                            ]);
    }

    #[test]
    fn test_en_passant_movegen() {
        let is_en_passant = |mv: Move, _: &Position| mv.get_flag() == MoveFlag::EnPassant;

        expected_moves_test("8/2p5/3p4/KP5r/1R2Pp1k/8/6P1/8 b - e3 0 1", is_en_passant, []);

        expected_moves_test("8/8/3p4/KPp4r/1R3p1k/8/4P1P1/8 w - c6 0 2", is_en_passant, []);

        expected_moves_test("8/8/3p4/KPpP3r/1R3p1k/8/4P1P1/8 w - c6 0 2", is_en_passant,
                            [
                                Move::new_non_promotion(Square::C6, Square::D5, MoveFlag::EnPassant),
                                Move::new_non_promotion(Square::C6, Square::B5, MoveFlag::EnPassant),
                            ]);

        expected_moves_test("8/B7/3p4/kPpP3r/3K1p2/8/4P1P1/8 w - c6 0 2", is_en_passant,
                            [
                                Move::new_non_promotion(Square::C6, Square::D5, MoveFlag::EnPassant),
                                Move::new_non_promotion(Square::C6, Square::B5, MoveFlag::EnPassant),
                            ]);
        
        expected_moves_test("8/8/b2p4/kPpP3r/2K2p2/8/4P1P1/8 w - c6 0 2", is_en_passant,
                            [
                                Move::new_non_promotion(Square::C6, Square::D5, MoveFlag::EnPassant),
                            ]);
    }

    #[test]
    fn test_king_movegen() {
        let is_king_move = |mv: Move, pos: &Position| mv.get_flag() == MoveFlag::NormalMove && pos.current_side_king() & mv.get_source().mask() != 0;

        expected_moves_test("3N3B/5k1P/R4b2/8/8/3K4/8/8 b - - 0 1", is_king_move,
                            [
                                Move::new_non_promotion(Square::G6, Square::F7, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::F8, Square::F7, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::E8, Square::F7, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::E7, Square::F7, MoveFlag::NormalMove),
                            ]);

        expected_moves_test("5R1B/5k1P/R4b2/8/8/3K4/8/8 b - - 0 1", is_king_move,
                            [
                                Move::new_non_promotion(Square::G6, Square::F7, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::F8, Square::F7, MoveFlag::NormalMove),
                                Move::new_non_promotion(Square::E7, Square::F7, MoveFlag::NormalMove),
                            ]);
    }

    #[test]
    fn test_white_castling_movegen() {
        let is_castling_move = |mv: Move, _: &Position| mv.get_flag() == MoveFlag::Castling;

        expected_moves_test("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", is_castling_move,
                            [
                                Move::new_non_promotion(Square::C1, Square::E1, MoveFlag::Castling),
                                Move::new_non_promotion(Square::G1, Square::E1, MoveFlag::Castling),
                            ]);

        expected_moves_test("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBB1bP/R3K2R w KQkq - 0 1", is_castling_move,
                            [
                                Move::new_non_promotion(Square::C1, Square::E1, MoveFlag::Castling),
                            ]);

        expected_moves_test("4k3/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2b2Q1p/PrPBB1rP/R3K2R w KQ - 0 1", is_castling_move,
                            [
                                Move::new_non_promotion(Square::C1, Square::E1, MoveFlag::Castling),
                            ]);

        expected_moves_test("4k3/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2b2Q1p/PrrBB1RP/R3K2R w KQ - 0 1", is_castling_move,
                            [
                                Move::new_non_promotion(Square::G1, Square::E1, MoveFlag::Castling),
                            ]);

        expected_moves_test("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBB2P/RN2K1nR w KQkq - 0 1", is_castling_move, []);
    }
}
