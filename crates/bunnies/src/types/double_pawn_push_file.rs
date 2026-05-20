//! Sentinel-encoded file for the side that just double-pushed a pawn (`-1` = none, `0..=7` = `File` index).

use super::{
    bitboard::Bitboard,
    board::Board,
    color::Color,
    file::File,
    piece::Piece,
    rank::Rank,
    square::Square,
};

/// FEN / context encoding: `-1` means no en-passant target; otherwise the [`File`] index `0..=7`.
pub type DoublePawnPushFile = i8;

mod private {
    pub trait Sealed {}
    impl Sealed for super::DoublePawnPushFile {}
}

/// Const-safe en-passant file encoding and square geometry.
pub const trait ConstDoublePawnPushFile: private::Sealed {
    /// No en-passant capture is available next move.
    const NONE: DoublePawnPushFile;

    /// After a pawn step, the EP file to store in the new context (`NONE` unless a double push).
    fn from_pawn_step(from: Square, to: Square) -> DoublePawnPushFile;

    /// `true` when `self` holds a file index `0..=7`.
    fn has_file(self) -> bool;

    fn from_file(file: Option<File>) -> DoublePawnPushFile;

    /// The file when [`Self::has_file`] returns `true`, otherwise `None`.
    fn file(self) -> Option<File>;

    /// Bitboard of squares from which the side to move might capture en passant on `self`'s file.
    fn ep_possible_src_mask(self, stm: Color) -> Bitboard;

    /// Destination square of an en-passant capture for the side to move.
    fn ep_dst_square(self, stm: Color) -> Square;

    /// Square of the pawn that was skipped over (remove this pawn on EP capture).
    fn ep_capture_square(self, stm: Color) -> Square;
}

/// [`ConstDoublePawnPushFile`] plus validation that needs a [`Board`] read.
pub trait DoublePawnPushFileUtils: ConstDoublePawnPushFile + private::Sealed {
    /// Whether this value is consistent with pawn placement (used by FEN / position validation).
    fn ep_target_is_valid(self, halfmove: u16, side_to_move: Color, board: &Board) -> bool;
}

impl const ConstDoublePawnPushFile for DoublePawnPushFile {
    const NONE: DoublePawnPushFile = -1;

    fn from_pawn_step(from: Square, to: Square) -> DoublePawnPushFile {
        if is_double_pawn_step(from, to) {
            from.file() as DoublePawnPushFile
        } else {
            Self::NONE
        }
    }

    fn has_file(self) -> bool {
        self >= 0 && self < 8
    }

    fn from_file(file: Option<File>) -> DoublePawnPushFile {
        match file {
            None => -1,
            Some(file) => file as u8 as i8,
        }
    }

    fn file(self) -> Option<File> {
        if self.has_file() {
            Some({
                let value = self as u8;
                unsafe { File::try_from(value).unwrap_unchecked() }
            })
        } else {
            None
        }
    }

    fn ep_possible_src_mask(self, stm: Color) -> Bitboard {
        debug_assert!(self.has_file());
        let f = {
            let value = self as u8;
            unsafe { File::try_from(value).unwrap_unchecked() }
        };
        let double_pawn_push_dst = match stm {
            Color::White => Square::from_rank_and_file(Rank::Five, f).mask(),
            Color::Black => Square::from_rank_and_file(Rank::Four, f).mask(),
        };

        ((double_pawn_push_dst << 1) & !File::H.mask())
            | ((double_pawn_push_dst >> 1) & !File::A.mask())
    }

    fn ep_dst_square(self, stm: Color) -> Square {
        debug_assert!(self.has_file());
        let f = {
            let value = self as u8;
            unsafe { File::try_from(value).unwrap_unchecked() }
        };
        match stm {
            Color::White => Square::from_rank_and_file(Rank::Six, f),
            Color::Black => Square::from_rank_and_file(Rank::Three, f),
        }
    }

    fn ep_capture_square(self, stm: Color) -> Square {
        debug_assert!(self.has_file());
        let f = {
            let value = self as u8;
            unsafe { File::try_from(value).unwrap_unchecked() }
        };
        match stm {
            Color::White => Square::from_rank_and_file(Rank::Five, f),
            Color::Black => Square::from_rank_and_file(Rank::Four, f),
        }
    }
}

impl DoublePawnPushFileUtils for DoublePawnPushFile {
    fn ep_target_is_valid(self, halfmove: u16, side_to_move: Color, board: &Board) -> bool {
        if !self.has_file() {
            return true;
        }
        if halfmove < 1 {
            return false;
        }
        let color_just_moved = side_to_move.other();
        let pawns_mask = board.piece_mask::<{ Piece::Pawn }>();
        let colored_pawns_mask = pawns_mask & board.color_mask_at(color_just_moved);
        debug_assert!(self.has_file());
        let file_mask = unsafe { File::try_from(self as u8).unwrap_unchecked() }.mask();
        let rank_mask = match color_just_moved {
            Color::White => Rank::Four.mask(),
            Color::Black => Rank::Five.mask(),
        };
        colored_pawns_mask & file_mask & rank_mask != 0
    }
}

const fn is_double_pawn_step(from: Square, to: Square) -> bool {
    let from_mask = from.mask();
    let to_mask = to.mask();

    to_mask & (from_mask << 16) != 0 || to_mask & (from_mask >> 16) != 0
}
