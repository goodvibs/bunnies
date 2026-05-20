//! Chess piece types (including [`Piece::Null`] for empty squares).

use crate::utilities::{Array, IterableEnum, impl_u8_conversions};

/// Chess piece type without color (used with [`Color`](crate::types::Color) to form [`ColoredPiece`](crate::types::ColoredPiece)).
///
/// Values are chosen to allow efficient bit packing:
/// - `Null = 0` for empty squares
/// - `Pawn = 1` through `King = 6` for actual pieces
///
/// Can be used as a const generic parameter to specialize algorithms by piece type.
#[repr(u8)]
#[derive(Clone, Copy, Eq, Debug, std::marker::ConstParamTy)]
#[derive_const(PartialEq)]
pub enum Piece {
    /// Empty square placeholder (value 0).
    Null = 0,
    /// Pawn (value 1).
    Pawn = 1,
    /// Knight (value 2).
    Knight = 2,
    /// Bishop (value 3).
    Bishop = 3,
    /// Rook (value 4).
    Rook = 4,
    /// Queen (value 5).
    Queen = 5,
    /// King (value 6).
    King = 6,
}

impl Piece {
    /// One past the maximum valid piece value (for bounds checking).
    pub const LIMIT: u8 = 7;
    /// Alias for `Null` representing all pieces when used as a mask selector.
    pub const ALL_PIECES: Piece = Piece::Null;

    /// Creates a `Piece` from its numeric discriminant.
    ///
    /// # Safety
    /// `piece_int` must be less than [`Piece::LIMIT`]. Violating this is undefined behavior.
    pub const unsafe fn from(piece_int: u8) -> Piece {
        debug_assert!(piece_int < Piece::LIMIT, "Piece type number out of bounds");
        unsafe { std::mem::transmute::<u8, Piece>(piece_int) }
    }

    /// Returns `true` for bishops, rooks, and queens (sliding attackers).
    pub const fn is_sliding_piece(&self) -> bool {
        matches!(*self, Piece::Bishop | Piece::Rook | Piece::Queen)
    }

    /// Parses a piece from an uppercase ASCII character (PNBRQK or any other returns `Null`).
    pub const fn from_uppercase_char(piece_char: char) -> Piece {
        match piece_char {
            'P' => Piece::Pawn,
            'N' => Piece::Knight,
            'B' => Piece::Bishop,
            'R' => Piece::Rook,
            'Q' => Piece::Queen,
            'K' => Piece::King,
            _ => Piece::Null,
        }
    }

    /// Returns the Piece from the given lowercase char.
    pub const fn from_lowercase_char(piece_char: char) -> Piece {
        match piece_char {
            'p' => Piece::Pawn,
            'n' => Piece::Knight,
            'b' => Piece::Bishop,
            'r' => Piece::Rook,
            'q' => Piece::Queen,
            'k' => Piece::King,
            _ => Piece::Null,
        }
    }

    /// Returns the uppercase ASCII character corresponding to the Piece.
    pub const fn uppercase_ascii(&self) -> char {
        match self {
            Piece::Null => ' ',
            Piece::Pawn => 'P',
            Piece::Knight => 'N',
            Piece::Bishop => 'B',
            Piece::Rook => 'R',
            Piece::Queen => 'Q',
            Piece::King => 'K',
        }
    }

    /// Returns the lowercase ASCII character corresponding to the Piece.
    pub const fn lowercase_ascii(&self) -> char {
        match self {
            Piece::Null => ' ',
            Piece::Pawn => 'p',
            Piece::Knight => 'n',
            Piece::Bishop => 'b',
            Piece::Rook => 'r',
            Piece::Queen => 'q',
            Piece::King => 'k',
        }
    }

    /// Returns the unfilled Unicode character corresponding to the Piece.
    pub const fn unfilled_unicode(&self) -> char {
        match self {
            Piece::Null => ' ',
            Piece::Pawn => '♙',
            Piece::Knight => '♘',
            Piece::Bishop => '♗',
            Piece::Rook => '♖',
            Piece::Queen => '♕',
            Piece::King => '♔',
        }
    }

    /// Returns the filled Unicode character corresponding to the Piece.
    pub const fn filled_unicode(&self) -> char {
        match self {
            Piece::Null => ' ',
            Piece::Pawn => '♟',
            Piece::Knight => '♞',
            Piece::Bishop => '♝',
            Piece::Rook => '♜',
            Piece::Queen => '♛',
            Piece::King => '♚',
        }
    }

    /// All actual piece types (excludes `Null`).
    pub const PIECES: Array<Piece, 6> = Array([
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::King,
    ]);

    /// All piece types except King (for under-promotion targets and piece drops).
    pub const NON_KING_PIECES: Array<Piece, 5> = Array([
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
    ]);

    /// Valid promotion targets (Knight through Queen; excludes King and Pawn).
    pub const PROMOTION_PIECES: Array<Piece, 4> =
        Array([Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen]);

    /// Sliding attackers (bishop, rook, queen) for attack generation.
    pub const SLIDING_PIECES: Array<Piece, 3> = Array([Piece::Bishop, Piece::Rook, Piece::Queen]);
}

impl const IterableEnum<7> for Piece {
    const ALL: Array<Piece, 7> = Array([
        Piece::Null,
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::King,
    ]);
}

impl_u8_conversions!(Piece, 7);
