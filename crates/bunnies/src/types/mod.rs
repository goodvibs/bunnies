mod bitboard;
mod board;
mod castling_rights;
mod color;
mod colored_piece;
mod double_pawn_push_file;
mod file;
mod flank;
mod knight_move_direction;
mod r#move;
mod move_flag;
mod move_list;
mod piece;
mod position;
mod position_context;
mod queen_like_move_direction;
mod rank;
mod square;
mod square_delta;
mod typed_position;
mod unified_move_direction;
mod with_zobrist;
mod without_zobrist;
mod zobrist_policy;

pub use bitboard::*;
pub use board::*;
pub use castling_rights::*;
pub use color::*;
pub use colored_piece::*;
pub use double_pawn_push_file::*;
pub use file::*;
pub use flank::*;
pub use knight_move_direction::*;
pub use r#move::*;
pub use move_flag::*;
pub use move_list::*;
pub use piece::*;
pub use position::*;
pub use position_context::*;
pub use queen_like_move_direction::*;
pub use rank::*;
pub use square::*;
pub use square_delta::*;
pub use typed_position::*;
pub use unified_move_direction::*;
pub use with_zobrist::*;
pub use without_zobrist::*;
pub use zobrist_policy::*;

use crate::utilities::Array;

/// Static lookup table for move directions between any two squares.
/// This is used by QueenLikeMoveDirection, KnightMoveDirection, and UnifiedMoveDirection.
static MOVE_DIRECTION_LOOKUP: Array<Array<UnifiedMoveDirection, 64>, 64> = {
    use crate::types::{KnightMoveDirection, QueenLikeMoveDirection, Square, same_line};

    const fn unified_move_direction_at(
        src_square: Square,
        dst_square: Square,
    ) -> UnifiedMoveDirection {
        if same_line(src_square, dst_square) {
            let mut _d = 0;
            let direction = QueenLikeMoveDirection::calc(src_square, dst_square, &mut _d);
            UnifiedMoveDirection::from_queen_like(direction)
        } else {
            match KnightMoveDirection::calc(src_square, dst_square) {
                Some(direction) => UnifiedMoveDirection::from_knight_like(direction),
                None => UnifiedMoveDirection::NULL,
            }
        }
    }

    let mut arr = [UnifiedMoveDirection::NULL; 64 * 64];
    let mut i = 0usize;
    while i < 64 * 64 {
        let src_square = unsafe { Square::try_from((i / 64) as u8).unwrap_unchecked() };
        let dst_square = unsafe { Square::try_from((i % 64) as u8).unwrap_unchecked() };
        arr[i] = unified_move_direction_at(src_square, dst_square);
        i += 1;
    }
    unsafe { std::mem::transmute(arr) }
};
