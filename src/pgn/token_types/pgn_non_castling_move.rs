use logos::Lexer;
use regex::Regex;
use static_init::dynamic;
use crate::pgn::token::{ParsablePgnToken, PgnToken};
use crate::pgn::lexing_error::PgnLexingError;
use crate::pgn::token_types::pgn_move::{PgnCommonMoveInfo, PgnMove};
use crate::utils::PieceType;
use crate::r#move::{Move, MoveFlag};
use crate::utils::Square;
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
const NON_CASTLING_MOVE_REGEX: &str = r"([PNBRQK])?([a-h])?([1-8])?(x)?([a-h])([1-8])(?:=([NBRQ]))?([+#])?([?!]*)\s*(?:\$([0-9]+))?";

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

    fn render(&self, include_annotation: bool, include_nag: bool) -> String {
        let piece = if self.piece_moved == PieceType::Pawn {
            ""
        } else {
            &*self.piece_moved.to_char().to_string()
        };

        let disambiguation = match (self.disambiguation_file, self.disambiguation_rank) {
            (Some(file), Some(rank)) => format!("{}{}", file, rank),
            (Some(file), None) => file.to_string(),
            (None, Some(rank)) => rank.to_string(),
            (None, None) => "".to_string()
        };

        let capture = if self.is_capture { "x" } else { "" };

        let destination = self.to.to_string();

        let promotion = if self.promoted_to != PieceType::NoPieceType {
            format!("={}", self.promoted_to.to_char())
        } else {
            "".to_string()
        };

        let ending = self.common_move_info.render(include_annotation, include_nag);

        format!("{}{}{}{}{}{}", piece, disambiguation, capture, destination, promotion, ending)
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
                Some(m) => unsafe { PieceType::from_char(m.as_str().chars().next().unwrap()) },
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

#[cfg(test)]
mod tests {
    use logos::Logos;
    use super::*;
    use crate::pgn::token::PgnToken;
    use crate::utils::PieceType;
    use crate::r#move::Move;
    use crate::utils::Square;

    #[test]
    fn test_parse_pawn_move() {
        let mut lex = PgnToken::lexer("e4");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, PieceType::Pawn);
        assert_eq!(move_data.to, Square::E4);
        assert_eq!(move_data.is_capture, false);
        assert_eq!(move_data.promoted_to, PieceType::NoPieceType);
        assert_eq!(move_data.disambiguation_file, None);
        assert_eq!(move_data.disambiguation_rank, None);
    }

    #[test]
    fn test_parse_piece_move() {
        let mut lex = PgnToken::lexer("Nf3");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, PieceType::Knight);
        assert_eq!(move_data.to, Square::F3);
        assert_eq!(move_data.is_capture, false);
    }

    #[test]
    fn test_parse_capture_move() {
        let mut lex = PgnToken::lexer("Bxe5");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, PieceType::Bishop);
        assert_eq!(move_data.to, Square::E5);
        assert_eq!(move_data.is_capture, true);
    }

    #[test]
    fn test_parse_pawn_capture() {
        let mut lex = PgnToken::lexer("exd5");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, PieceType::Pawn);
        assert_eq!(move_data.to, Square::D5);
        assert_eq!(move_data.is_capture, true);
        assert_eq!(move_data.disambiguation_file, Some('e'));
    }

    #[test]
    fn test_parse_promotion() {
        let mut lex = PgnToken::lexer("e8=Q");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, PieceType::Pawn);
        assert_eq!(move_data.to, Square::E8);
        assert_eq!(move_data.promoted_to, PieceType::Queen);
    }

    #[test]
    fn test_parse_promotion_with_capture() {
        let mut lex = PgnToken::lexer("dxe8=Q+");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, PieceType::Pawn);
        assert_eq!(move_data.to, Square::E8);
        assert_eq!(move_data.is_capture, true);
        assert_eq!(move_data.promoted_to, PieceType::Queen);
        assert_eq!(move_data.common_move_info.is_check, true);
    }

    #[test]
    fn test_parse_with_disambiguation_file() {
        let mut lex = PgnToken::lexer("Rfe1");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, PieceType::Rook);
        assert_eq!(move_data.to, Square::E1);
        assert_eq!(move_data.disambiguation_file, Some('f'));
        assert_eq!(move_data.disambiguation_rank, None);
    }

    #[test]
    fn test_parse_with_disambiguation_rank() {
        let mut lex = PgnToken::lexer("R2e1");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, PieceType::Rook);
        assert_eq!(move_data.to, Square::E1);
        assert_eq!(move_data.disambiguation_file, None);
        assert_eq!(move_data.disambiguation_rank, Some('2'));
    }

    #[test]
    fn test_parse_with_both_disambiguation() {
        let mut lex = PgnToken::lexer("Qd5e4");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, PieceType::Queen);
        assert_eq!(move_data.to, Square::E4);
        assert_eq!(move_data.disambiguation_file, Some('d'));
        assert_eq!(move_data.disambiguation_rank, Some('5'));
    }

    #[test]
    fn test_parse_with_check() {
        let mut lex = PgnToken::lexer("Qe4+");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, PieceType::Queen);
        assert_eq!(move_data.to, Square::E4);
        assert_eq!(move_data.common_move_info.is_check, true);
        assert_eq!(move_data.common_move_info.is_checkmate, false);
    }

    #[test]
    fn test_parse_with_checkmate() {
        let mut lex = PgnToken::lexer("Qe4#");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, PieceType::Queen);
        assert_eq!(move_data.to, Square::E4);
        assert_eq!(move_data.common_move_info.is_check, true);
        assert_eq!(move_data.common_move_info.is_checkmate, true);
    }

    #[test]
    fn test_parse_with_annotation() {
        let mut lex = PgnToken::lexer("Qe4!?");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, PieceType::Queen);
        assert_eq!(move_data.to, Square::E4);
        assert_eq!(move_data.common_move_info.annotation, Some("!?".to_string()));
    }

    #[test]
    fn test_parse_with_nag() {
        let mut lex = PgnToken::lexer("Qe4 $1");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, PieceType::Queen);
        assert_eq!(move_data.to, Square::E4);
        assert_eq!(move_data.common_move_info.nag, Some(1));
    }

    #[test]
    fn test_parse_complex_move() {
        let mut lex = PgnToken::lexer("Rd3xe3+!? $2");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, PieceType::Rook);
        assert_eq!(move_data.to, Square::E3);
        assert_eq!(move_data.is_capture, true);
        assert_eq!(move_data.disambiguation_file, Some('d'));
        assert_eq!(move_data.disambiguation_rank, Some('3'));
        assert_eq!(move_data.common_move_info.is_check, true);
        assert_eq!(move_data.common_move_info.annotation, Some("!?".to_string()));
        assert_eq!(move_data.common_move_info.nag, Some(2));
    }

    #[test]
    fn test_matches_move() {
        let state = State::from_fen("r1bqkbnr/ppp2ppp/2np4/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 4").unwrap();

        // Test knight move
        let knight_move = PgnNonCastlingMove {
            piece_moved: PieceType::Knight,
            disambiguation_file: None,
            disambiguation_rank: None,
            to: Square::D4,
            promoted_to: PieceType::NoPieceType,
            is_capture: false,
            common_move_info: PgnCommonMoveInfo {
                is_check: false,
                is_checkmate: false,
                annotation: None,
                nag: None
            }
        };

        let actual_move = Move::new_non_promotion(
            Square::D4,
            Square::F3,
            MoveFlag::NormalMove
        );

        assert!(knight_move.matches_move(actual_move, &state));

        // Test with disambiguation
        let knight_move_with_file = {
            let mut knight_move = knight_move.clone();
            knight_move.disambiguation_file = Some('f');
            knight_move
        };

        assert!(knight_move_with_file.matches_move(actual_move, &state));

        // Test with incorrect file disambiguation
        let knight_move_with_wrong_file = {
            let mut knight_move = knight_move.clone();
            knight_move.disambiguation_file = Some('e');
            knight_move
        };

        assert!(!knight_move_with_wrong_file.matches_move(actual_move, &state));
    }

    #[test]
    fn test_invalid_move() {
        let mut lex = PgnToken::lexer("Xx9");
        lex.next();
        let result = PgnNonCastlingMove::parse(&mut lex);
        assert!(result.is_err());
    }
}