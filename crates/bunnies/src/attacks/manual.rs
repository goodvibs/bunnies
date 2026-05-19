//! Contains functions that manually calculate attacks for all pieces

use crate::types::{Bitboard, Color, File, Piece, Square};
use std::cmp;

#[inline]
const fn walk_ray_limited(
    mut pos: Bitboard,
    occupied_mask: Bitboard,
    shift: u8,
    shift_left: bool,
    max_steps: u32,
) -> Bitboard {
    let mut attacks: Bitboard = 0;
    let mut i = 0;
    while i < max_steps {
        pos = if shift_left {
            pos << shift
        } else {
            pos >> shift
        };
        if pos == 0 {
            break;
        }
        attacks |= pos;
        if occupied_mask & pos != 0 {
            break;
        }
        i += 1;
    }
    attacks
}

#[inline]
const fn square_edge_distances(square: Square) -> (u32, u32, u32, u32) {
    let idx = square as u32;
    let n_distance = idx / 8;
    let s_distance = 7 - n_distance;
    let w_distance = idx % 8;
    let e_distance = 7 - w_distance;
    (n_distance, s_distance, w_distance, e_distance)
}

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
    multi_pawn_attacks_left(pawns_mask, by_color) | multi_pawn_attacks_right(pawns_mask, by_color)
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
    let (n_distance, s_distance, w_distance, e_distance) = square_edge_distances(src_square);
    walk_ray_limited(src_square_mask, occupied_mask, 1, true, w_distance)
        | walk_ray_limited(src_square_mask, occupied_mask, 8, true, n_distance)
        | walk_ray_limited(src_square_mask, occupied_mask, 1, false, e_distance)
        | walk_ray_limited(src_square_mask, occupied_mask, 8, false, s_distance)
}

/// Returns a bitboard with all squares attacked by a bishop on `src_square`
/// with `occupied_mask` as the mask of occupied squares
pub const fn manual_single_bishop_attacks(src_square: Square, occupied_mask: Bitboard) -> Bitboard {
    let (n_distance, s_distance, w_distance, e_distance) = square_edge_distances(src_square);
    let src_mask = src_square.mask();
    walk_ray_limited(
        src_mask,
        occupied_mask,
        9,
        true,
        cmp::min(n_distance, w_distance),
    ) | walk_ray_limited(
        src_mask,
        occupied_mask,
        7,
        true,
        cmp::min(n_distance, e_distance),
    ) | walk_ray_limited(
        src_mask,
        occupied_mask,
        7,
        false,
        cmp::min(s_distance, w_distance),
    ) | walk_ray_limited(
        src_mask,
        occupied_mask,
        9,
        false,
        cmp::min(s_distance, e_distance),
    )
}

pub const fn manual_sliding_piece_attacks<const P: Piece>(
    from: Square,
    occupied_mask: Bitboard,
) -> Bitboard {
    match P {
        Piece::Bishop => manual_single_bishop_attacks(from, occupied_mask),
        Piece::Rook => manual_single_rook_attacks(from, occupied_mask),
        Piece::Queen => {
            manual_single_bishop_attacks(from, occupied_mask)
                | manual_single_rook_attacks(from, occupied_mask)
        }
        _ => panic!("P is not a sliding piece"),
    }
}
