//! Move generation: small pure helpers for masks, then writers that take explicit
//! bitboards and closures only where attack or castling needs hidden board state.

use crate::Rank;
use crate::Square;
use crate::SquareDelta;
use crate::SquareDeltaUtils;
use crate::attacks::manual::multi_pawn_attacks_left;
use crate::attacks::manual::multi_pawn_attacks_right;
use crate::attacks::{
    multi_pawn_attacks, multi_pawn_moves, single_king_attacks, single_knight_attacks,
    sliding_piece_attacks,
};
use crate::r#move::{Move, MoveFlag, MoveList};
use crate::position::Position;
use crate::{Bitboard, Color, DoublePawnPushFile, Flank};
use crate::{BitboardUtils, ConstBitboardGeometry, ConstDoublePawnPushFile, Piece};

/// Returns `to_bb` restricted to squares legal for `from` given current pins.
/// For non-pinned pieces, returns `to_bb` unchanged. Branchless on the hot path.
#[inline]
fn pin_restrict(from: Square, to_bb: Bitboard, king_sq: Square, pinned: Bitboard) -> Bitboard {
    let pin_mask = if from.mask() & pinned != 0 {
        Bitboard::edge_to_edge_ray(from, king_sq)
    } else {
        !0
    };
    to_bb & pin_mask
}

fn generate_pawn_promotions(src_square: Square, dst_square: Square) -> [Move; 4] {
    Piece::PROMOTION_PIECES
        .map(|promotion_piece| Move::new_promotion(src_square, dst_square, promotion_piece))
}

/// EP pseudo-move may expose the king along a rank; needs full attack evaluation.
#[inline]
fn en_passant_requires_full_attack_probe(
    checkers: Bitboard,
    king_sq: Square,
    pawn_src: Square,
) -> bool {
    checkers != 0 || king_sq.rank() == pawn_src.rank()
}

fn splat_promotions(sd: SquareDelta, to_bb: Bitboard, moves: &mut MoveList) {
    for dst in to_bb.iter_set_bits_as_squares() {
        let from = dst.relative(sd).expect("Invalid SquareDelta for to_bb");
        moves.push_all(generate_pawn_promotions(from, dst));
    }
}

fn splat_normal_pawn_moves(sd: SquareDelta, to_bb: Bitboard, moves: &mut MoveList) {
    for dst in to_bb.iter_set_bits_as_squares() {
        let from = dst.relative(sd).expect("Invalid SquareDelta for to_bb");
        moves.push(Move::new_non_promotion(from, dst, MoveFlag::NormalMove));
    }
}

fn splat_moves(from: Square, to_bb: Bitboard, moves: &mut MoveList) {
    for dst in to_bb.iter_set_bits_as_squares() {
        moves.push(Move::new_non_promotion(from, dst, MoveFlag::NormalMove));
    }
}

/// Splits `dsts` into promotions (on `promo_rank`) and the rest, emitting both via the
/// per-delta splat helpers. `sd` is the source-from-destination delta for this attack direction.
#[inline]
fn emit_pawn_dsts(sd: SquareDelta, dsts: Bitboard, promo_rank: Bitboard, moves: &mut MoveList) {
    let promotions = dsts & promo_rank;
    splat_normal_pawn_moves(sd, dsts & !promotions, moves);
    splat_promotions(sd, promotions, moves);
}

fn add_legal_pawn_captures<const STM: Color>(
    stm_pawns: Bitboard,
    opposite_pieces: Bitboard,
    king_sq: Square,
    dst_mask: Bitboard,
    pinned: Bitboard,
    moves: &mut MoveList,
) {
    let up_left = SquareDelta::UP_LEFT.from_perspective(STM);
    let up_right = SquareDelta::UP_RIGHT.from_perspective(STM);
    let down_right = up_left * -1;
    let down_left = up_right * -1;
    let promo_rank = Rank::Eight.from_perspective(STM).mask();

    // Free pawns: batch attack generation, no pin reasoning required.
    let free = stm_pawns & !pinned;
    let left = multi_pawn_attacks_left(free, STM) & opposite_pieces & dst_mask;
    let right = multi_pawn_attacks_right(free, STM) & opposite_pieces & dst_mask;
    emit_pawn_dsts(down_right, left, promo_rank, moves);
    emit_pawn_dsts(down_left, right, promo_rank, moves);

    // Pinned pawns (rare): per-source emission so the pin restriction is just an AND.
    for from in (stm_pawns & pinned).iter_set_bits_as_squares() {
        let attacks = multi_pawn_attacks(from.mask(), STM)
            & opposite_pieces
            & dst_mask
            & Bitboard::edge_to_edge_ray(from, king_sq);
        let promotions = attacks & promo_rank;
        for dst in (attacks & !promotions).iter_set_bits_as_squares() {
            moves.push(Move::new_non_promotion(from, dst, MoveFlag::NormalMove));
        }
        for dst in promotions.iter_set_bits_as_squares() {
            moves.push_all(generate_pawn_promotions(from, dst));
        }
    }
}

fn add_legal_en_passants<const STM: Color>(
    dpf: DoublePawnPushFile,
    checkers: Bitboard,
    stm_pawns: Bitboard,
    king_sq: Square,
    pinned: Bitboard,
    ep_is_legal: impl Fn(Square, Square, Square) -> bool,
    moves: &mut MoveList,
) {
    if !dpf.is_some() {
        return;
    }

    if checkers.count_ones() > 1 {
        return;
    }

    let capture_square = dpf.ep_capture_square(STM);
    let to = dpf.ep_dst_square(STM);
    let to_bb = to.mask();

    for from in (dpf.ep_possible_src_mask(STM) & stm_pawns).iter_set_bits_as_squares() {
        if pin_restrict(from, to_bb, king_sq, pinned) == 0 {
            continue;
        }

        let needs_probe = en_passant_requires_full_attack_probe(checkers, king_sq, from);
        if needs_probe && !ep_is_legal(from, to, capture_square) {
            continue;
        }
        moves.push(Move::new_non_promotion(from, to, MoveFlag::EnPassant));
    }
}

fn add_legal_pawn_pushes<const STM: Color>(
    occupied: Bitboard,
    pawns_stm: Bitboard,
    king_sq: Square,
    dst_mask: Bitboard,
    pinned: Bitboard,
    moves: &mut MoveList,
) {
    // Pinned pawns can only push if pinned along the king's file (vertical pin).
    let king_file_mask = king_sq.file().mask();
    let movable_pawns = pawns_stm & !(pinned & !king_file_mask);

    let promo_rank = Rank::Eight.from_perspective(STM).mask();
    let push_again_mask = Rank::Three.from_perspective(STM).mask();
    let down = SquareDelta::DOWN.from_perspective(STM);

    let single_push_dsts = multi_pawn_moves(movable_pawns, STM) & !occupied;
    emit_pawn_dsts(down, single_push_dsts & dst_mask, promo_rank, moves);

    let double_push_dsts =
        multi_pawn_moves(single_push_dsts & push_again_mask, STM) & !occupied & dst_mask;
    splat_normal_pawn_moves(down * 2, double_push_dsts, moves);
}

fn add_legal_knight_moves(
    stm_knights_not_pinned: Bitboard,
    dst_mask: Bitboard,
    moves: &mut MoveList,
) {
    for src_square in stm_knights_not_pinned.iter_set_bits_as_squares() {
        let to_bb = single_knight_attacks(src_square) & dst_mask;
        splat_moves(src_square, to_bb, moves);
    }
}

fn add_legal_sliding_moves<const P: Piece>(
    occupancy: Bitboard,
    stm_pieces_of_kind: Bitboard,
    king_sq: Square,
    dst_mask: Bitboard,
    pinned: Bitboard,
    moves: &mut MoveList,
) {
    for from in stm_pieces_of_kind.iter_set_bits_as_squares() {
        let to_bb = sliding_piece_attacks(from, occupancy, P) & dst_mask;
        splat_moves(from, pin_restrict(from, to_bb, king_sq, pinned), moves);
    }
}

/// `king_dst_is_safe(dst, king_src_bb | dst.mask())` must be true iff the king may step to `dst`.
fn add_legal_king_moves(
    king_sq: Square,
    stm_occupancy: Bitboard,
    king_bb: Bitboard,
    king_dst_is_safe: impl Fn(Square, Bitboard) -> bool,
    moves: &mut MoveList,
) {
    let king_moves = single_king_attacks(king_sq) & !stm_occupancy;

    for dst_square in king_moves.iter_set_bits_as_squares() {
        if king_dst_is_safe(dst_square, king_bb | dst_square.mask()) {
            moves.push(Move::new_non_promotion(
                king_sq,
                dst_square,
                MoveFlag::NormalMove,
            ));
        }
    }
}

fn add_legal_castling_moves<const STM: Color>(
    may_castle: impl Fn(Flank) -> bool,
    moves: &mut MoveList,
) {
    let king_src_square = STM.king_initial_square();

    for flank in Flank::ALL {
        if may_castle(flank) {
            moves.push(Move::new_non_promotion(
                king_src_square,
                flank.king_castled_square(STM),
                MoveFlag::Castling,
            ));
        }
    }
}

impl<const N: usize, const STM: Color> Position<N, STM> {
    /// Fills `moves` with all legal moves (does not clear `moves`; clear or use a fresh list if needed).
    pub fn generate_legal_moves(&self, moves: &mut MoveList) {
        let ctx = self.context();
        let board = &self.board;
        let king_sq = self.king_square(STM);
        let stm_pieces = board.color_mask_at(STM);
        let stm_king_bb = stm_pieces & board.piece_mask::<{ Piece::King }>();

        // 1. King moves are always legal candidates, regardless of check status.
        add_legal_king_moves(
            king_sq,
            stm_pieces,
            stm_king_bb,
            |dst, occ| !board.is_square_attacked_after_move(dst, STM.other(), occ),
            moves,
        );

        // 2. Double check: only the king can move.
        if ctx.checkers.count_ones() > 1 {
            return;
        }

        // 3. Single / no check: compute destination mask + castling eligibility.
        //    - No check: any non-friendly square; castling allowed.
        //    - Single check: must capture the checker, or (for sliders) interpose; no castling.
        let (dst_mask, allow_castling) = match Square::from_bitboard(ctx.checkers) {
            None => (!stm_pieces, true),
            Some(checker_sq) => {
                let resolve_mask = if board.piece_at(checker_sq).is_sliding_piece() {
                    !stm_pieces & (ctx.checkers | Bitboard::between(checker_sq, king_sq))
                } else {
                    checker_sq.mask()
                };
                (resolve_mask, false)
            }
        };

        // 4. Emit pawns, knights, sliders, castling.
        let pawns = stm_pieces & board.piece_mask::<{ Piece::Pawn }>();
        let opposite = board.color_mask_at(STM.other());
        let occupied = board.pieces();

        add_legal_pawn_captures::<STM>(pawns, opposite, king_sq, dst_mask, ctx.pinned, moves);

        add_legal_en_passants::<STM>(
            ctx.double_pawn_push_file,
            ctx.checkers,
            pawns,
            king_sq,
            ctx.pinned,
            |src, dst, capture_square| {
                !board.is_square_attacked_after_move(
                    king_sq,
                    STM.other(),
                    src.mask() | dst.mask() | capture_square.mask(),
                )
            },
            moves,
        );

        add_legal_pawn_pushes::<STM>(occupied, pawns, king_sq, dst_mask, ctx.pinned, moves);

        let knights = stm_pieces & board.piece_mask::<{ Piece::Knight }>() & !ctx.pinned;
        add_legal_knight_moves(knights, dst_mask, moves);

        add_legal_sliding_moves::<{ Piece::Bishop }>(
            occupied,
            stm_pieces & board.piece_mask::<{ Piece::Bishop }>(),
            king_sq,
            dst_mask,
            ctx.pinned,
            moves,
        );
        add_legal_sliding_moves::<{ Piece::Rook }>(
            occupied,
            stm_pieces & board.piece_mask::<{ Piece::Rook }>(),
            king_sq,
            dst_mask,
            ctx.pinned,
            moves,
        );
        add_legal_sliding_moves::<{ Piece::Queen }>(
            occupied,
            stm_pieces & board.piece_mask::<{ Piece::Queen }>(),
            king_sq,
            dst_mask,
            ctx.pinned,
            moves,
        );

        if allow_castling {
            add_legal_castling_moves::<STM>(|flank| self.can_legally_castle(flank), moves);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::position::Position;
    use crate::{Color, Move, MoveFlag, MoveList, Piece, Square};
    use std::collections::HashSet;

    fn expected_moves_test_for_position<const M: usize, const STM: Color>(
        pos: &Position<1, STM>,
        include_move: fn(Move, &Position<1, STM>) -> bool,
        expected_moves: [Move; M],
    ) {
        let mut legal = MoveList::new();
        pos.generate_legal_moves(&mut legal);
        let mut moves_set = HashSet::new();
        for mv in legal.as_slice().iter().copied() {
            if include_move(mv, pos) {
                moves_set.insert(mv);
            }
        }

        let expected_moves_set = HashSet::from(expected_moves);

        assert_eq!(moves_set.len(), expected_moves_set.len());
        assert!(moves_set.iter().all(|mv| expected_moves_set.contains(mv)));
    }

    fn expected_moves_test<const M: usize>(
        fen: &str,
        include_move_white: fn(Move, &Position<1, { Color::White }>) -> bool,
        include_move_black: fn(Move, &Position<1, { Color::Black }>) -> bool,
        expected_moves: [Move; M],
    ) {
        let side_to_move = fen
            .split_ascii_whitespace()
            .nth(1)
            .expect("valid FEN has side-to-move field");
        match side_to_move {
            "w" => {
                let pos = Position::<1, { Color::White }>::from_fen(fen).unwrap();
                expected_moves_test_for_position(&pos, include_move_white, expected_moves);
            }
            "b" => {
                let pos = Position::<1, { Color::Black }>::from_fen(fen).unwrap();
                expected_moves_test_for_position(&pos, include_move_black, expected_moves);
            }
            _ => panic!("invalid side-to-move in FEN"),
        }
    }

    #[test]
    fn test_knight_movegen() {
        let is_knight_move_white = |mv: Move, pos: &Position<1, { Color::White }>| {
            pos.board.piece_mask::<{ Piece::Knight }>()
                & pos.board.color_mask_at(Color::White)
                & mv.source().mask()
                != 0
        };
        let is_knight_move_black = |mv: Move, pos: &Position<1, { Color::Black }>| {
            pos.board.piece_mask::<{ Piece::Knight }>()
                & pos.board.color_mask_at(Color::Black)
                & mv.source().mask()
                != 0
        };

        expected_moves_test(
            "r5k1/pP1n2np/Q7/bbpnp1R1/Np6/1B6/RPPP2P1/4K1N1 b - - 5 12",
            is_knight_move_white,
            is_knight_move_black,
            [
                Move::new_non_promotion(Square::D7, Square::F6, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::D7, Square::F8, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::D7, Square::B6, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::D7, Square::B8, MoveFlag::NormalMove),
            ],
        );

        expected_moves_test(
            "Rn3k2/pP1n2np/Q7/bbpnpR2/Np6/1B6/RPPP2P1/4K1N1 b - - 7 13",
            is_knight_move_white,
            is_knight_move_black,
            [
                Move::new_non_promotion(Square::G7, Square::F5, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::D5, Square::F6, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::D7, Square::F6, MoveFlag::NormalMove),
            ],
        );
    }

    #[test]
    fn test_sliding_piece_movegen() {
        let is_sliding_piece_move_white = |mv: Move, pos: &Position<1, { Color::White }>| {
            let cur = pos.board.color_mask_at(Color::White);
            (pos.board.piece_mask::<{ Piece::Bishop }>()
                | pos.board.piece_mask::<{ Piece::Rook }>()
                | pos.board.piece_mask::<{ Piece::Queen }>())
                & cur
                & mv.source().mask()
                != 0
        };
        let is_sliding_piece_move_black = |mv: Move, pos: &Position<1, { Color::Black }>| {
            let cur = pos.board.color_mask_at(Color::Black);
            (pos.board.piece_mask::<{ Piece::Bishop }>()
                | pos.board.piece_mask::<{ Piece::Rook }>()
                | pos.board.piece_mask::<{ Piece::Queen }>())
                & cur
                & mv.source().mask()
                != 0
        };

        expected_moves_test(
            "r2q1rk1/pP1q3p/Q4n2/bbp1p3/Np4q1/1B1r1NRn/pPbP1PPP/R3K2R b KQ - 0 1",
            is_sliding_piece_move_white,
            is_sliding_piece_move_black,
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
            is_sliding_piece_move_white,
            is_sliding_piece_move_black,
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
        let is_pawn_push_white = |mv: Move, pos: &Position<1, { Color::White }>| {
            pos.board.piece_mask::<{ Piece::Pawn }>()
                & pos.board.color_mask_at(Color::White)
                & mv.source().mask()
                != 0
                && (mv.source() as i8 - mv.destination() as i8) % 8 == 0
        };
        let is_pawn_push_black = |mv: Move, pos: &Position<1, { Color::Black }>| {
            pos.board.piece_mask::<{ Piece::Pawn }>()
                & pos.board.color_mask_at(Color::Black)
                & mv.source().mask()
                != 0
                && (mv.source() as i8 - mv.destination() as i8) % 8 == 0
        };

        expected_moves_test(
            "2bb3k/P1Ppqp1P/bn2pnp1/3Pr3/1p5b/2NQ3p/PPPPPPPP/R3K2R w KQ - 0 1",
            is_pawn_push_white,
            is_pawn_push_black,
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
        let is_non_ep_pawn_capture_white = |mv: Move, pos: &Position<1, { Color::White }>| {
            pos.board.piece_mask::<{ Piece::Pawn }>()
                & pos.board.color_mask_at(Color::White)
                & mv.source().mask()
                != 0
                && mv.flag() != MoveFlag::EnPassant
                && (mv.source() as i8 - mv.destination() as i8) % 8 != 0
        };
        let is_non_ep_pawn_capture_black = |mv: Move, pos: &Position<1, { Color::Black }>| {
            pos.board.piece_mask::<{ Piece::Pawn }>()
                & pos.board.color_mask_at(Color::Black)
                & mv.source().mask()
                != 0
                && mv.flag() != MoveFlag::EnPassant
                && (mv.source() as i8 - mv.destination() as i8) % 8 != 0
        };

        expected_moves_test(
            "1qbb3k/P1PpqP1P/bn2pnp1/3Pr3/1p5b/1nNQ3p/PPPPPPPP/Rqn1Kb1R w KQ - 0 1",
            is_non_ep_pawn_capture_white,
            is_non_ep_pawn_capture_black,
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
        let is_en_passant_white = |mv: Move, _: &Position<1, { Color::White }>| {
            mv.flag() == MoveFlag::EnPassant
        };
        let is_en_passant_black = |mv: Move, _: &Position<1, { Color::Black }>| {
            mv.flag() == MoveFlag::EnPassant
        };

        expected_moves_test(
            "8/2p5/3p4/KP5r/1R2Pp1k/8/6P1/8 b - e3 0 1",
            is_en_passant_white,
            is_en_passant_black,
            [],
        );

        expected_moves_test(
            "8/8/3p4/KPp4r/1R3p1k/8/4P1P1/8 w - c6 0 2",
            is_en_passant_white,
            is_en_passant_black,
            [],
        );

        expected_moves_test(
            "8/8/3p4/KPpP3r/1R3p1k/8/4P1P1/8 w - c6 0 2",
            is_en_passant_white,
            is_en_passant_black,
            [
                Move::new_non_promotion(Square::D5, Square::C6, MoveFlag::EnPassant),
                Move::new_non_promotion(Square::B5, Square::C6, MoveFlag::EnPassant),
            ],
        );

        expected_moves_test(
            "8/B7/3p4/kPpP3r/3K1p2/8/4P1P1/8 w - c6 0 2",
            is_en_passant_white,
            is_en_passant_black,
            [
                Move::new_non_promotion(Square::D5, Square::C6, MoveFlag::EnPassant),
                Move::new_non_promotion(Square::B5, Square::C6, MoveFlag::EnPassant),
            ],
        );

        expected_moves_test(
            "8/8/b2p4/kPpP3r/2K2p2/8/4P1P1/8 w - c6 0 2",
            is_en_passant_white,
            is_en_passant_black,
            [Move::new_non_promotion(
                Square::D5,
                Square::C6,
                MoveFlag::EnPassant,
            )],
        );
    }

    #[test]
    fn test_king_movegen() {
        let is_king_move_white = |mv: Move, pos: &Position<1, { Color::White }>| {
            mv.flag() == MoveFlag::NormalMove
                && pos.board.piece_mask::<{ Piece::King }>()
                    & pos.board.color_mask_at(Color::White)
                    & mv.source().mask()
                    != 0
        };
        let is_king_move_black = |mv: Move, pos: &Position<1, { Color::Black }>| {
            mv.flag() == MoveFlag::NormalMove
                && pos.board.piece_mask::<{ Piece::King }>()
                    & pos.board.color_mask_at(Color::Black)
                    & mv.source().mask()
                    != 0
        };

        expected_moves_test(
            "3N3B/5k1P/R4b2/8/8/3K4/8/8 b - - 0 1",
            is_king_move_white,
            is_king_move_black,
            [
                Move::new_non_promotion(Square::F7, Square::G6, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::F7, Square::F8, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::F7, Square::E8, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::F7, Square::E7, MoveFlag::NormalMove),
            ],
        );

        expected_moves_test(
            "5R1B/5k1P/R4b2/8/8/3K4/8/8 b - - 0 1",
            is_king_move_white,
            is_king_move_black,
            [
                Move::new_non_promotion(Square::F7, Square::G6, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::F7, Square::F8, MoveFlag::NormalMove),
                Move::new_non_promotion(Square::F7, Square::E7, MoveFlag::NormalMove),
            ],
        );
    }

    #[test]
    fn test_white_castling_movegen() {
        let is_castling_move_white = |mv: Move, _: &Position<1, { Color::White }>| {
            mv.flag() == MoveFlag::Castling
        };
        let is_castling_move_black = |mv: Move, _: &Position<1, { Color::Black }>| {
            mv.flag() == MoveFlag::Castling
        };

        expected_moves_test(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            is_castling_move_white,
            is_castling_move_black,
            [
                Move::new_non_promotion(Square::E1, Square::C1, MoveFlag::Castling),
                Move::new_non_promotion(Square::E1, Square::G1, MoveFlag::Castling),
            ],
        );

        expected_moves_test(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBB1bP/R3K2R w KQkq - 0 1",
            is_castling_move_white,
            is_castling_move_black,
            [Move::new_non_promotion(
                Square::E1,
                Square::C1,
                MoveFlag::Castling,
            )],
        );

        expected_moves_test(
            "4k3/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2b2Q1p/PrPBB1rP/R3K2R w KQ - 0 1",
            is_castling_move_white,
            is_castling_move_black,
            [Move::new_non_promotion(
                Square::E1,
                Square::C1,
                MoveFlag::Castling,
            )],
        );

        expected_moves_test(
            "4k3/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2b2Q1p/PrrBB1RP/R3K2R w KQ - 0 1",
            is_castling_move_white,
            is_castling_move_black,
            [Move::new_non_promotion(
                Square::E1,
                Square::G1,
                MoveFlag::Castling,
            )],
        );

        expected_moves_test(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBB2P/RN2K1nR w KQkq - 0 1",
            is_castling_move_white,
            is_castling_move_black,
            [],
        );
    }
}
