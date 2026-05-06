use crate::r#move::{Move, MoveFlag};
use crate::pgn::lexing_error::PgnLexingError;
use crate::pgn::token::{ParsablePgnToken, PgnToken};
use crate::pgn::token_types::pgn_move::{PgnCommonMoveInfo, PgnMove};
use crate::position::Board;
use crate::{File, Flank};
use logos::Lexer;
use regex::Regex;
use std::sync::LazyLock;

/// The regex pattern for a castling move.
/// Capturing groups:
/// 0. Everything
/// 1. Queenside castling
/// 2. Kingside castling
/// 3. Check or checkmate
/// 4. Annotation
/// 5. NAG
const CASTLING_MOVE_REGEX: &str = r"(?:(O-O-O|0-0-0)|(O-O|0-0))([+#])?([?!]+)?\s*(?:\$([0-9]+))?";

static COMPILED_CASTLING_MOVE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(CASTLING_MOVE_REGEX).unwrap());

#[derive(Clone, Debug, PartialEq)]
pub struct PgnCastlingMove {
    pub flank: Flank,
    pub common_move_info: PgnCommonMoveInfo,
}

impl PgnMove for PgnCastlingMove {
    fn matches_move(&self, move_: Move, _from_board: &Board) -> bool {
        let flag = move_.flag();
        let matches_flank = match self.flank {
            Flank::Kingside => move_.to().file() == File::G,
            Flank::Queenside => move_.to().file() == File::C,
        };
        if flag != MoveFlag::Castling || !matches_flank {
            return false;
        }

        true
    }

    fn get_common_move_info(&self) -> &PgnCommonMoveInfo {
        &self.common_move_info
    }
}

impl ParsablePgnToken for PgnCastlingMove {
    fn parse(lex: &mut Lexer<PgnToken>) -> Result<Self, PgnLexingError> {
        let text = lex.slice();
        if let Some(captures) = COMPILED_CASTLING_MOVE_REGEX.captures(text) {
            let flank = if captures.get(2).is_some() {
                Flank::Kingside
            } else {
                Flank::Queenside
            };

            let check_or_checkmate = captures.get(3);
            let annotation = captures.get(4);
            let nag = captures.get(5);

            Ok(PgnCastlingMove {
                flank,
                common_move_info: PgnCommonMoveInfo::from(check_or_checkmate, annotation, nag),
            })
        } else {
            Err(PgnLexingError::InvalidCastlingMove(text.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;
    use crate::Square;
    use crate::r#move::{Move, MoveFlag};
    use crate::pgn::token::ParsablePgnToken;
    use crate::pgn::token_types::pgn_move::PgnMove;
    use crate::position::{INITIAL_FEN, Position};
    use logos::Logos;

    #[test]
    fn test_parse_kingside_castling_move() {
        let mut lex = PgnToken::lexer("O-O");
        lex.next();
        let castling_move = PgnCastlingMove::parse(&mut lex).unwrap();
        assert_eq!(castling_move.flank, Flank::Kingside);
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
        assert_eq!(castling_move.flank, Flank::Queenside);
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
        assert_eq!(castling_move.flank, Flank::Kingside);
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
        assert_eq!(castling_move.flank, Flank::Queenside);
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
        assert_eq!(castling_move.flank, Flank::Kingside);
        assert_eq!(castling_move.get_common_move_info().is_check, false);
        assert_eq!(castling_move.get_common_move_info().is_checkmate, false);
        assert_eq!(
            castling_move.get_common_move_info().annotation,
            Some("!?".to_string())
        );
        assert_eq!(castling_move.get_common_move_info().nag, None);
    }

    #[test]
    fn test_parse_queenside_castling_move_with_nag() {
        let mut lex = PgnToken::lexer("O-O-O $1");
        lex.next();
        let castling_move = PgnCastlingMove::parse(&mut lex).unwrap();
        assert_eq!(castling_move.flank, Flank::Queenside);
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
        assert_eq!(castling_move.flank, Flank::Kingside);
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
            flank: Flank::Kingside,
            common_move_info: PgnCommonMoveInfo {
                is_check: false,
                is_checkmate: false,
                annotation: None,
                nag: None,
            },
        };
        let state = Position::<1, { Color::White }>::from_fen(INITIAL_FEN).unwrap();
        let kingside_castling_move =
            Move::new_non_promotion(Square::E8, Square::G8, MoveFlag::Castling);
        let queenside_castling_move =
            Move::new_non_promotion(Square::E8, Square::C8, MoveFlag::Castling);
        let kingside_match = castling_move.matches_move(kingside_castling_move, &state.board);
        let queenside_match = castling_move.matches_move(queenside_castling_move, &state.board);
        assert_eq!(kingside_match, true);
        assert_eq!(queenside_match, false);
    }
}
