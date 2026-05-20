//! Zobrist random tables and hash-key helpers.

use crate::{
    types::{Board, CastlingRights, Color, DoublePawnPushFile, Piece, Square},
    utilities::{Array, IterableEnum, Prng},
};

const RNG_SEED: u64 = 161803398875;

const NUM_PIECE_SQUARE_KEYS: usize = 64 * 12;
const NUM_CASTLING_RIGHTS_KEYS: usize = 16;
const NUM_DOUBLE_PAWN_PUSH_FILE_KEYS: usize = 8;
const NUM_SIDE_TO_MOVE_KEYS: usize = 1;

const PIECE_SQUARE_KEYS_START: usize = 0;
const CASTLING_RIGHTS_KEYS_START: usize = PIECE_SQUARE_KEYS_START + NUM_PIECE_SQUARE_KEYS;
const DOUBLE_PAWN_PUSH_KEYS_START: usize = CASTLING_RIGHTS_KEYS_START + NUM_CASTLING_RIGHTS_KEYS;
const SIDE_TO_MOVE_KEYS_START: usize = DOUBLE_PAWN_PUSH_KEYS_START + NUM_DOUBLE_PAWN_PUSH_FILE_KEYS;

const NUM_RANDOMS: usize = NUM_PIECE_SQUARE_KEYS
    + NUM_CASTLING_RIGHTS_KEYS
    + NUM_DOUBLE_PAWN_PUSH_FILE_KEYS
    + NUM_SIDE_TO_MOVE_KEYS;

const _: () = assert!(NUM_RANDOMS == SIDE_TO_MOVE_KEYS_START + NUM_SIDE_TO_MOVE_KEYS);

static RANDOMS: Array<u64, { NUM_RANDOMS }> = {
    let mut rng = Prng::new(RNG_SEED);
    let mut randoms = Array([0; NUM_RANDOMS]);
    let mut i = 0;
    while i < NUM_RANDOMS {
        randoms[i] = rng.generate();
        i += 1;
    }
    randoms
};

const fn copy_keys<const N: usize, const START: usize>() -> [u64; N] {
    let mut res = [0; N];
    res.copy_from_slice(&RANDOMS[START..START + N]);
    res
}

const fn fill(slice: &mut [u64], val: u64) {
    let mut i = 0;
    while i < slice.len() {
        slice[i] = val;
        i += 1;
    }
}

static PIECE_SQUARE_KEYS: Array<Array<u64, 64>, 12> = {
    let keys = copy_keys::<{ NUM_PIECE_SQUARE_KEYS }, { PIECE_SQUARE_KEYS_START }>();
    let mut keys_lookup: Array<Array<u64, 64>, 12> = unsafe { std::mem::transmute(keys) };

    fill(
        &mut keys_lookup[Piece::Pawn as usize][Square::A8 as usize..=Square::H8 as usize],
        0,
    );
    fill(
        &mut keys_lookup[Piece::Pawn as usize][Square::A1 as usize..=Square::H1 as usize],
        0,
    );

    keys_lookup
};

static CASTLING_RIGHTS_KEYS: Array<u64, { NUM_CASTLING_RIGHTS_KEYS }> = {
    let mut keys = copy_keys::<{ NUM_CASTLING_RIGHTS_KEYS }, { CASTLING_RIGHTS_KEYS_START }>();
    keys[CastlingRights::B1111 as usize] = 0;
    unsafe { std::mem::transmute(keys) }
};

static DOUBLE_PAWN_PUSH_FILE_KEYS: Array<u64, { NUM_DOUBLE_PAWN_PUSH_FILE_KEYS }> = {
    let keys = copy_keys::<{ NUM_DOUBLE_PAWN_PUSH_FILE_KEYS }, { DOUBLE_PAWN_PUSH_KEYS_START }>();
    unsafe { std::mem::transmute(keys) }
};

static BLACK_SIDE_TO_MOVE_KEY: u64 = RANDOMS[SIDE_TO_MOVE_KEYS_START];

/// Returns piece-square key contribution for (`piece`, `square`).
pub const fn piece_square_key(piece: Piece, square: Square) -> u64 {
    PIECE_SQUARE_KEYS[piece as usize][square as usize]
}

/// Returns castling-rights key contribution.
pub const fn castling_rights_key(castling_rights: CastlingRights) -> u64 {
    CASTLING_RIGHTS_KEYS[castling_rights as usize]
}

/// Returns en-passant-file key contribution (`0` when no file is available).
pub const fn double_pawn_push_key(double_pawn_push_file: DoublePawnPushFile) -> u64 {
    if double_pawn_push_file < 0 {
        0
    } else {
        DOUBLE_PAWN_PUSH_FILE_KEYS[double_pawn_push_file as usize]
    }
}

/// Returns side-to-move key contribution.
pub const fn side_to_move_key(side_to_move: Color) -> u64 {
    match side_to_move {
        Color::White => 0,
        Color::Black => BLACK_SIDE_TO_MOVE_KEY,
    }
}

impl Board {
    /// Computes piece-placement Zobrist hash for this board only.
    pub const fn calc_zobrist_hash(&self) -> u64 {
        let mut hash = 0;
        for square in Square::ALL {
            let piece = self.piece_at(square);
            hash ^= piece_square_key(piece, square);
        }

        hash
    }
}

/// Computes full position hash from board, castling rights, en-passant file, and side to move.
pub const fn calc_position_zobrist_hash(
    board: &Board,
    castling_rights: CastlingRights,
    double_pawn_push_file: DoublePawnPushFile,
    side_to_move: Color,
) -> u64 {
    board.calc_zobrist_hash()
        ^ castling_rights_key(castling_rights)
        ^ double_pawn_push_key(double_pawn_push_file)
        ^ side_to_move_key(side_to_move)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::utilities::IterableEnum;

    fn expected_randoms() -> [u64; NUM_RANDOMS] {
        let mut rng = Prng::new(RNG_SEED);
        let mut randoms = [0u64; NUM_RANDOMS];

        let mut i = 0;
        while i < NUM_RANDOMS {
            randoms[i] = rng.generate();
            i += 1;
        }

        randoms
    }

    fn expected_piece_square_key(piece: Piece, square: Square) -> u64 {
        if piece == Piece::Pawn
            && ((Square::A1 as usize..=Square::H1 as usize).contains(&(square as usize))
                || (Square::A8 as usize..=Square::H8 as usize).contains(&(square as usize)))
        {
            0
        } else {
            RANDOMS[PIECE_SQUARE_KEYS_START + piece as usize * 64 + square as usize]
        }
    }

    #[test]
    fn every_random_number_is_unique() {
        let mut seen = HashSet::with_capacity(NUM_RANDOMS);

        for (i, random) in RANDOMS.iter().copied().enumerate() {
            assert!(
                seen.insert(random),
                "duplicate random number at RANDOMS[{i}]: {random:#018x}"
            );
        }

        assert_eq!(seen.len(), NUM_RANDOMS);
    }

    #[test]
    fn randoms_are_generated_deterministically_from_seed() {
        let expected = expected_randoms();

        for i in 0..NUM_RANDOMS {
            assert_eq!(
                RANDOMS[i], expected[i],
                "RANDOMS[{i}] did not match deterministic PRNG output"
            );
        }
    }

    #[test]
    fn random_table_layout_is_correct() {
        assert_eq!(NUM_PIECE_SQUARE_KEYS, 64 * 12);
        assert_eq!(NUM_CASTLING_RIGHTS_KEYS, 16);
        assert_eq!(NUM_DOUBLE_PAWN_PUSH_FILE_KEYS, 8);
        assert_eq!(NUM_SIDE_TO_MOVE_KEYS, 1);

        assert_eq!(PIECE_SQUARE_KEYS_START, 0);
        assert_eq!(CASTLING_RIGHTS_KEYS_START, 64 * 12);
        assert_eq!(DOUBLE_PAWN_PUSH_KEYS_START, 64 * 12 + 16);
        assert_eq!(SIDE_TO_MOVE_KEYS_START, 64 * 12 + 16 + 8);
        assert_eq!(NUM_RANDOMS, 64 * 12 + 16 + 8 + 1);
    }

    #[test]
    fn pawn_keys_are_zero_on_first_and_eighth_ranks() {
        for square in [
            Square::A1,
            Square::B1,
            Square::C1,
            Square::D1,
            Square::E1,
            Square::F1,
            Square::G1,
            Square::H1,
            Square::A8,
            Square::B8,
            Square::C8,
            Square::D8,
            Square::E8,
            Square::F8,
            Square::G8,
            Square::H8,
        ] {
            assert_eq!(
                piece_square_key(Piece::Pawn, square),
                0,
                "pawn key on {square:?} should be zero"
            );
        }
    }

    #[test]
    fn piece_square_keys_match_expected_table_values() {
        for piece in Piece::ALL {
            for square in Square::ALL {
                assert_eq!(
                    piece_square_key(piece, square),
                    expected_piece_square_key(piece, square),
                    "bad piece-square key for {piece:?} on {square:?}"
                );
            }
        }
    }

    #[test]
    fn castling_rights_b1111_key_is_zero() {
        assert_eq!(castling_rights_key(CastlingRights::B1111), 0);
    }

    #[test]
    fn castling_rights_keys_match_randoms_except_b1111() {
        for castling_rights in CastlingRights::ALL {
            let expected = if castling_rights == CastlingRights::B1111 {
                0
            } else {
                RANDOMS[CASTLING_RIGHTS_KEYS_START + castling_rights as usize]
            };

            assert_eq!(
                castling_rights_key(castling_rights),
                expected,
                "bad castling-rights key for {castling_rights:?}"
            );
        }
    }

    #[test]
    fn double_pawn_push_key_is_zero_for_negative_sentinel() {
        assert_eq!(double_pawn_push_key(-1), 0);
    }

    #[test]
    fn double_pawn_push_keys_match_randoms_for_valid_files() {
        for file in 0..8 {
            let file = file as DoublePawnPushFile;

            assert_eq!(
                double_pawn_push_key(file),
                RANDOMS[DOUBLE_PAWN_PUSH_KEYS_START + file as usize],
                "bad double-pawn-push key for file {file}"
            );
        }
    }

    #[test]
    fn side_to_move_key_is_zero_for_white_and_random_for_black() {
        assert_eq!(side_to_move_key(Color::White), 0);
        assert_eq!(
            side_to_move_key(Color::Black),
            RANDOMS[SIDE_TO_MOVE_KEYS_START]
        );
    }

    #[test]
    fn calc_zobrist_hash_matches_manual_piece_square_xor() {
        let board = Board::initial();

        let mut expected = 0;
        for square in Square::ALL {
            expected ^= piece_square_key(board.piece_at(square), square);
        }

        assert_eq!(board.calc_zobrist_hash(), expected);
    }
}
