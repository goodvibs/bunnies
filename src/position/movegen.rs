//! Move generation functions for the state struct

use crate::Square;
use crate::attacks::{
    multi_pawn_attacks, multi_pawn_moves, single_king_attacks, single_knight_attacks,
    sliding_piece_attacks,
};
use crate::masks::{FILE_A, FILE_H, RANK_3, RANK_6};
use crate::r#move::{Move, MoveFlag, MoveList};
use crate::position::legal_gen_kind::LegalGenKind;
use crate::position::Position;
use crate::{Bitboard, Color, Flank};
use crate::{BitboardUtils, Piece};

fn generate_pawn_promotions(src_square: Square, dst_square: Square) -> [Move; 4] {
    Piece::PROMOTION_PIECES
        .map(|promotion_piece| Move::new_promotion(src_square, dst_square, promotion_piece))
}

const fn ep_possible_src_masks(stm: Color, double_pawn_push: i8) -> Bitboard {
    assert!(double_pawn_push >= 0 && double_pawn_push <= 7);

    let double_pawn_push_dst = match stm {
        Color::White => unsafe { Square::from_rank_file(4, double_pawn_push as u8).mask() },
        Color::Black => unsafe { Square::from_rank_file(3, double_pawn_push as u8).mask() },
    };

    ((double_pawn_push_dst << 1) & !FILE_H) | ((double_pawn_push_dst >> 1) & !FILE_A)
}

const fn ep_dst_square(stm: Color, double_pawn_push: i8) -> Square {
    assert!(double_pawn_push >= 0 && double_pawn_push <= 7);

    match stm {
        Color::White => unsafe { Square::from_rank_file(5, double_pawn_push as u8) },
        Color::Black => unsafe { Square::from_rank_file(2, double_pawn_push as u8) },
    }
}

const fn ep_capture_square(stm: Color, double_pawn_push: i8) -> Square {
    assert!(double_pawn_push >= 0 && double_pawn_push <= 7);

    match stm {
        Color::White => unsafe { Square::from_rank_file(4, double_pawn_push as u8) },
        Color::Black => unsafe { Square::from_rank_file(3, double_pawn_push as u8) },
    }
}

unsafe fn pawn_push_origin(stm: Color, dst_square: Square) -> Square {
    match stm {
        Color::White => unsafe { dst_square.down().unwrap_unchecked() },
        Color::Black => unsafe { dst_square.up().unwrap_unchecked() },
    }
}

unsafe fn pawn_double_push_origin(stm: Color, dst_square: Square) -> Square {
    match stm {
        Color::White => unsafe {
            dst_square
                .down()
                .unwrap_unchecked()
                .down()
                .unwrap_unchecked()
        },
        Color::Black => unsafe { dst_square.up().unwrap_unchecked().up().unwrap_unchecked() },
    }
}

const fn additional_pawn_push_rank_mask(stm: Color) -> Bitboard {
    match stm {
        Color::White => RANK_3,
        Color::Black => RANK_6,
    }
}

const fn castling_king_src_square_for(stm: Color) -> Square {
    match stm {
        Color::White => Square::E1,
        Color::Black => Square::E8,
    }
}

impl<const N: usize, const STM: Color> Position<N, STM> {
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
    fn add_legal_non_ep_pawn_captures(
        &self,
        possible_dsts: Bitboard,
        pinned_bb: Bitboard,
        moves: &mut MoveList,
    ) {
        let stm = STM;
        let opposite_side_pieces = self.board.color_mask_at(stm.other());

        let promotion_rank = self.current_side_promotion_rank();

        let current_side_pawns = self.board.piece_mask::<{ Piece::Pawn }>() & self.board.color_mask_at(stm);
        let current_side_king = self.board.piece_mask::<{ Piece::King }>() & self.board.color_mask_at(stm);

        for src in current_side_pawns.iter_set_bits_as_masks() {
            let src_square = unsafe { Square::from_bitboard(src) };

            let mut possible_captures =
                multi_pawn_attacks(src, stm) & opposite_side_pieces & possible_dsts;

            if src_square.mask() & pinned_bb != 0 {
                let possible_move_ray = Bitboard::edge_to_edge_ray(src_square, unsafe {
                    Square::from_bitboard(current_side_king)
                });
                possible_captures &= possible_move_ray;
            }

            for dst_square in possible_captures.iter_set_bits_as_squares() {
                if dst_square.rank() == promotion_rank {
                    moves.push_promotions(generate_pawn_promotions(src_square, dst_square));
                } else {
                    moves.push(Move::new_non_promotion(
                        src_square,
                        dst_square,
                        MoveFlag::NormalMove,
                    ));
                }
            }
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
    fn add_legal_en_passants(&self, pinned_bb: Bitboard, moves: &mut MoveList) {
        let stm = STM;
        let double_pawn_push = self.context().double_pawn_push;
        let current_side_pawns = self.board.piece_mask::<{ Piece::Pawn }>() & self.board.color_mask_at(stm);
        let current_side_king = self.board.piece_mask::<{ Piece::King }>() & self.board.color_mask_at(stm);

        if double_pawn_push != -1 {
            let capture_square = ep_capture_square(stm, double_pawn_push);
            let dst_square = ep_dst_square(stm, double_pawn_push);

            for src_square in ep_possible_src_masks(stm, double_pawn_push).iter_set_bits_as_squares()
            {
                if src_square.mask() & pinned_bb != 0 {
                    let possible_move_ray = Bitboard::edge_to_edge_ray(src_square, unsafe {
                        Square::from_bitboard(current_side_king)
                    });
                    if possible_move_ray & dst_square.mask() == 0 {
                        continue;
                    }
                }

                if src_square.mask() & current_side_pawns != 0 {
                    if self.context().checkers != 0
                        || current_side_king & src_square.rank_mask() != 0
                    {
                        let mut board_copy = self.board.clone();

                        board_copy.move_color(stm, dst_square, src_square);
                        board_copy.move_piece(Piece::Pawn, dst_square, src_square);
                        board_copy.remove_color_at(stm.other(), capture_square);
                        board_copy.remove_piece_at(Piece::Pawn, capture_square);

                        if !board_copy.is_square_attacked(
                            unsafe { Square::from_bitboard(current_side_king) },
                            stm.other(),
                        ) {
                            moves.push(Move::new_non_promotion(
                                src_square,
                                dst_square,
                                MoveFlag::EnPassant,
                            ));
                        }
                    } else {
                        moves.push(Move::new_non_promotion(
                            src_square,
                            dst_square,
                            MoveFlag::EnPassant,
                        ));
                    }
                }
            }
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
    fn add_legal_pawn_pushes(
        &self,
        possible_dsts: Bitboard,
        pinned_bb: Bitboard,
        moves: &mut MoveList,
    ) {
        let stm = STM;
        let occupied_mask = self.board.pieces();

        let mut movable_pawns = self.board.piece_mask::<{ Piece::Pawn }>() & self.board.color_mask_at(stm);

        let pinned_pawns = pinned_bb & movable_pawns;
        if pinned_pawns != 0 {
            let current_side_king = self.board.piece_mask::<{ Piece::King }>() & self.board.color_mask_at(stm);
            let current_side_king_file = unsafe { Square::from_bitboard(current_side_king) }.file();

            for pinned_pawn_square in pinned_pawns.iter_set_bits_as_squares() {
                if pinned_pawn_square.file() != current_side_king_file {
                    movable_pawns &= !pinned_pawn_square.mask();
                }
            }
        }

        let single_push_dsts = multi_pawn_moves(movable_pawns, stm) & !occupied_mask;
        let single_push_dsts_without_check = single_push_dsts & possible_dsts;
        for dst_square in single_push_dsts_without_check.iter_set_bits_as_squares() {
            let src_square = unsafe { pawn_push_origin(stm, dst_square) };

            if dst_square.rank() == self.current_side_promotion_rank() {
                moves.push_promotions(generate_pawn_promotions(src_square, dst_square));
            } else {
                moves.push(Move::new_non_promotion(
                    src_square,
                    dst_square,
                    MoveFlag::NormalMove,
                ));
            }
        }

        let double_push_dsts = multi_pawn_moves(
            single_push_dsts & additional_pawn_push_rank_mask(stm),
            stm,
        ) & !occupied_mask
            & possible_dsts;
        for dst_square in double_push_dsts.iter_set_bits_as_squares() {
            let src_square = unsafe { pawn_double_push_origin(stm, dst_square) };
            moves.push(Move::new_non_promotion(
                src_square,
                dst_square,
                MoveFlag::NormalMove,
            ));
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
    fn add_legal_knight_moves(
        &self,
        possible_dsts: Bitboard,
        pinned_bb: Bitboard,
        moves: &mut MoveList,
    ) {
        let stm = STM;
        let movable_knights = self.board.piece_mask::<{ Piece::Knight }>()
            & self.board.color_mask_at(stm)
            & !pinned_bb;

        for src_square in movable_knights.iter_set_bits_as_squares() {
            let knight_attacks = single_knight_attacks(src_square);
            let knight_moves = knight_attacks & possible_dsts;

            for dst_square in knight_moves.iter_set_bits_as_squares() {
                moves.push(Move::new_non_promotion(
                    src_square,
                    dst_square,
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
    fn add_legal_sliding_piece_moves(
        &self,
        piece: Piece,
        possible_dsts: Bitboard,
        pinned_bb: Bitboard,
        moves: &mut MoveList,
    ) {
        let stm = STM;
        let all_occupancy_bb = self.board.pieces();
        let current_side_king = self.board.piece_mask::<{ Piece::King }>() & self.board.color_mask_at(stm);

        let piece_mask = self.board.piece_mask_at(piece) & self.board.color_mask_at(stm);

        for src_square in piece_mask.iter_set_bits_as_squares() {
            let attacks = sliding_piece_attacks(src_square, all_occupancy_bb, piece);
            let mut possible_moves = attacks & possible_dsts;

            if src_square.mask() & pinned_bb != 0 {
                let possible_move_ray = Bitboard::edge_to_edge_ray(src_square, unsafe {
                    Square::from_bitboard(current_side_king)
                });
                possible_moves &= possible_move_ray;
            }

            for dst_square in possible_moves.iter_set_bits_as_squares() {
                moves.push(Move::new_non_promotion(
                    src_square,
                    dst_square,
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
    fn add_legal_king_moves(&self, moves: &mut MoveList) {
        let stm = STM;
        let current_side_mask = self.board.color_mask_at(stm);

        let king_src_bb = self.board.piece_mask::<{ Piece::King }>() & current_side_mask;
        let king_src_square = unsafe { Square::from_bitboard(king_src_bb) };

        let king_attacks = single_king_attacks(king_src_square);
        let king_moves = king_attacks & !current_side_mask;

        for dst_square in king_moves.iter_set_bits_as_squares() {
            if !self.board.is_square_attacked_after_king_move(
                dst_square,
                stm.other(),
                king_src_bb | dst_square.mask(),
            ) {
                moves.push(Move::new_non_promotion(
                    king_src_square,
                    dst_square,
                    MoveFlag::NormalMove,
                ));
            }
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
     * performed in [`Position::can_legally_castle`].
     *
     * @param moves Mutable reference to a vector where generated moves will be added
     */
    fn add_legal_castling_moves(&self, moves: &mut MoveList) {
        let king_src_square = castling_king_src_square_for(STM);

        for flank in Flank::ALL {
            if self.can_legally_castle(flank) {
                let king_dst_square = match flank {
                    Flank::Kingside => unsafe { Square::from(king_src_square as u8 + 2) },
                    Flank::Queenside => unsafe { Square::from(king_src_square as u8 - 2) },
                };
                moves.push(Move::new_non_promotion(
                    king_src_square,
                    king_dst_square,
                    MoveFlag::Castling,
                ));
            }
        }
    }

    /// Fills `out` with all legal moves (stack allocation only; clears `out` first).
    pub fn generate_legal_moves(&self, out: &mut MoveList) {
        out.clear();

        let pinned_bb = self.context().pinned;
        let mut possible_non_king_dsts = !self.board.color_mask_at(STM);

        match self.context().checkers {
            0 => {
                self.add_legal_non_ep_pawn_captures(possible_non_king_dsts, pinned_bb, out);
                self.add_legal_en_passants(pinned_bb, out);
                self.add_legal_pawn_pushes(possible_non_king_dsts, pinned_bb, out);
                self.add_legal_knight_moves(possible_non_king_dsts, pinned_bb, out);
                self.add_legal_sliding_piece_moves(Piece::Bishop, possible_non_king_dsts, pinned_bb, out);
                self.add_legal_sliding_piece_moves(Piece::Rook, possible_non_king_dsts, pinned_bb, out);
                self.add_legal_sliding_piece_moves(Piece::Queen, possible_non_king_dsts, pinned_bb, out);
                self.add_legal_king_moves(out);
                self.add_legal_castling_moves(out);
            }
            checkers if checkers.count_ones() == 1 => {
                let checker_square = unsafe { Square::from_bitboard(checkers) };
                let is_checker_a_slider = self.board.piece_at(checker_square).is_sliding_piece();

                let current_side_king =
                    self.board.piece_mask::<{ Piece::King }>() & self.board.color_mask_at(STM);

                if is_checker_a_slider {
                    possible_non_king_dsts &= checkers
                        | Bitboard::between(checker_square, unsafe {
                            Square::from_bitboard(current_side_king)
                        });
                } else {
                    possible_non_king_dsts = checker_square.mask();
                }

                self.add_legal_non_ep_pawn_captures(possible_non_king_dsts, pinned_bb, out);
                self.add_legal_en_passants(pinned_bb, out);
                self.add_legal_pawn_pushes(possible_non_king_dsts, pinned_bb, out);
                self.add_legal_knight_moves(possible_non_king_dsts, pinned_bb, out);
                self.add_legal_sliding_piece_moves(Piece::Bishop, possible_non_king_dsts, pinned_bb, out);
                self.add_legal_sliding_piece_moves(Piece::Rook, possible_non_king_dsts, pinned_bb, out);
                self.add_legal_sliding_piece_moves(Piece::Queen, possible_non_king_dsts, pinned_bb, out);
                self.add_legal_king_moves(out);
            }
            _ => {
                self.add_legal_king_moves(out);
            }
        }
    }

    /// Pseudo-legal move generation for fast search.
    ///
    /// **Currently** this is identical to [`Self::generate_legal_moves`]. A dedicated
    /// pseudo-legal path (faster, then filtered with [`Self::is_legal_move`]) is left as a future optimization.
    pub fn generate_pseudolegal_moves(&self, out: &mut MoveList) {
        self.generate_legal_moves(out);
    }

    /// Whether `mv` is fully legal from this position (mover's king not left in check).
    pub fn is_legal_move(&self, mv: Move) -> bool {
        let mut clone = self.clone();
        if clone.make_move_in_place(mv).is_err() {
            return false;
        }
        !clone.is_opposite_side_in_check()
    }

    /// Capture or en passant (for move ordering); castling and quiet promotions are not captures.
    pub fn is_capture_move(&self, mv: Move) -> bool {
        match mv.flag() {
            MoveFlag::Castling => false,
            MoveFlag::EnPassant => true,
            MoveFlag::NormalMove | MoveFlag::Promotion => {
                let dst = mv.destination();
                if !self.board.is_occupied_at(dst) {
                    return false;
                }
                self.board.get_colored_piece_at(dst).color() != STM
            }
        }
    }

    /// Legal moves, optionally restricted to captures or quiets (post-filter; allocates a temp list).
    pub fn generate_legal_moves_kind(&self, kind: LegalGenKind, out: &mut MoveList) {
        match kind {
            LegalGenKind::All => self.generate_legal_moves(out),
            LegalGenKind::Captures | LegalGenKind::Quiets => {
                let mut tmp = MoveList::new();
                self.generate_legal_moves(&mut tmp);
                out.clear();
                let want_captures = matches!(kind, LegalGenKind::Captures);
                for &mv in tmp.as_slice() {
                    let is_cap = self.is_capture_move(mv);
                    if want_captures == is_cap {
                        out.push(mv);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::position::{LegalGenKind, Position};
    use crate::{Color, Move, MoveFlag, MoveList, Piece, Square, TypedPosition};
    use std::collections::HashSet;

    fn expected_moves_test<const M: usize>(
        fen: &str,
        include_move: fn(Move, &TypedPosition<1>) -> bool,
        expected_moves: [Move; M],
    ) {
        let pos = TypedPosition::<1>::from_fen(fen).unwrap();
        let mut legal = MoveList::new();
        pos.generate_legal_moves(&mut legal);
        let mut moves_set = HashSet::new();
        for mv in legal.as_slice().iter().copied() {
            if include_move(mv, &pos) {
                moves_set.insert(mv);
            }
        }

        let expected_moves_set = HashSet::from(expected_moves);

        assert_eq!(moves_set.len(), expected_moves_set.len());
        assert!(moves_set.iter().all(|mv| expected_moves_set.contains(mv)));
    }

    #[test]
    fn test_knight_movegen() {
        let is_knight_move = |mv: Move, pos: &TypedPosition<1>| {
            pos.board().piece_mask::<{ Piece::Knight }>()
                & pos.board().color_mask_at(pos.side_to_move())
                & mv.source().mask()
                != 0
        };

        expected_moves_test(
            "r5k1/pP1n2np/Q7/bbpnp1R1/Np6/1B6/RPPP2P1/4K1N1 b - - 5 12",
            is_knight_move,
            [
                Move::new_non_promotion(Square::D7, Square::F6, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::D7, Square::F8, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::D7, Square::B6, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::D7, Square::B8, MoveFlag::NormalMove),
            ],
        );

        expected_moves_test(
            "Rn3k2/pP1n2np/Q7/bbpnpR2/Np6/1B6/RPPP2P1/4K1N1 b - - 7 13",
            is_knight_move,
            [
                Move::new_non_promotion(Square::G7, Square::F5, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::D5, Square::F6, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::D7, Square::F6, MoveFlag::NormalMove),
            ],
        );
    }

    #[test]
    fn test_sliding_piece_movegen() {
        let is_sliding_piece_move = |mv: Move, pos: &TypedPosition<1>| {
            let stm = pos.side_to_move();
            let cur = pos.board().color_mask_at(stm);
            (pos.board().piece_mask::<{ Piece::Bishop }>()
                | pos.board().piece_mask::<{ Piece::Rook }>()
                | pos.board().piece_mask::<{ Piece::Queen }>())
                & cur
                & mv.source().mask()
                != 0
        };

        expected_moves_test(
            "r2q1rk1/pP1q3p/Q4n2/bbp1p3/Np4q1/1B1r1NRn/pPbP1PPP/R3K2R b KQ - 0 1",
            is_sliding_piece_move,
            [
                Move::new_non_promotion(Square::F8, Square::F7, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::D7, Square::D5, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::D7, Square::E6, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::D7, Square::F7, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::B5, Square::C4, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::D3, Square::B3, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::D3, Square::D5, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::C2, Square::B3, MoveFlag::NormalMove),
            ],
        );

        expected_moves_test(
            "2B2rk1/pP5p/Q2p1n2/2p1p3/Npq3r1/1B1r1NRn/1P1P1PPP/R3K2R b KQ - 0 1",
            is_sliding_piece_move,
            [
                Move::new_non_promotion(Square::F8, Square::F7, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::F8, Square::E8, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::F8, Square::D8, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::F8, Square::C8, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::D3, Square::F3, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::D3, Square::E3, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::D3, Square::C3, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::D3, Square::B3, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::D3, Square::D2, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::D3, Square::D4, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::D3, Square::D5, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::G4, Square::G3, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::G4, Square::G5, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::G4, Square::G6, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::G4, Square::G7, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::C4, Square::B3, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::C4, Square::D5, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::C4, Square::E6, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::C4, Square::F7, MoveFlag::NormalMove),
            ],
        );
    }

    #[test]
    fn test_white_pawn_push_movegen() {
        let is_pawn_push = |mv: Move, pos: &TypedPosition<1>| {
            pos.board().piece_mask::<{ Piece::Pawn }>()
                & pos.board().color_mask_at(pos.side_to_move())
                & mv.source().mask()
                != 0
                && (mv.source() as i8 - mv.destination() as i8) % 8 == 0
        };

        expected_moves_test(
            "2bb3k/P1Ppqp1P/bn2pnp1/3Pr3/1p5b/2NQ3p/PPPPPPPP/R3K2R w KQ - 0 1",
            is_pawn_push,
            [
                Move::new_non_promotion(Square::A2, Square::A3, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::A2, Square::A4, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::B2, Square::B3, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::E2, Square::E3, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::E2, Square::E4, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::G2, Square::G3, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::G2, Square::G4, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::D5, Square::D6, MoveFlag::NormalMove),
                Move::new_promotion(Square::A7, Square::A8, Piece::Knight),
                Move::new_promotion(Square::A7, Square::A8, Piece::Bishop),
                Move::new_promotion(Square::A7, Square::A8, Piece::Rook),
                Move::new_promotion(Square::A7, Square::A8, Piece::Queen),
            ],
        );
    }

    #[test]
    fn test_white_non_ep_pawn_capture_movegen() {
        let is_non_ep_pawn_capture = |mv: Move, pos: &TypedPosition<1>| {
            pos.board().piece_mask::<{ Piece::Pawn }>()
                & pos.board().color_mask_at(pos.side_to_move())
                & mv.source().mask()
                != 0
                && mv.flag() != MoveFlag::EnPassant
                && (mv.source() as i8 - mv.destination() as i8) % 8 != 0
        };

        expected_moves_test(
            "1qbb3k/P1PpqP1P/bn2pnp1/3Pr3/1p5b/1nNQ3p/PPPPPPPP/Rqn1Kb1R w KQ - 0 1",
            is_non_ep_pawn_capture,
            [
                Move::new_non_promotion(Square::A2, Square::B3, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::C2, Square::B3, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::G2, Square::H3, MoveFlag::NormalMove),
                Move::new_promotion(Square::A7, Square::B8, Piece::Knight),
                Move::new_promotion(Square::A7, Square::B8, Piece::Bishop),
                Move::new_promotion(Square::A7, Square::B8, Piece::Rook),
                Move::new_promotion(Square::A7, Square::B8, Piece::Queen),
                Move::new_promotion(Square::C7, Square::B8, Piece::Knight),
                Move::new_promotion(Square::C7, Square::B8, Piece::Bishop),
                Move::new_promotion(Square::C7, Square::B8, Piece::Rook),
                Move::new_promotion(Square::C7, Square::B8, Piece::Queen),
                Move::new_promotion(Square::C7, Square::D8, Piece::Knight),
                Move::new_promotion(Square::C7, Square::D8, Piece::Bishop),
                Move::new_promotion(Square::C7, Square::D8, Piece::Rook),
                Move::new_promotion(Square::C7, Square::D8, Piece::Queen),
                Move::new_non_promotion(Square::D5, Square::E6, MoveFlag::NormalMove),
            ],
        );
    }

    #[test]
    fn test_en_passant_movegen() {
        let is_en_passant = |mv: Move, _: &TypedPosition<1>| mv.flag() == MoveFlag::EnPassant;

        expected_moves_test(
            "8/2p5/3p4/KP5r/1R2Pp1k/8/6P1/8 b - e3 0 1",
            is_en_passant,
            [],
        );

        expected_moves_test(
            "8/8/3p4/KPp4r/1R3p1k/8/4P1P1/8 w - c6 0 2",
            is_en_passant,
            [],
        );

        expected_moves_test(
            "8/8/3p4/KPpP3r/1R3p1k/8/4P1P1/8 w - c6 0 2",
            is_en_passant,
            [
                Move::new_non_promotion(Square::D5, Square::C6, MoveFlag::EnPassant),
                Move::new_non_promotion(Square::B5, Square::C6, MoveFlag::EnPassant),
            ],
        );

        expected_moves_test(
            "8/B7/3p4/kPpP3r/3K1p2/8/4P1P1/8 w - c6 0 2",
            is_en_passant,
            [
                Move::new_non_promotion(Square::D5, Square::C6, MoveFlag::EnPassant),
                Move::new_non_promotion(Square::B5, Square::C6, MoveFlag::EnPassant),
            ],
        );

        expected_moves_test(
            "8/8/b2p4/kPpP3r/2K2p2/8/4P1P1/8 w - c6 0 2",
            is_en_passant,
            [Move::new_non_promotion(
                Square::D5,
                Square::C6,
                MoveFlag::EnPassant,
            )],
        );
    }

    #[test]
    fn test_king_movegen() {
        let is_king_move = |mv: Move, pos: &TypedPosition<1>| {
            mv.flag() == MoveFlag::NormalMove
                && pos.board().piece_mask::<{ Piece::King }>()
                    & pos.board().color_mask_at(pos.side_to_move())
                    & mv.source().mask()
                    != 0
        };

        expected_moves_test(
            "3N3B/5k1P/R4b2/8/8/3K4/8/8 b - - 0 1",
            is_king_move,
            [
                Move::new_non_promotion(Square::F7, Square::G6, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::F7, Square::F8, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::F7, Square::E8, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::F7, Square::E7, MoveFlag::NormalMove),
            ],
        );

        expected_moves_test(
            "5R1B/5k1P/R4b2/8/8/3K4/8/8 b - - 0 1",
            is_king_move,
            [
                Move::new_non_promotion(Square::F7, Square::G6, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::F7, Square::F8, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::F7, Square::E7, MoveFlag::NormalMove),
            ],
        );
    }

    #[test]
    fn test_white_castling_movegen() {
        let is_castling_move = |mv: Move, _: &TypedPosition<1>| mv.flag() == MoveFlag::Castling;

        expected_moves_test(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            is_castling_move,
            [
                Move::new_non_promotion(Square::E1, Square::C1, MoveFlag::Castling),
                Move::new_non_promotion(Square::E1, Square::G1, MoveFlag::Castling),
            ],
        );

        expected_moves_test(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBB1bP/R3K2R w KQkq - 0 1",
            is_castling_move,
            [Move::new_non_promotion(
                Square::E1,
                Square::C1,
                MoveFlag::Castling,
            )],
        );

        expected_moves_test(
            "4k3/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2b2Q1p/PrPBB1rP/R3K2R w KQ - 0 1",
            is_castling_move,
            [Move::new_non_promotion(
                Square::E1,
                Square::C1,
                MoveFlag::Castling,
            )],
        );

        expected_moves_test(
            "4k3/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2b2Q1p/PrrBB1RP/R3K2R w KQ - 0 1",
            is_castling_move,
            [Move::new_non_promotion(
                Square::E1,
                Square::G1,
                MoveFlag::Castling,
            )],
        );

        expected_moves_test(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBB2P/RN2K1nR w KQkq - 0 1",
            is_castling_move,
            [],
        );
    }

    #[test]
    fn pseudo_generator_matches_legal_until_fast_impl() {
        let pos = TypedPosition::<1>::from_fen(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        )
        .unwrap();
        let mut legal = MoveList::new();
        let mut pseudo = MoveList::new();
        pos.generate_legal_moves(&mut legal);
        pos.generate_pseudolegal_moves(&mut pseudo);
        assert_eq!(legal.as_slice(), pseudo.as_slice());
    }

    #[test]
    fn legal_gen_kind_partitions_startpos() {
        let pos = Position::<1, { Color::White }>::initial();
        let mut all = MoveList::new();
        let mut cap = MoveList::new();
        let mut quiet = MoveList::new();
        pos.generate_legal_moves_kind(LegalGenKind::All, &mut all);
        pos.generate_legal_moves_kind(LegalGenKind::Captures, &mut cap);
        pos.generate_legal_moves_kind(LegalGenKind::Quiets, &mut quiet);
        assert_eq!(all.len(), cap.len() + quiet.len());
    }
}
