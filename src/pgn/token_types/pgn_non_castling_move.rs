use logos::Lexer;
use regex::Regex;
use crate::pgn::pgn_token::{ParsablePgnToken, PgnToken};
use crate::pgn::lexing_error::PgnLexingError;
use crate::pgn::token_types::pgn_move::{PgnCommonMoveInfo, PgnMove};
use crate::piece_type::PieceType;
use crate::r#move::{Move, MoveFlag};
use crate::square::Square;
use crate::state::State;

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
        let move_regex = Regex::new(r"([PNBRQK]?)([a-h]?)([1-8]?)(x?)([a-h])([1-8])(=[NBRQ])?([+#])?([?!]*)\s*(\$[0-9]+)?").unwrap();
        if let Some(captures) = move_regex.captures(text) {
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

            Ok(
                PgnNonCastlingMove {
                    disambiguation_file,
                    disambiguation_rank,
                    to,
                    piece_moved,
                    promoted_to,
                    is_capture,
                    common_move_info: PgnCommonMoveInfo::from(captures.get(8), captures.get(9), captures.get(10))
                }
            )
        } else {
            Err(PgnLexingError::InvalidMove(text.to_string()))
        }
    }
}