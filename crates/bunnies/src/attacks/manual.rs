//! Contains functions that manually calculate attacks for all pieces

use crate::Bitboard;
use crate::Color;
use crate::File;
use crate::Square;
use std::cmp;

/// Returns a bitboard with all squares attacked by knights indicated by the bits in `knights_mask`
pub const fn multi_knight_attacks(knights_mask: Bitboard) -> Bitboard {
    (knights_mask << 17 & !File::H.mask())
        | (knights_mask << 15 & !File::A.mask())
        | (knights_mask << 10 & !(File::G.mask() | File::H.mask()))
        | (knights_mask << 6 & !(File::A.mask() | File::B.mask()))
        | (knights_mask >> 17 & !File::A.mask())
        | (knights_mask >> 15 & !File::H.mask())
        | (knights_mask >> 10 & !(File::A.mask() | File::B.mask()))
        | (knights_mask >> 6 & !(File::G.mask() | File::H.mask()))
}

/// Returns a bitboard with all squares attacked by kings indicated by the bits in `kings_mask`
pub const fn multi_king_attacks(kings_mask: Bitboard) -> Bitboard {
    (kings_mask << 9 & !File::H.mask())
        | (kings_mask << 8)
        | (kings_mask << 7 & !File::A.mask())
        | (kings_mask >> 9 & !File::A.mask())
        | (kings_mask >> 8)
        | (kings_mask >> 7 & !File::H.mask())
        | (kings_mask << 1 & !File::H.mask())
        | (kings_mask >> 1 & !File::A.mask())
}

pub const fn multi_pawn_attacks_left(pawns_mask: Bitboard, by_color: Color) -> Bitboard {
    match by_color {
        Color::White => pawns_mask << 9 & !File::H.mask(),
        Color::Black => pawns_mask >> 9 & !File::A.mask(),
    }
}

pub const fn multi_pawn_attacks_right(pawns_mask: Bitboard, by_color: Color) -> Bitboard {
    match by_color {
        Color::White => pawns_mask << 7 & !File::A.mask(),
        Color::Black => pawns_mask >> 7 & !File::H.mask(),
    }
}

/// Returns a bitboard with all squares attacked by pawns indicated by the bits in `pawns_mask`
pub const fn multi_pawn_attacks(pawns_mask: Bitboard, by_color: Color) -> Bitboard {
    match by_color {
        Color::White => (pawns_mask << 9 & !File::H.mask()) | (pawns_mask << 7 & !File::A.mask()),
        Color::Black => (pawns_mask >> 7 & !File::H.mask()) | (pawns_mask >> 9 & !File::A.mask()),
    }
}

/// Returns a bitboard with all squares that pawns indicated by the bits in `pawns_mask` can move to
pub const fn multi_pawn_moves(pawns_mask: Bitboard, by_color: Color) -> Bitboard {
    match by_color {
        Color::White => pawns_mask << 8,
        Color::Black => pawns_mask >> 8,
    }
}

/// Returns a bitboard with all squares attacked by a rook on `src_square`
/// with `occupied_mask` as the mask of occupied squares
pub const fn manual_single_rook_attacks(src_square: Square, occupied_mask: Bitboard) -> Bitboard {
    let src_square_mask = src_square.mask();
    let mut result: Bitboard = 0;

    let mut mask = src_square_mask << 1;
    while mask != 0 && mask & File::H.mask() == 0 {
        result |= mask;
        if occupied_mask & mask != 0 {
            break;
        }
        mask <<= 1;
    }

    let mut mask = src_square_mask << 8;
    while mask != 0 {
        result |= mask;
        if occupied_mask & mask != 0 {
            break;
        }
        mask <<= 8;
    }

    let mut mask = src_square_mask >> 1;
    while mask != 0 && mask & File::A.mask() == 0 {
        result |= mask;
        if occupied_mask & mask != 0 {
            break;
        }
        mask >>= 1;
    }

    let mut mask = src_square_mask >> 8;
    while mask != 0 {
        result |= mask;
        if occupied_mask & mask != 0 {
            break;
        }
        mask >>= 8;
    }

    result
}

/// Returns a bitboard with all squares attacked by a bishop on `src_square`
/// with `occupied_mask` as the mask of occupied squares
pub fn manual_single_bishop_attacks(src_square: Square, occupied_mask: Bitboard) -> Bitboard {
    let mut attacks: Bitboard = 0;
    let leading_zeros = src_square as u32;
    let n_distance: u32 = leading_zeros / 8;
    let s_distance: u32 = 7 - n_distance;
    let w_distance: u32 = leading_zeros % 8;
    let e_distance: u32 = 7 - w_distance;
    let src_mask = src_square.mask();
    let (mut pos_nw, mut pos_ne, mut pos_sw, mut pos_se): (Bitboard, Bitboard, Bitboard, Bitboard) =
        (src_mask, src_mask, src_mask, src_mask);
    for _ in 0..cmp::min(n_distance, w_distance) {
        pos_nw <<= 9;
        attacks |= pos_nw;
        if occupied_mask & pos_nw != 0 {
            break;
        }
    }
    for _ in 0..cmp::min(n_distance, e_distance) {
        pos_ne <<= 7;
        attacks |= pos_ne;
        if occupied_mask & pos_ne != 0 {
            break;
        }
    }
    for _ in 0..cmp::min(s_distance, w_distance) {
        pos_sw >>= 7;
        attacks |= pos_sw;
        if occupied_mask & pos_sw != 0 {
            break;
        }
    }
    for _ in 0..cmp::min(s_distance, e_distance) {
        pos_se >>= 9;
        attacks |= pos_se;
        if occupied_mask & pos_se != 0 {
            break;
        }
    }
    attacks
}
