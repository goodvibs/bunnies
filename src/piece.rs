#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// Represents a piece on the board.
/// Includes Null, Pawn, Knight, Bishop, Rook, Queen, and King.
/// The values are 0, 1, 2, 3, 4, 5, and 6 respectively.
pub enum Piece {
    Null = 0,
    Pawn = 1,
    Knight = 2,
    Bishop = 3,
    Rook = 4,
    Queen = 5,
    King = 6,
}

impl Piece {
    pub const LIMIT: u8 = 7;
    pub const ALL_PIECES: Piece = Piece::Null;

    /// Returns the Piece from the given number.
    /// # Safety
    /// The number must be less than Piece::LIMIT.
    pub const unsafe fn from(piece_type_number: u8) -> Piece {
        assert!(
            piece_type_number < Piece::LIMIT,
            "Piece type number out of bounds"
        );
        unsafe { std::mem::transmute::<u8, Piece>(piece_type_number) }
    }

    pub fn is_sliding_piece(&self) -> bool {
        Self::SLIDING_PIECES.contains(self)
    }

    /// Returns the Piece from the given uppercase char.
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

    /// An array of all Pieces (7 in total).
    pub const ALL: [Piece; 7] = [
        Piece::Null,
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::King,
    ];

    /// An array of all Pieces representing actual pieces (6 in total).
    pub const PIECES: [Piece; 6] = [
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::King,
    ];

    /// An array of all Pieces representing non-king pieces (5 in total).
    pub const NON_KING_PIECES: [Piece; 5] = [
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
    ];

    /// An array of all Pieces representing promotion pieces (4 in total).
    pub const PROMOTION_PIECES: [Piece; 4] = [
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
    ];

    pub const SLIDING_PIECES: [Piece; 3] =
        [Piece::Bishop, Piece::Rook, Piece::Queen];
}
