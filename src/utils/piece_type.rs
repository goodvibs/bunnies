#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// Represents the type of a piece.
/// Includes NoPieceType, Pawn, Knight, Bishop, Rook, Queen, and King.
/// The values are 0, 1, 2, 3, 4, 5, and 6 respectively.
pub enum PieceType {
    NoPieceType=0,
    Pawn=1,
    Knight=2,
    Bishop=3,
    Rook=4,
    Queen=5,
    King=6
}

impl PieceType {
    pub const LIMIT: u8 = 7;
    pub const AllPieceTypes: PieceType = PieceType::NoPieceType;

    /// Returns the PieceType from the given number.
    /// # Safety
    /// The number must be less than PieceType::LIMIT.
    pub const unsafe fn from(piece_type_number: u8) -> PieceType {
        assert!(piece_type_number < PieceType::LIMIT, "Piece type number out of bounds");
        unsafe { std::mem::transmute::<u8, PieceType>(piece_type_number) }
    }

    /// Returns the PieceType from the given uppercase char.
    pub const fn from_uppercase_char(piece_char: char) -> PieceType {
        match piece_char {
            'P' => PieceType::Pawn,
            'N' => PieceType::Knight,
            'B' => PieceType::Bishop,
            'R' => PieceType::Rook,
            'Q' => PieceType::Queen,
            'K' => PieceType::King,
            _ => PieceType::NoPieceType
        }
    }
    
    /// Returns the PieceType from the given lowercase char.
    pub const fn from_lowercase_char(piece_char: char) -> PieceType {
        match piece_char {
            'p' => PieceType::Pawn,
            'n' => PieceType::Knight,
            'b' => PieceType::Bishop,
            'r' => PieceType::Rook,
            'q' => PieceType::Queen,
            'k' => PieceType::King,
            _ => PieceType::NoPieceType
        }
    }

    /// Returns the uppercase ASCII character corresponding to the PieceType.
    pub const fn uppercase_ascii(&self) -> char {
        match self {
            PieceType::NoPieceType => '_',
            PieceType::Pawn => 'P',
            PieceType::Knight => 'N',
            PieceType::Bishop => 'B',
            PieceType::Rook => 'R',
            PieceType::Queen => 'Q',
            PieceType::King => 'K'
        }
    }

    /// Returns the lowercase ASCII character corresponding to the PieceType.
    pub const fn lowercase_ascii(&self) -> char {
        match self {
            PieceType::NoPieceType => '_',
            PieceType::Pawn => 'p',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Rook => 'r',
            PieceType::Queen => 'q',
            PieceType::King => 'k'
        }
    }

    /// Returns the unfilled Unicode character corresponding to the PieceType.
    pub const fn unfilled_unicode(&self) -> char {
        match self {
            PieceType::NoPieceType => ' ',
            PieceType::Pawn => '♙',
            PieceType::Knight => '♘',
            PieceType::Bishop => '♗',
            PieceType::Rook => '♖',
            PieceType::Queen => '♕',
            PieceType::King => '♔'
        }
    }

    /// Returns the filled Unicode character corresponding to the PieceType.
    pub const fn filled_unicode(&self) -> char {
        match self {
            PieceType::NoPieceType => ' ',
            PieceType::Pawn => '♟',
            PieceType::Knight => '♞',
            PieceType::Bishop => '♝',
            PieceType::Rook => '♜',
            PieceType::Queen => '♛',
            PieceType::King => '♚'
        }
    }

    /// An array of all PieceTypes (7 in total).
    pub const ALL: [PieceType; 7] = [
        PieceType::NoPieceType,
        PieceType::Pawn,
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::Queen,
        PieceType::King
    ];

    /// An array of all PieceTypes representing actual pieces (6 in total).
    pub const PIECES: [PieceType; 6] = [
        PieceType::Pawn,
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::Queen,
        PieceType::King
    ];

    /// An array of all PieceTypes representing non-king pieces (5 in total).
    pub const NON_KING_PIECES: [PieceType; 5] = [
        PieceType::Pawn,
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::Queen
    ];

    /// An array of all PieceTypes representing promotion pieces (4 in total).
    pub const PROMOTION_PIECES: [PieceType; 4] = [
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::Queen
    ];
}