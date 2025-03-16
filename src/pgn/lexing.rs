use logos::{Logos, Lexer};
use regex::{Match, Regex};
use crate::color::Color;
use crate::piece_type::PieceType;
use crate::r#move::{Move, MoveFlag};
use crate::square::Square;
use crate::state::State;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CommonMoveInfo {
    pub is_check: bool,
    pub is_checkmate: bool,
    pub annotation: Option<String>
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Tag {
    pub name: String,
    pub value: String
}

impl Tag {
    pub fn parse(lex: &mut Lexer<PgnToken>) -> Option<Tag> {
        let text = lex.slice();
        let tag_regex = Regex::new(r#"\[\s*([A-Za-z0-9_]+)\s+"([^"]*)"\s*\]"#).unwrap();

        if let Some(captures) = tag_regex.captures(text) {
            let name = captures.get(1).unwrap().as_str().to_string();
            let value = captures.get(2).unwrap().as_str().to_string();
            return Some(Tag { name, value });
        }

        None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NonCastlingMove {
    pub disambiguation_file: Option<char>,
    pub disambiguation_rank: Option<char>,
    pub to: Square,
    pub piece_moved: PieceType,
    pub promoted_to: PieceType,
    pub is_capture: bool,
    pub common_move_info: CommonMoveInfo
}

impl NonCastlingMove {
    pub fn parse(lex: &mut Lexer<PgnToken>) -> Option<NonCastlingMove> {
        let text = lex.slice();
        let move_regex = Regex::new(r"([PNBRQK]?)([a-h]?)([1-8]?)(x?)([a-h])([1-8])(=[NBRQ])?([+#])?([?!]*)").unwrap();
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

            Some(
                NonCastlingMove {
                    disambiguation_file,
                    disambiguation_rank,
                    to,
                    piece_moved,
                    promoted_to,
                    is_capture,
                    common_move_info: CommonMoveInfo::from(captures.get(8), captures.get(9))
                }
            )
        } else {
            None
        }
    }

    pub fn matches_move(&self, mv: &Move, initial_state: &State) -> bool {
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
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CastlingMove {
    pub is_kingside: bool,
    pub common_move_info: CommonMoveInfo
}

impl CastlingMove {
    pub fn parse(lex: &mut Lexer<PgnToken>) -> Option<CastlingMove> {
        let text = lex.slice();
        let move_regex = Regex::new(r"(O-O(-O)?)|(0-0(-0)?)([+#])?([?!]*)").unwrap();
        if let Some(captures) = move_regex.captures(text) {
            let is_kingside = captures.get(1).is_some();

            Some(
                CastlingMove {
                    is_kingside,
                    common_move_info: CommonMoveInfo::from(captures.get(4), captures.get(5))
                }
            )
        } else {
            None
        }
    }

    pub fn matches_move(&self, mv: &Move, initial_state: &State) -> bool {
        let flag = mv.get_flag();
        if flag != MoveFlag::Castling {
            return false
        } else if self.is_kingside != (mv.get_destination().get_file() == 6) {
            return false
        }

        true
    }
}

impl CommonMoveInfo {
    fn from(check_or_checkmate: Option<Match>, annotation: Option<Match>) -> CommonMoveInfo {
        let (is_check, is_checkmate) = match check_or_checkmate {
            Some(m) => {
                let check_or_checkmate_char = m.as_str().chars().next().unwrap();
                (true, check_or_checkmate_char == '#')
            },
            None => (false, false)
        };

        CommonMoveInfo { is_check, is_checkmate, annotation: annotation.map(|m| m.as_str().to_string()) }
    }
}

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[\s\t\n]+")]
pub enum PgnToken {
    // Tags [TagName "TagValue"]
    #[regex(r#"\[\s*([A-Za-z0-9_]+)\s+"[^"]*"\s*\]"#, Tag::parse)]
    Tag(Tag),

    // Move numbers like 1. or 1...
    #[regex(r"([0-9]+)\.+", parse_move_number)]
    MoveNumber(u16),

    // Moves like g4, Nf6, exd5+?!, etc.
    #[regex(r"[PNBRQK]?[a-h]?[1-8]?x?[a-h][1-8](=[NBRQ])?[+#]?[?!]*", NonCastlingMove::parse)]
    NonCastlingMove(NonCastlingMove),

    #[regex(r"((O-O(-O)?)|(0-0(-0)?))[+#]?[?!]*", CastlingMove::parse)]
    CastlingMove(CastlingMove),

    // Comments in { }
    #[regex(r"\{[^}]*\}", parse_comment)]
    Comment(String), // Regular comment

    // NAG (Numeric Annotation Glyph)
    #[regex(r"\$([0-9]+)", parse_nag)]
    Nag(u8),

    // Start of variation
    #[token("(")]
    StartVariation,

    // End of variation
    #[token(")")]
    EndVariation,

    #[token("1-0", |_| Some(Color::White))]
    #[token("0-1", |_| Some(Color::Black))]
    #[token("1/2-1/2", |_| None::<Color>)]
    Result(Option<Color>),

    #[token("*")]
    Incomplete
}

fn parse_move_number(lex: &mut Lexer<PgnToken>) -> Option<u16> {
    let text = lex.slice();
    let number_regex = Regex::new(r"[0-9]+").unwrap();

    match number_regex.find(text) {
        Some(m) => m.as_str().parse::<u16>().ok(),
        None => None
    }
}

fn parse_comment(lex: &mut Lexer<PgnToken>) -> Option<String> {
    let text = lex.slice();
    let comment_regex = Regex::new(r"\{([^}]*)\}").unwrap();

    match comment_regex.captures(text) {
        Some(captures) => Some(captures.get(1).unwrap().as_str().to_string()),
        None => None
    }
}

fn parse_nag(lex: &mut Lexer<PgnToken>) -> Option<u8> {
    let text = lex.slice();
    let nag_regex = Regex::new(r"\$([0-9]+)").unwrap();

    match nag_regex.captures(text) {
        Some(captures) => captures.get(1).unwrap().as_str().parse::<u8>().ok(),
        None => None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos::Logos;

    #[test]
    fn test_tag_parsing() {
        let mut lex = PgnToken::lexer(r#"[Event "Chess Championship"] [Site "New York, USA"]"#);

        if let Some(PgnToken::Tag(Tag { name, value })) = lex.next() {
            assert_eq!(name, "Event");
            assert_eq!(value, "Chess Championship");
        } else {
            panic!("Failed to parse Event tag");
        }

        if let Some(PgnToken::Tag(Tag { name, value })) = lex.next() {
            assert_eq!(name, "Site");
            assert_eq!(value, "New York, USA");
        } else {
            panic!("Failed to parse Site tag");
        }

        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_move_numbers() {
        let mut lex = PgnToken::lexer("1. 1... 24...");

        if let Some(PgnToken::MoveNumber(num)) = lex.next() {
            assert_eq!(num, 1);
        } else {
            panic!("Failed to parse first move number");
        }

        if let Some(PgnToken::MoveNumber(num)) = lex.next() {
            assert_eq!(num, 1);
        } else {
            panic!("Failed to parse second move number");
        }

        if let Some(PgnToken::MoveNumber(num)) = lex.next() {
            assert_eq!(num, 24);
        } else {
            panic!("Failed to parse third move number");
        }

        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_pawn_moves() {
        let mut lex = PgnToken::lexer("e4 d5 exd5 c6");

        // e4
        if let Some(PgnToken::NonCastlingMove(NonCastlingMove {
                        disambiguation_file,
                        disambiguation_rank,
                        to,
                        piece_moved,
                        promoted_to,
                        is_capture,
                        common_move_info
                    })) = lex.next() {
            assert_eq!(disambiguation_file, None);
            assert_eq!(disambiguation_rank, None);
            assert_eq!(to, Square::E4);
            assert_eq!(piece_moved, PieceType::Pawn);
            assert_eq!(promoted_to, PieceType::NoPieceType);
            assert_eq!(is_capture, false);
            assert_eq!(common_move_info.is_check, false);
            assert_eq!(common_move_info.is_checkmate, false);
            assert_eq!(common_move_info.annotation, None);
        } else {
            panic!("Failed to parse e4");
        }

        // d5
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove { to, .. })) = lex.next() {
            assert_eq!(to, Square::D5);
        } else {
            panic!("Failed to parse d5");
        }

        // exd5
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove {
                        disambiguation_file,
                        is_capture,
                        to,
                        ..
                    })) = lex.next() {
            assert_eq!(disambiguation_file, Some('e'));
            assert_eq!(is_capture, true);
            assert_eq!(to, Square::D5);
        } else {
            panic!("Failed to parse exd5");
        }

        // c6
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove { to, .. })) = lex.next() {
            assert_eq!(to, Square::C6);
        } else {
            panic!("Failed to parse c6");
        }

        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_piece_moves() {
        let mut lex = PgnToken::lexer("Nf3 Bb4 Qd8 Re1 Ke2");

        // Nf3
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove {
                        piece_moved,
                        to,
                        ..
                    })) = lex.next() {
            assert_eq!(piece_moved, PieceType::Knight);
            assert_eq!(to, Square::F3);
        } else {
            panic!("Failed to parse Nf3");
        }

        // Bb4
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove {
                        piece_moved,
                        to,
                        ..
                    })) = lex.next() {
            assert_eq!(piece_moved, PieceType::Bishop);
            assert_eq!(to, Square::B4);
        } else {
            panic!("Failed to parse Bb4");
        }

        // Qd8
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove {
                        piece_moved,
                        to,
                        ..
                    })) = lex.next() {
            assert_eq!(piece_moved, PieceType::Queen);
            assert_eq!(to, Square::D8);
        } else {
            panic!("Failed to parse Qd8");
        }

        // Re1
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove {
                        piece_moved,
                        to,
                        ..
                    })) = lex.next() {
            assert_eq!(piece_moved, PieceType::Rook);
            assert_eq!(to, Square::E1);
        } else {
            panic!("Failed to parse Re1");
        }

        // Ke2
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove {
                        piece_moved,
                        to,
                        ..
                    })) = lex.next() {
            assert_eq!(piece_moved, PieceType::King);
            assert_eq!(to, Square::E2);
        } else {
            panic!("Failed to parse Ke2");
        }

        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_disambiguation() {
        let mut lex = PgnToken::lexer("Nbd7 R1e2 Qh4g3 Rdf8");

        // Nbd7
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove {
                        disambiguation_file,
                        disambiguation_rank,
                        piece_moved,
                        to,
                        ..
                    })) = lex.next() {
            assert_eq!(piece_moved, PieceType::Knight);
            assert_eq!(disambiguation_file, Some('b'));
            assert_eq!(disambiguation_rank, None);
            assert_eq!(to, Square::D7);
        } else {
            panic!("Failed to parse Nbd7");
        }

        // R1e2
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove {
                        disambiguation_file,
                        disambiguation_rank,
                        piece_moved,
                        to,
                        ..
                    })) = lex.next() {
            assert_eq!(piece_moved, PieceType::Rook);
            assert_eq!(disambiguation_file, None);
            assert_eq!(disambiguation_rank, Some('1'));
            assert_eq!(to, Square::E2);
        } else {
            panic!("Failed to parse R1e2");
        }

        // Qh4g3 (this should be parsed as a queen move from h4 to g3)
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove {
                        disambiguation_file,
                        disambiguation_rank,
                        piece_moved,
                        to,
                        ..
                    })) = lex.next() {
            assert_eq!(piece_moved, PieceType::Queen);
            assert_eq!(disambiguation_file, Some('h'));
            assert_eq!(disambiguation_rank, Some('4'));
            assert_eq!(to, Square::G3);
        } else {
            panic!("Failed to parse Qh4g3");
        }

        // Rdf8
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove {
                        disambiguation_file,
                        disambiguation_rank,
                        piece_moved,
                        to,
                        ..
                    })) = lex.next() {
            assert_eq!(piece_moved, PieceType::Rook);
            assert_eq!(disambiguation_file, Some('d'));
            assert_eq!(disambiguation_rank, None);
            assert_eq!(to, Square::F8);
        } else {
            panic!("Failed to parse Rdf8");
        }

        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_castling() {
        let mut lex = PgnToken::lexer("O-O O-O-O 0-0 0-0-0");

        // O-O
        if let Some(PgnToken::CastlingMove (CastlingMove {
                        is_kingside,
                        common_move_info
                    })) = lex.next() {
            assert_eq!(is_kingside, true);
            assert_eq!(common_move_info.is_check, false);
            assert_eq!(common_move_info.is_checkmate, false);
        } else {
            panic!("Failed to parse O-O");
        }

        // O-O-O
        if let Some(PgnToken::CastlingMove (CastlingMove {
                        is_kingside,
                        ..
                    })) = lex.next() {
            assert_eq!(is_kingside, false);
        } else {
            panic!("Failed to parse O-O-O");
        }

        // 0-0
        if let Some(PgnToken::CastlingMove (CastlingMove {
                        is_kingside,
                        ..
                    })) = lex.next() {
            assert_eq!(is_kingside, true);
        } else {
            panic!("Failed to parse 0-0");
        }

        // 0-0-0
        if let Some(PgnToken::CastlingMove (CastlingMove {
                        is_kingside,
                        ..
                    })) = lex.next() {
            assert_eq!(is_kingside, false);
        } else {
            panic!("Failed to parse 0-0-0");
        }

        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_check_and_mate() {
        let mut lex = PgnToken::lexer("e4+ Nf6# Qxd7+ Kxd7#");

        // e4+
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove {
                        common_move_info,
                        ..
                    })) = lex.next() {
            assert_eq!(common_move_info.is_check, true);
            assert_eq!(common_move_info.is_checkmate, false);
        } else {
            panic!("Failed to parse e4+");
        }

        // Nf6#
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove {
                        common_move_info,
                        ..
                    })) = lex.next() {
            assert_eq!(common_move_info.is_check, true);
            assert_eq!(common_move_info.is_checkmate, true);
        } else {
            panic!("Failed to parse Nf6#");
        }

        // Qxd7+
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove {
                        is_capture,
                        common_move_info,
                        ..
                    })) = lex.next() {
            assert_eq!(is_capture, true);
            assert_eq!(common_move_info.is_check, true);
            assert_eq!(common_move_info.is_checkmate, false);
        } else {
            panic!("Failed to parse Qxd7+");
        }

        // Kxd7#
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove {
                        is_capture,
                        common_move_info,
                        ..
                    })) = lex.next() {
            assert_eq!(is_capture, true);
            assert_eq!(common_move_info.is_check, true);
            assert_eq!(common_move_info.is_checkmate, true);
        } else {
            panic!("Failed to parse Kxd7#");
        }

        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_promotion() {
        let mut lex = PgnToken::lexer("e8=Q d1=N axb8=R c1=B");

        // e8=Q
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove {
                        to,
                        piece_moved,
                        promoted_to,
                        ..
                    })) = lex.next() {
            assert_eq!(to, Square::E8);
            assert_eq!(piece_moved, PieceType::Pawn);
            assert_eq!(promoted_to, PieceType::Queen);
        } else {
            panic!("Failed to parse e8=Q");
        }

        // d1=N
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove {
                        to,
                        promoted_to,
                        ..
                    })) = lex.next() {
            assert_eq!(to, Square::D1);
            assert_eq!(promoted_to, PieceType::Knight);
        } else {
            panic!("Failed to parse d1=N");
        }

        // axb8=R
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove {
                        disambiguation_file,
                        is_capture,
                        to,
                        promoted_to,
                        ..
                    })) = lex.next() {
            assert_eq!(disambiguation_file, Some('a'));
            assert_eq!(is_capture, true);
            assert_eq!(to, Square::B8);
            assert_eq!(promoted_to, PieceType::Rook);
        } else {
            panic!("Failed to parse axb8=R");
        }

        // c1=B
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove {
                        to,
                        promoted_to,
                        ..
                    })) = lex.next() {
            assert_eq!(to, Square::C1);
            assert_eq!(promoted_to, PieceType::Bishop);
        } else {
            panic!("Failed to parse c1=B");
        }

        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_annotation_symbols() {
        let mut lex = PgnToken::lexer("e4! d5? Nf3!? Nc6?! O-O!! e5??");

        // e4!
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove {
                        common_move_info,
                        ..
                    })) = lex.next() {
            assert_eq!(common_move_info.annotation, Some("!".to_string()));
        } else {
            panic!("Failed to parse e4!");
        }

        // d5?
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove {
                        common_move_info,
                        ..
                    })) = lex.next() {
            assert_eq!(common_move_info.annotation, Some("?".to_string()));
        } else {
            panic!("Failed to parse d5?");
        }

        // Nf3!?
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove {
                        common_move_info,
                        ..
                    })) = lex.next() {
            assert_eq!(common_move_info.annotation, Some("!?".to_string()));
        } else {
            panic!("Failed to parse Nf3!?");
        }

        // Nc6?!
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove {
                        common_move_info,
                        ..
                    })) = lex.next() {
            assert_eq!(common_move_info.annotation, Some("?!".to_string()));
        } else {
            panic!("Failed to parse Nc6?!");
        }

        // O-O!!
        if let Some(PgnToken::CastlingMove (NonCastlingMove {
                        common_move_info,
                        ..
                    })) = lex.next() {
            assert_eq!(common_move_info.annotation, Some("!!".to_string()));
        } else {
            panic!("Failed to parse O-O!!");
        }

        // e5??
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove {
                        common_move_info,
                        ..
                    })) = lex.next() {
            assert_eq!(common_move_info.annotation, Some("??".to_string()));
        } else {
            panic!("Failed to parse e5??");
        }

        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_comments() {
        let mut lex = PgnToken::lexer("{Simple comment} {Multiple\nline\ncomment}");

        if let Some(PgnToken::Comment(text)) = lex.next() {
            assert_eq!(text, "Simple comment");
        } else {
            panic!("Failed to parse first comment");
        }

        if let Some(PgnToken::Comment(text)) = lex.next() {
            assert_eq!(text, "Multiple\nline\ncomment");
        } else {
            panic!("Failed to parse second comment");
        }

        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_nag() {
        let mut lex = PgnToken::lexer("$1 $12 $42");

        if let Some(PgnToken::Nag(num)) = lex.next() {
            assert_eq!(num, 1);
        } else {
            panic!("Failed to parse $1");
        }

        if let Some(PgnToken::Nag(num)) = lex.next() {
            assert_eq!(num, 12);
        } else {
            panic!("Failed to parse $12");
        }

        if let Some(PgnToken::Nag(num)) = lex.next() {
            assert_eq!(num, 42);
        } else {
            panic!("Failed to parse $42");
        }

        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_variations() {
        let mut lex = PgnToken::lexer("e4 e5 (d5 Nf3)");

        // e4
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove { to, .. })) = lex.next().unwrap() {
            assert_eq!(to, Square::E4);
        } else {
            panic!("Failed to parse e4");
        }

        // e5
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove { to, .. })) = lex.next().unwrap() {
            assert_eq!(to, Square::E5);
        } else {
            panic!("Failed to parse e5");
        }

        // (
        if let Some(PgnToken::StartVariation) = lex.next().unwrap() {
            // success
        } else {
            panic!("Failed to parse start variation");
        }

        // d5
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove { to, .. })) = lex.next() {
            assert_eq!(to, Square::D5);
        } else {
            panic!("Failed to parse d5");
        }

        // Nf3
        if let Some(PgnToken::NonCastlingMove (NonCastlingMove { to, .. })) = lex.next() {
            assert_eq!(to, Square::F3);
        } else {
            panic!("Failed to parse Nf3");
        }

        // )
        if let Some(PgnToken::EndVariation) = lex.next().unwrap() {
            // success
        } else {
            panic!("Failed to parse end variation");
        }

        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_game_results() {
        let mut lex = PgnToken::lexer("1-0 0-1 1/2-1/2 *");

        if let Some(PgnToken::Result(winner)) = lex.next() {
            assert_eq!(winner, Some(Color::White));
        } else {
            panic!("Failed to parse 1-0");
        }

        if let Some(PgnToken::Result(winner)) = lex.next() {
            assert_eq!(winner, Some(Color::Black));
        } else {
            panic!("Failed to parse 0-1");
        }

        if let Some(PgnToken::Result(winner)) = lex.next() {
            assert_eq!(winner, None); // Draw
        } else {
            panic!("Failed to parse 1/2-1/2");
        }

        if let Some(PgnToken::Incomplete) = lex.next().unwrap() {
            // success
        } else {
            panic!("Failed to parse *");
        }

        assert_eq!(lex.next(), None);
    }
}