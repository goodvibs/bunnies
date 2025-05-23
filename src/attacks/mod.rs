//! This module contains functions to calculate attack bitboards for different pieces.
//! The direct exports are the recommended way to calculate attack masks.
//! However, the `precomputed`, `manual`, and `magic` submodules may also be used.

use crate::Color;
use crate::Square;
use crate::{Bitboard, Piece};

pub mod magic;
pub mod manual;
pub mod precomputed;

/// Returns an attack mask encoding all squares attacked by a knight on `src_square`
pub fn single_knight_attacks(src_square: Square) -> Bitboard {
    precomputed::precomputed_single_knight_attacks(src_square)
}

/// Returns an attack mask encoding all squares attacked by a king on `src_square`
pub fn single_king_attacks(src_square: Square) -> Bitboard {
    precomputed::precomputed_single_king_attacks(src_square)
}

/// Returns an attack mask encoding all squares attacked by knight(s) on `knights_mask`
pub fn multi_knight_attacks(knights_mask: Bitboard) -> Bitboard {
    manual::multi_knight_attacks(knights_mask)
}

/// Returns an attack mask encoding all squares attacked by king(s) on `kings_mask`
pub fn multi_king_attacks(kings_mask: Bitboard) -> Bitboard {
    manual::multi_king_attacks(kings_mask)
}

/// Returns an attack mask encoding all squares attacked by pawn(s) on `pawns_mask`
pub fn multi_pawn_attacks(pawns_mask: Bitboard, by_color: Color) -> Bitboard {
    manual::multi_pawn_attacks(pawns_mask, by_color)
}

/// Returns a mask encoding all squares that pawn(s) on `pawns_mask` can move to
pub fn multi_pawn_moves(pawns_mask: Bitboard, by_color: Color) -> Bitboard {
    manual::multi_pawn_moves(pawns_mask, by_color)
}

/// Returns an attack mask encoding all squares attacked by a bishop on `src_square`,
/// with `occupied_mask` as the mask of occupied squares
pub fn single_bishop_attacks(src_square: Square, occupied_mask: Bitboard) -> Bitboard {
    magic::magic_single_bishop_attacks(src_square, occupied_mask)
}

/// Returns an attack mask encoding all squares attacked by a rook on `src_square`,
/// with `occupied_mask` as the mask of occupied squares
pub fn single_rook_attacks(src_square: Square, occupied_mask: Bitboard) -> Bitboard {
    magic::magic_single_rook_attacks(src_square, occupied_mask)
}

/// Returns an attack mask encoding all squares attacked by a queen on `src_square`,
/// with `occupied_mask` as the mask of occupied squares
pub fn single_queen_attacks(src_square: Square, occupied_mask: Bitboard) -> Bitboard {
    magic::magic_single_bishop_attacks(src_square, occupied_mask)
        | magic::magic_single_rook_attacks(src_square, occupied_mask)
}

/// Returns an attack mask encoding all squares attacked by `piece` on `src_square`,
/// with `occupied_mask` as the mask of occupied squares
pub fn sliding_piece_attacks(
    src_square: Square,
    occupied_mask: Bitboard,
    piece: Piece,
) -> Bitboard {
    match piece {
        Piece::Bishop => single_bishop_attacks(src_square, occupied_mask),
        Piece::Rook => single_rook_attacks(src_square, occupied_mask),
        Piece::Queen => single_queen_attacks(src_square, occupied_mask),
        _ => panic!("Not a sliding piece!"),
    }
}
