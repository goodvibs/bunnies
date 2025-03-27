use crate::Color;
use crate::PieceType;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// Represents a colored (white or black) piece on the board.
pub enum ColoredPieceType {
    NoPiece=0,
    WhitePawn=1, WhiteKnight=2, WhiteBishop=3, WhiteRook=4, WhiteQueen=5, WhiteKing=6,
    BlackPawn=9, BlackKnight=10, BlackBishop=11, BlackRook=12, BlackQueen=13, BlackKing=14
}

impl ColoredPieceType {
    /// The number of possible variants.
    pub const LIMIT: usize = 15;
    /// The numeric difference between the white and black colored pieces values.
    pub const COLOR_DIFFERENCE: u8 = 8;

    /// Returns a new ColoredPieceType.
    pub const fn new(color: Color, piece_type: PieceType) -> ColoredPieceType {
        let piece_type_int = piece_type as u8;
        let is_piece = piece_type_int != PieceType::NoPieceType as u8;
        let color_int_shifted = (is_piece as u8 & color as u8) << 3;
        unsafe { std::mem::transmute::<u8, ColoredPieceType>(color_int_shifted | piece_type_int) }
    }

    /// Returns the color of the piece.
    pub const fn color(&self) -> Color {
        unsafe { std::mem::transmute::<u8, Color>(*self as u8 >> 3) }
    }

    /// Returns the piece type of the piece.
    pub const fn piece_type(&self) -> PieceType {
        unsafe { std::mem::transmute::<u8, PieceType>(*self as u8 & 0b111) }
    }

    /// Returns a ColoredPieceType from an ASCII character.
    pub const fn from_ascii(c: char) -> ColoredPieceType {
        match c {
            'P' => ColoredPieceType::WhitePawn,
            'N' => ColoredPieceType::WhiteKnight,
            'B' => ColoredPieceType::WhiteBishop,
            'R' => ColoredPieceType::WhiteRook,
            'Q' => ColoredPieceType::WhiteQueen,
            'K' => ColoredPieceType::WhiteKing,
            'p' => ColoredPieceType::BlackPawn,
            'n' => ColoredPieceType::BlackKnight,
            'b' => ColoredPieceType::BlackBishop,
            'r' => ColoredPieceType::BlackRook,
            'q' => ColoredPieceType::BlackQueen,
            'k' => ColoredPieceType::BlackKing,
            _ => ColoredPieceType::NoPiece
        }
    }

    /// Returns the ASCII character representation of the piece.
    pub const fn ascii(&self) -> char {
        match self {
            ColoredPieceType::NoPiece => ' ',
            ColoredPieceType::WhitePawn => 'P',
            ColoredPieceType::WhiteKnight => 'N',
            ColoredPieceType::WhiteBishop => 'B',
            ColoredPieceType::WhiteRook => 'R',
            ColoredPieceType::WhiteQueen => 'Q',
            ColoredPieceType::WhiteKing => 'K',
            ColoredPieceType::BlackPawn => 'p',
            ColoredPieceType::BlackKnight => 'n',
            ColoredPieceType::BlackBishop => 'b',
            ColoredPieceType::BlackRook => 'r',
            ColoredPieceType::BlackQueen => 'q',
            ColoredPieceType::BlackKing => 'k'
        }
    }

    /// Returns the Unicode character representation of the piece.
    /// White pieces are unfilled, black pieces are filled.
    pub const fn unicode(&self) -> char {
        match self {
            ColoredPieceType::NoPiece => ' ',
            ColoredPieceType::WhitePawn => '♙',
            ColoredPieceType::WhiteKnight => '♘',
            ColoredPieceType::WhiteBishop => '♗',
            ColoredPieceType::WhiteRook => '♖',
            ColoredPieceType::WhiteQueen => '♕',
            ColoredPieceType::WhiteKing => '♔',
            ColoredPieceType::BlackPawn => '♟',
            ColoredPieceType::BlackKnight => '♞',
            ColoredPieceType::BlackBishop => '♝',
            ColoredPieceType::BlackRook => '♜',
            ColoredPieceType::BlackQueen => '♛',
            ColoredPieceType::BlackKing => '♚'
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colored_piece() {
        assert_eq!(ColoredPieceType::NoPiece as u8, 0);
        assert_eq!(ColoredPieceType::WhitePawn as u8, 1);
        assert_eq!(ColoredPieceType::BlackPawn as u8, 9);

        assert_eq!(ColoredPieceType::LIMIT, 15);
        assert_eq!(ColoredPieceType::COLOR_DIFFERENCE, 8);

        assert_eq!(ColoredPieceType::new(Color::White, PieceType::Pawn), ColoredPieceType::WhitePawn);
        assert_eq!(ColoredPieceType::new(Color::Black, PieceType::Pawn), ColoredPieceType::BlackPawn);

        assert_eq!(ColoredPieceType::WhitePawn.color(), Color::White);
        assert_eq!(ColoredPieceType::BlackPawn.color(), Color::Black);

        assert_eq!(ColoredPieceType::WhitePawn.piece_type(), PieceType::Pawn);
        assert_eq!(ColoredPieceType::BlackPawn.piece_type(), PieceType::Pawn);

        assert_eq!(ColoredPieceType::from_ascii('P'), ColoredPieceType::WhitePawn);
        assert_eq!(ColoredPieceType::from_ascii('p'), ColoredPieceType::BlackPawn);
        assert_eq!(ColoredPieceType::from_ascii(' '), ColoredPieceType::NoPiece);

        assert_eq!(ColoredPieceType::WhitePawn.ascii(), 'P');
        assert_eq!(ColoredPieceType::BlackPawn.ascii(), 'p');
        assert_eq!(ColoredPieceType::NoPiece.ascii(), ' ');

        assert_eq!(ColoredPieceType::WhitePawn.unicode(), '♙');
        assert_eq!(ColoredPieceType::BlackPawn.unicode(), '♟');
        assert_eq!(ColoredPieceType::NoPiece.unicode(), ' ');
    }
}