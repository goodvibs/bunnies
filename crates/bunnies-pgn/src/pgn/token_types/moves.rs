use crate::File;
use crate::Flank;
use crate::Piece;
use crate::Square;
use crate::r#move::{Move, MoveFlag};
use crate::pgn::error::PgnError;
use crate::pgn::token::{CASTLING_MOVE_REGEX, NON_CASTLING_MOVE_REGEX, ParsablePgnToken, PgnToken};
use crate::position::Board;
use logos::Lexer;
use regex::{Match, Regex};
use std::fmt::Debug;
use std::sync::LazyLock;

static COMPILED_NON_CASTLING_MOVE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(NON_CASTLING_MOVE_REGEX).unwrap());
static COMPILED_CASTLING_MOVE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(CASTLING_MOVE_REGEX).unwrap());

pub trait PgnMove: Debug {
    fn matches_move(&self, move_: Move, from_board: &Board) -> bool;

    fn get_common_move_info(&self) -> &PgnCommonMoveInfo;
}

#[derive(Clone, Debug, PartialEq)]
pub struct PgnCommonMoveInfo {
    pub is_check: bool,
    pub is_checkmate: bool,
    pub annotation: Option<String>,
    pub nag: Option<u8>,
}

impl PgnCommonMoveInfo {
    pub fn render(&self, include_annotation: bool, include_nag: bool) -> String {
        let check_or_checkmate = if self.is_checkmate {
            "#"
        } else if self.is_check {
            "+"
        } else {
            ""
        };

        let annotation = if include_annotation {
            self.annotation.as_deref().unwrap_or("")
        } else {
            ""
        };

        let nag = if include_nag {
            match self.nag {
                Some(nag) => format!(" ${}", nag),
                None => "".to_string(),
            }
        } else {
            "".to_string()
        };

        format!("{}{}{}", check_or_checkmate, annotation, nag)
    }

    pub fn from(
        check_or_checkmate: Option<Match>,
        annotation: Option<Match>,
        nag: Option<Match>,
    ) -> PgnCommonMoveInfo {
        let (is_check, is_checkmate) = match check_or_checkmate {
            Some(m) => {
                let check_or_checkmate_char = m.as_str().chars().next().unwrap();
                (true, check_or_checkmate_char == '#')
            }
            None => (false, false),
        };

        let annotation = annotation.map(|m| m.as_str().to_string());

        let nag = match nag {
            Some(m) => m.as_str().parse().ok(),
            None => None,
        };

        PgnCommonMoveInfo {
            is_check,
            is_checkmate,
            annotation,
            nag,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PgnNonCastlingMove {
    pub disambiguation_file: Option<char>,
    pub disambiguation_rank: Option<char>,
    pub to: Square,
    pub piece_moved: Piece,
    pub promoted_to: Piece,
    pub is_capture: bool,
    pub common_move_info: PgnCommonMoveInfo,
}

impl PgnMove for PgnNonCastlingMove {
    fn matches_move(&self, move_: Move, board: &Board) -> bool {
        let to = move_.to();
        let from = move_.from();
        let flag = move_.flag();
        let promotion = match flag {
            MoveFlag::Promotion => move_.promotion(),
            _ => Piece::Null,
        };

        if self.to != to
            || self.promoted_to != promotion
            || self.piece_moved != board.piece_at(from)
            || self.is_capture != move_.is_capture_on_board(board)
        {
            return false;
        }

        if let Some(file) = self.disambiguation_file
            && from.file() as u8 != file as u8 - b'a'
        {
            return false;
        }

        if let Some(rank) = self.disambiguation_rank
            && from.rank() as u8 != rank as u8 - b'1'
        {
            return false;
        }

        true
    }

    fn get_common_move_info(&self) -> &PgnCommonMoveInfo {
        &self.common_move_info
    }
}

impl ParsablePgnToken for PgnNonCastlingMove {
    fn parse(lex: &mut Lexer<PgnToken>) -> Result<Self, PgnError> {
        let text = lex.slice();
        if let Some(captures) = COMPILED_NON_CASTLING_MOVE_REGEX.captures(text) {
            let piece_moved = match captures.get(1).map(|m| m.as_str().chars().next().unwrap()) {
                None => Piece::Pawn,
                Some(c) => Piece::from_uppercase_char(c),
            };

            let disambiguation_file = captures.get(2).map(|m| m.as_str().chars().next().unwrap());
            let disambiguation_rank = captures.get(3).map(|m| m.as_str().chars().next().unwrap());

            let to_file_char = captures.get(5).unwrap().as_str().chars().next().unwrap();
            let to_rank_char = captures.get(6).unwrap().as_str().chars().next().unwrap();
            let to_file = to_file_char as u8 - b'a';
            let to_rank = to_rank_char as u8 - b'1';
            let to = Square::from_rank_and_file(
                unsafe { to_rank.try_into().unwrap_unchecked() },
                unsafe { File::try_from(to_file).unwrap_unchecked() },
            );

            let promoted_to = match captures.get(7) {
                Some(m) => Piece::from_uppercase_char(m.as_str().chars().next().unwrap()),
                None => Piece::Null,
            };

            let is_capture = captures.get(4).is_some();
            let check_or_checkmate = captures.get(8);
            let annotation = captures.get(9);
            let nag = captures.get(10);

            Ok(PgnNonCastlingMove {
                disambiguation_file,
                disambiguation_rank,
                to,
                piece_moved,
                promoted_to,
                is_capture,
                common_move_info: PgnCommonMoveInfo::from(check_or_checkmate, annotation, nag),
            })
        } else {
            Err(PgnError::InvalidMove(text.to_string()))
        }
    }
}

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
    fn parse(lex: &mut Lexer<PgnToken>) -> Result<Self, PgnError> {
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
            Err(PgnError::InvalidCastlingMove(text.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;
    use crate::Square;
    use crate::pgn::token::PgnToken;
    use crate::position::{INITIAL_FEN, Position};
    use logos::Logos;

    #[test]
    fn test_parse_pawn_move() {
        let mut lex = PgnToken::lexer("e4");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, Piece::Pawn);
        assert_eq!(move_data.to, Square::E4);
        assert_eq!(move_data.is_capture, false);
        assert_eq!(move_data.promoted_to, Piece::Null);
        assert_eq!(move_data.disambiguation_file, None);
        assert_eq!(move_data.disambiguation_rank, None);
    }

    #[test]
    fn test_parse_piece_move() {
        let mut lex = PgnToken::lexer("Nf3");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, Piece::Knight);
        assert_eq!(move_data.to, Square::F3);
        assert_eq!(move_data.is_capture, false);
    }

    #[test]
    fn test_parse_capture_move() {
        let mut lex = PgnToken::lexer("Bxe5");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, Piece::Bishop);
        assert_eq!(move_data.to, Square::E5);
        assert_eq!(move_data.is_capture, true);
    }

    #[test]
    fn test_parse_pawn_capture() {
        let mut lex = PgnToken::lexer("exd5");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, Piece::Pawn);
        assert_eq!(move_data.to, Square::D5);
        assert_eq!(move_data.is_capture, true);
        assert_eq!(move_data.disambiguation_file, Some('e'));
    }

    #[test]
    fn test_parse_promotion() {
        let mut lex = PgnToken::lexer("e8=Q");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, Piece::Pawn);
        assert_eq!(move_data.to, Square::E8);
        assert_eq!(move_data.promoted_to, Piece::Queen);
    }

    #[test]
    fn test_parse_promotion_with_capture() {
        let mut lex = PgnToken::lexer("dxe8=Q+");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, Piece::Pawn);
        assert_eq!(move_data.to, Square::E8);
        assert_eq!(move_data.is_capture, true);
        assert_eq!(move_data.promoted_to, Piece::Queen);
        assert_eq!(move_data.common_move_info.is_check, true);
    }

    #[test]
    fn test_parse_with_disambiguation_file() {
        let mut lex = PgnToken::lexer("Rfe1");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, Piece::Rook);
        assert_eq!(move_data.to, Square::E1);
        assert_eq!(move_data.disambiguation_file, Some('f'));
        assert_eq!(move_data.disambiguation_rank, None);
    }

    #[test]
    fn test_parse_with_disambiguation_rank() {
        let mut lex = PgnToken::lexer("R2e1");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, Piece::Rook);
        assert_eq!(move_data.to, Square::E1);
        assert_eq!(move_data.disambiguation_file, None);
        assert_eq!(move_data.disambiguation_rank, Some('2'));
    }

    #[test]
    fn test_parse_with_both_disambiguation() {
        let mut lex = PgnToken::lexer("Qd5e4");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, Piece::Queen);
        assert_eq!(move_data.to, Square::E4);
        assert_eq!(move_data.disambiguation_file, Some('d'));
        assert_eq!(move_data.disambiguation_rank, Some('5'));
    }

    #[test]
    fn test_parse_with_check() {
        let mut lex = PgnToken::lexer("Qe4+");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, Piece::Queen);
        assert_eq!(move_data.to, Square::E4);
        assert_eq!(move_data.common_move_info.is_check, true);
        assert_eq!(move_data.common_move_info.is_checkmate, false);
    }

    #[test]
    fn test_parse_with_checkmate() {
        let mut lex = PgnToken::lexer("Qe4#");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, Piece::Queen);
        assert_eq!(move_data.to, Square::E4);
        assert_eq!(move_data.common_move_info.is_check, true);
        assert_eq!(move_data.common_move_info.is_checkmate, true);
    }

    #[test]
    fn test_parse_with_annotation() {
        let mut lex = PgnToken::lexer("Qe4!?");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, Piece::Queen);
        assert_eq!(move_data.to, Square::E4);
        assert_eq!(
            move_data.common_move_info.annotation,
            Some("!?".to_string())
        );
    }

    #[test]
    fn test_parse_with_nag() {
        let mut lex = PgnToken::lexer("Qe4 $1");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, Piece::Queen);
        assert_eq!(move_data.to, Square::E4);
        assert_eq!(move_data.common_move_info.nag, Some(1));
    }

    #[test]
    fn test_parse_complex_move() {
        let mut lex = PgnToken::lexer("Rd3xe3+!? $2");
        lex.next();
        let move_data = PgnNonCastlingMove::parse(&mut lex).unwrap();

        assert_eq!(move_data.piece_moved, Piece::Rook);
        assert_eq!(move_data.to, Square::E3);
        assert_eq!(move_data.is_capture, true);
        assert_eq!(move_data.disambiguation_file, Some('d'));
        assert_eq!(move_data.disambiguation_rank, Some('3'));
        assert_eq!(move_data.common_move_info.is_check, true);
        assert_eq!(
            move_data.common_move_info.annotation,
            Some("!?".to_string())
        );
        assert_eq!(move_data.common_move_info.nag, Some(2));
    }

    #[test]
    fn test_matches_move() {
        let state = Position::<1, { Color::White }>::from_fen(
            "r1bqkbnr/ppp2ppp/2np4/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 4",
        )
        .unwrap();

        // Test knight move
        let knight_move = PgnNonCastlingMove {
            piece_moved: Piece::Knight,
            disambiguation_file: None,
            disambiguation_rank: None,
            to: Square::D4,
            promoted_to: Piece::Null,
            is_capture: false,
            common_move_info: PgnCommonMoveInfo {
                is_check: false,
                is_checkmate: false,
                annotation: None,
                nag: None,
            },
        };

        let actual_move = Move::new_non_promotion(Square::F3, Square::D4, MoveFlag::NormalMove);
        assert!(knight_move.matches_move(actual_move, &state.board));

        // Test with disambiguation
        let knight_move_with_file = {
            let mut knight_move = knight_move.clone();
            knight_move.disambiguation_file = Some('f');
            knight_move
        };
        assert!(knight_move_with_file.matches_move(actual_move, &state.board));

        // Test with incorrect file disambiguation
        let knight_move_with_wrong_file = {
            let mut knight_move = knight_move.clone();
            knight_move.disambiguation_file = Some('e');
            knight_move
        };
        assert!(!knight_move_with_wrong_file.matches_move(actual_move, &state.board));
    }

    #[test]
    fn test_invalid_move() {
        let mut lex = PgnToken::lexer("Xx9");
        lex.next();
        let result = PgnNonCastlingMove::parse(&mut lex);
        assert!(result.is_err());
    }

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
        assert!(castling_move.is_err());
    }

    #[test]
    fn test_castling_matches_move() {
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
        assert!(kingside_match);
        assert!(!queenside_match);
    }
}
