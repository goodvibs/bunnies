use crate::utils::Color;
use crate::utils::ColoredPiece;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PieceType {
    NoPieceType=0,
    Pawn=1,
    Knight=2,
    Bishop=3,
    Rook=4,
    Queen=5,
    King=6
}

const ALL: [PieceType; 7] = [
    PieceType::NoPieceType,
    PieceType::Pawn,
    PieceType::Knight,
    PieceType::Bishop,
    PieceType::Rook,
    PieceType::Queen,
    PieceType::King
];

const ALL_PIECES: [PieceType; 6] = [
    PieceType::Pawn,
    PieceType::Knight,
    PieceType::Bishop,
    PieceType::Rook,
    PieceType::Queen,
    PieceType::King
];

const ALL_NON_KING_PIECES: [PieceType; 5] = [
    PieceType::Pawn,
    PieceType::Knight,
    PieceType::Bishop,
    PieceType::Rook,
    PieceType::Queen
];

const ALL_PROMOTION_PIECES: [PieceType; 4] = [
    PieceType::Knight,
    PieceType::Bishop,
    PieceType::Rook,
    PieceType::Queen
];

impl PieceType {
    pub const LIMIT: u8 = 7;
    pub const AllPieceTypes: PieceType = PieceType::NoPieceType;

    pub const unsafe fn from(piece_type_number: u8) -> PieceType {
        assert!(piece_type_number < PieceType::LIMIT, "Piece type number out of bounds");
        unsafe { std::mem::transmute::<u8, PieceType>(piece_type_number) }
    }

    pub const unsafe fn from_char(piece_char: char) -> PieceType {
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

    pub const fn to_uppercase_char(&self) -> char {
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

    pub const fn to_lowercase_char(&self) -> char {
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

    pub const fn to_unfilled_unicode(&self) -> char {
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

    pub const fn to_filled_unicode(&self) -> char {
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

    pub fn iter_all_types() -> [PieceType; 7] {
        ALL
    }

    pub fn iter_pieces() -> [PieceType; 6] {
        ALL_PIECES
    }
    
    pub fn iter_promotion_pieces() -> [PieceType; 4] {
        ALL_PROMOTION_PIECES
    }
}