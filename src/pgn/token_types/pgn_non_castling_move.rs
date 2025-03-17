use logos::Lexer;
use regex::Regex;
use static_init::dynamic;
use crate::pgn::pgn_token::{ParsablePgnToken, PgnToken};
use crate::pgn::lexing_error::PgnLexingError;
use crate::pgn::token_types::pgn_move::{PgnCommonMoveInfo, PgnMove};
use crate::piece_type::PieceType;
use crate::r#move::{Move, MoveFlag};
use crate::square::Square;
use crate::state::State;

/// Regex for parsing non-castling moves.
/// Capturing groups:
/// 0. Everything
/// 1. Piece moved (optional, pawn doesn't require this)
/// 2. Disambiguation file (optional)
/// 3. Disambiguation rank (optional)
/// 4. Capture (optional)
/// 5. Destination file
/// 6. Destination rank
/// 7. Promotion (optional)
/// 8. Check or checkmate (optional)
/// 9. Annotation (optional)
/// 10. NAG (optional)
const NON_CASTLING_MOVE_REGEX: &str = r"([PNBRQK]?)([a-h]?)([1-8]?)(x?)([a-h])([1-8])(?:=([NBRQ]))?([+#])?([?!]*)\s*(?:\$([0-9]+))?";

#[dynamic]
static COMPILED_NON_CASTLING_MOVE_REGEX: Regex = Regex::new(NON_CASTLING_MOVE_REGEX).unwrap();

#[derive(Clone, Debug, PartialEq)]
pub struct PgnNonCastlingMove {
    pub disambiguation_file: Option<char>,
    pub disambiguation_rank: Option<char>,
    pub to: Square,
    pub piece_moved: PieceType,
    pub promoted_to: PieceType,
    pub is_capture: bool,
    pub common_move_info: PgnCommonMoveInfo
}

impl PgnMove for PgnNonCastlingMove {
    fn matches_move(&self, mv: Move, initial_state: &State) -> bool {
        let dst = mv.get_destination();
        let src = mv.get_source();
        let flag = mv.get_flag();
        let promotion = match flag {
            MoveFlag::Promotion => mv.get_promotion(),
            _ => PieceType::NoPieceType
        };

        if self.to != dst {
            return false
        } else if self.promoted_to != promotion {
            return false
        } else if self.piece_moved != initial_state.board.get_piece_type_at(src) {
            return false
        } else if self.is_capture != mv.is_capture(initial_state) {
            return false
        } else if let Some(file) = self.disambiguation_file {
            if src.get_file() != file as u8 - 'a' as u8 {
                return false
            }
        } else if let Some(rank) = self.disambiguation_rank {
            if src.get_rank() != rank as u8 - '1' as u8 {
                return false
            }
        }

        true
    }

    fn get_common_move_info(&self) -> &PgnCommonMoveInfo {
        &self.common_move_info
    }

    fn get_common_move_info_mut(&mut self) -> &mut PgnCommonMoveInfo {
        &mut self.common_move_info
    }
}

impl ParsablePgnToken for PgnNonCastlingMove {
    fn parse(lex: &mut Lexer<PgnToken>) -> Result<Self, PgnLexingError> {
        let text = lex.slice();
        if let Some(captures) = COMPILED_NON_CASTLING_MOVE_REGEX.captures(text) {
            let piece_moved = match captures.get(1).map(|m| m.as_str().chars().next().unwrap()) {
                None => PieceType::Pawn,
                Some(c) => unsafe { PieceType::from_char(c) }
            };

            let disambiguation_file = captures.get(2).map(|m| m.as_str().chars().next().unwrap());
            let disambiguation_rank = captures.get(3).map(|m| m.as_str().chars().next().unwrap());

            let to_file_char = captures.get(5).unwrap().as_str().chars().next().unwrap();
            let to_rank_char = captures.get(6).unwrap().as_str().chars().next().unwrap();
            let to_file = to_file_char as u8 - 'a' as u8;
            let to_rank = to_rank_char as u8 - '1' as u8;
            let to = unsafe { Square::from_rank_file(to_rank, to_file) };

            let promoted_to = match captures.get(7) {
                Some(m) => {
                    let promoted_to_char = m.as_str().chars().nth(1).unwrap();
                    unsafe { PieceType::from_char(promoted_to_char) }
                },
                None => PieceType::NoPieceType
            };

            let is_capture = captures.get(4).is_some();
            let check_or_checkmate = captures.get(8);
            let annotation = captures.get(9);
            let nag = captures.get(10);

            Ok(
                PgnNonCastlingMove {
                    disambiguation_file,
                    disambiguation_rank,
                    to,
                    piece_moved,
                    promoted_to,
                    is_capture,
                    common_move_info: PgnCommonMoveInfo::from(check_or_checkmate, annotation, nag)
                }
            )
        } else {
            Err(PgnLexingError::InvalidMove(text.to_string()))
        }
    }
}