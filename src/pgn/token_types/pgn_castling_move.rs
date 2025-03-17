use logos::Lexer;
use regex::Regex;
use static_init::{dynamic};
use crate::pgn::pgn_token::{ParsablePgnToken, PgnToken};
use crate::pgn::lexing_error::PgnLexingError;
use crate::pgn::token_types::pgn_move::{PgnCommonMoveInfo, PgnMove};
use crate::r#move::{Move, MoveFlag};
use crate::state::State;

/// The regex pattern for a castling move.
/// Capturing groups:
/// 0. Everything
/// 1. Queenside castling
/// 2. Kingside castling
/// 3. Check or checkmate
/// 4. Annotation
/// 5. NAG
const CASTLING_MOVE_REGEX: &str = r"(?:(O-O-O|0-0-0)|(O-O|0-0))([+#])?([?!]+)?\s*(?:\$([0-9]+))?";

#[dynamic]
static COMPILED_CASTLING_MOVE_REGEX: Regex = Regex::new(CASTLING_MOVE_REGEX).unwrap();

#[derive(Clone, Debug, PartialEq)]
pub struct PgnCastlingMove {
    pub is_kingside: bool,
    pub common_move_info: PgnCommonMoveInfo
}

impl PgnMove for PgnCastlingMove {
    fn matches_move(&self, mv: Move, initial_state: &State) -> bool {
        let flag = mv.get_flag();
        if flag != MoveFlag::Castling {
            return false
        } else if self.is_kingside != (mv.get_destination().get_file() == 6) {
            return false
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

impl ParsablePgnToken for PgnCastlingMove {
    fn parse(lex: &mut Lexer<PgnToken>) -> Result<Self, PgnLexingError> {
        let text = lex.slice();
        if let Some(captures) = COMPILED_CASTLING_MOVE_REGEX.captures(text) {
            let is_kingside = captures.get(2).is_some();

            let check_or_checkmate = captures.get(3);
            let annotation = captures.get(4);
            let nag = captures.get(5);

            Ok(
                PgnCastlingMove {
                    is_kingside,
                    common_move_info: PgnCommonMoveInfo::from(check_or_checkmate, annotation, nag)
                }
            )
        } else {
            Err(PgnLexingError::InvalidCastlingMove(text.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use logos::Logos;
    use super::*;
    use crate::pgn::pgn_token::ParsablePgnToken;
    use crate::pgn::token_types::pgn_move::PgnMove;
    use crate::r#move::{Move, MoveFlag};
    use crate::square::Square;
    use crate::state::State;

    #[test]
    fn test_parse_kingside_castling_move() {
        let mut lex = PgnToken::lexer("O-O");
        lex.next();
        let castling_move = PgnCastlingMove::parse(&mut lex).unwrap();
        assert_eq!(castling_move.is_kingside, true);
        assert_eq!(castling_move.get_common_move_info().is_check, false);
        assert_eq!(castling_move.get_common_move_info().is_checkmate, false);
        assert_eq!(castling_move.get_common_move_info().annotation, None);
        assert_eq!(castling_move.get_common_move_info().nag, None);
    }

    #[test]
    fn test_parse_queenside_castling_move() {
        let mut lex = PgnToken::lexer("O-O-O");
        lex.next();
        let castling_move = PgnCastlingMove::parse(&mut lex).unwrap();
        assert_eq!(castling_move.is_kingside, false);
        assert_eq!(castling_move.get_common_move_info().is_check, false);
        assert_eq!(castling_move.get_common_move_info().is_checkmate, false);
        assert_eq!(castling_move.get_common_move_info().annotation, None);
        assert_eq!(castling_move.get_common_move_info().nag, None);
    }

    #[test]
    fn test_parse_kingside_castling_move_with_check() {
        let mut lex = PgnToken::lexer("O-O+");
        lex.next();
        let castling_move = PgnCastlingMove::parse(&mut lex).unwrap();
        assert_eq!(castling_move.is_kingside, true);
        assert_eq!(castling_move.get_common_move_info().is_check, true);
        assert_eq!(castling_move.get_common_move_info().is_checkmate, false);
        assert_eq!(castling_move.get_common_move_info().annotation, None);
        assert_eq!(castling_move.get_common_move_info().nag, None);
    }

    #[test]
    fn test_parse_queenside_castling_move_with_checkmate() {
        let mut lex = PgnToken::lexer("0-0-0#");
        lex.next();
        let castling_move = PgnCastlingMove::parse(&mut lex).unwrap();
        assert_eq!(castling_move.is_kingside, false);
        assert_eq!(castling_move.get_common_move_info().is_check, true);
        assert_eq!(castling_move.get_common_move_info().is_checkmate, true);
        assert_eq!(castling_move.get_common_move_info().annotation, None);
        assert_eq!(castling_move.get_common_move_info().nag, None);
    }

    #[test]
    fn test_parse_kingside_castling_move_with_annotation() {
        let mut lex = PgnToken::lexer("O-O!?");
        lex.next();
        let castling_move = PgnCastlingMove::parse(&mut lex).unwrap();
        assert_eq!(castling_move.is_kingside, true);
        assert_eq!(castling_move.get_common_move_info().is_check, false);
        assert_eq!(castling_move.get_common_move_info().is_checkmate, false);
        assert_eq!(castling_move.get_common_move_info().annotation, Some("!?".to_string()));
        assert_eq!(castling_move.get_common_move_info().nag, None);
    }

    #[test]
    fn test_parse_queenside_castling_move_with_nag() {
        let mut lex = PgnToken::lexer("O-O-O $1");
        lex.next();
        let castling_move = PgnCastlingMove::parse(&mut lex).unwrap();
        assert_eq!(castling_move.is_kingside, false);
        assert_eq!(castling_move.get_common_move_info().is_check, false);
        assert_eq!(castling_move.get_common_move_info().is_checkmate, false);
        assert_eq!(castling_move.get_common_move_info().annotation, None);
        assert_eq!(castling_move.get_common_move_info().nag, Some(1));
    }

    #[test]
    fn test_parse_kingside_castling_move_with_checkmate_and_nag() {
        let mut lex = PgnToken::lexer("O-O# $1");
        lex.next();
        let castling_move = PgnCastlingMove::parse(&mut lex).unwrap();
        assert_eq!(castling_move.is_kingside, true);
        assert_eq!(castling_move.get_common_move_info().is_check, true);
        assert_eq!(castling_move.get_common_move_info().is_checkmate, true);
        assert_eq!(castling_move.get_common_move_info().annotation, None);
        assert_eq!(castling_move.get_common_move_info().nag, Some(1));
    }

    #[test]
    fn test_parse_invalid_castling_move() {
        let mut lex = PgnToken::lexer("O-0");
        lex.next();
        let castling_move = PgnCastlingMove::parse(&mut lex);
        assert_eq!(castling_move.is_err(), true);
    }

    #[test]
    fn test_matches_move() {
        let castling_move = PgnCastlingMove {
            is_kingside: true,
            common_move_info: PgnCommonMoveInfo {
                is_check: false,
                is_checkmate: false,
                annotation: None,
                nag: None
            }
        };
        let state = State::initial();
        let kingside_castling_move = Move::new_non_promotion(Square::G8, Square::E8, MoveFlag::Castling);
        let queenside_castling_move = Move::new_non_promotion(Square::C8, Square::E8, MoveFlag::Castling);
        assert_eq!(castling_move.matches_move(kingside_castling_move, &state), true);
        assert_eq!(castling_move.matches_move(queenside_castling_move, &state), false);
    }
}