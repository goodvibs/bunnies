use crate::Color;
use crate::Piece;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// Represents a colored (white or black) piece on the board.
pub enum ColoredPiece {
    NoPiece = 0,
    WhitePawn = 1,
    WhiteKnight = 2,
    WhiteBishop = 3,
    WhiteRook = 4,
    WhiteQueen = 5,
    WhiteKing = 6,
    BlackPawn = 9,
    BlackKnight = 10,
    BlackBishop = 11,
    BlackRook = 12,
    BlackQueen = 13,
    BlackKing = 14,
}

impl ColoredPiece {
    /// The number of possible variants.
    pub const LIMIT: usize = 15;
    /// The numeric difference between the white and black colored pieces values.
    pub const COLOR_DIFFERENCE: u8 = 8;

    /// Returns a new ColoredPiece.
    pub const fn new(color: Color, piece_type: Piece) -> ColoredPiece {
        let piece_int = piece_type as u8;
        let is_piece = piece_int != Piece::Null as u8;
        let color_int_shifted = (is_piece as u8 & color as u8) << 3;
        unsafe { std::mem::transmute::<u8, ColoredPiece>(color_int_shifted | piece_int) }
    }

    /// Returns the color of the piece.
    pub const fn color(&self) -> Color {
        unsafe { std::mem::transmute::<u8, Color>(*self as u8 >> 3) }
    }

    /// Returns the piece type of the piece.
    pub const fn piece(&self) -> Piece {
        unsafe { std::mem::transmute::<u8, Piece>(*self as u8 & 0b111) }
    }

    /// Returns a ColoredPiece from an ASCII character.
    pub const fn from_ascii(c: char) -> ColoredPiece {
        match c {
            'P' => ColoredPiece::WhitePawn,
            'N' => ColoredPiece::WhiteKnight,
            'B' => ColoredPiece::WhiteBishop,
            'R' => ColoredPiece::WhiteRook,
            'Q' => ColoredPiece::WhiteQueen,
            'K' => ColoredPiece::WhiteKing,
            'p' => ColoredPiece::BlackPawn,
            'n' => ColoredPiece::BlackKnight,
            'b' => ColoredPiece::BlackBishop,
            'r' => ColoredPiece::BlackRook,
            'q' => ColoredPiece::BlackQueen,
            'k' => ColoredPiece::BlackKing,
            _ => ColoredPiece::NoPiece,
        }
    }

    /// Returns the ASCII character representation of the piece.
    pub const fn ascii(&self) -> char {
        match self {
            ColoredPiece::NoPiece => ' ',
            ColoredPiece::WhitePawn => 'P',
            ColoredPiece::WhiteKnight => 'N',
            ColoredPiece::WhiteBishop => 'B',
            ColoredPiece::WhiteRook => 'R',
            ColoredPiece::WhiteQueen => 'Q',
            ColoredPiece::WhiteKing => 'K',
            ColoredPiece::BlackPawn => 'p',
            ColoredPiece::BlackKnight => 'n',
            ColoredPiece::BlackBishop => 'b',
            ColoredPiece::BlackRook => 'r',
            ColoredPiece::BlackQueen => 'q',
            ColoredPiece::BlackKing => 'k',
        }
    }

    /// Returns the Unicode character representation of the piece.
    /// White pieces are unfilled, black pieces are filled.
    pub const fn unicode(&self) -> char {
        match self {
            ColoredPiece::NoPiece => ' ',
            ColoredPiece::WhitePawn => '♙',
            ColoredPiece::WhiteKnight => '♘',
            ColoredPiece::WhiteBishop => '♗',
            ColoredPiece::WhiteRook => '♖',
            ColoredPiece::WhiteQueen => '♕',
            ColoredPiece::WhiteKing => '♔',
            ColoredPiece::BlackPawn => '♟',
            ColoredPiece::BlackKnight => '♞',
            ColoredPiece::BlackBishop => '♝',
            ColoredPiece::BlackRook => '♜',
            ColoredPiece::BlackQueen => '♛',
            ColoredPiece::BlackKing => '♚',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colored_piece() {
        assert_eq!(ColoredPiece::NoPiece as u8, 0);
        assert_eq!(ColoredPiece::WhitePawn as u8, 1);
        assert_eq!(ColoredPiece::BlackPawn as u8, 9);

        assert_eq!(ColoredPiece::LIMIT, 15);
        assert_eq!(ColoredPiece::COLOR_DIFFERENCE, 8);

        assert_eq!(
            ColoredPiece::new(Color::White, Piece::Pawn),
            ColoredPiece::WhitePawn
        );
        assert_eq!(
            ColoredPiece::new(Color::Black, Piece::Pawn),
            ColoredPiece::BlackPawn
        );

        assert_eq!(ColoredPiece::WhitePawn.color(), Color::White);
        assert_eq!(ColoredPiece::BlackPawn.color(), Color::Black);

        assert_eq!(ColoredPiece::WhitePawn.piece(), Piece::Pawn);
        assert_eq!(ColoredPiece::BlackPawn.piece(), Piece::Pawn);

        assert_eq!(
            ColoredPiece::from_ascii('P'),
            ColoredPiece::WhitePawn
        );
        assert_eq!(
            ColoredPiece::from_ascii('p'),
            ColoredPiece::BlackPawn
        );
        assert_eq!(ColoredPiece::from_ascii(' '), ColoredPiece::NoPiece);

        assert_eq!(ColoredPiece::WhitePawn.ascii(), 'P');
        assert_eq!(ColoredPiece::BlackPawn.ascii(), 'p');
        assert_eq!(ColoredPiece::NoPiece.ascii(), ' ');

        assert_eq!(ColoredPiece::WhitePawn.unicode(), '♙');
        assert_eq!(ColoredPiece::BlackPawn.unicode(), '♟');
        assert_eq!(ColoredPiece::NoPiece.unicode(), ' ');
    }
}
