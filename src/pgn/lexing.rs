use logos::{Logos, Lexer};
use regex::{Regex};
use crate::color::Color;
use crate::pgn::pgn_castling_move::PgnCastlingMove;
use crate::pgn::pgn_non_castling_move::PgnNonCastlingMove;
use crate::pgn::pgn_tag::PgnTag;
use crate::piece_type::PieceType;
use crate::square::Square;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[\s\t\n]+")]
pub enum PgnToken {
    // Tags [TagName "TagValue"]
    #[regex(r#"\[\s*([A-Za-z0-9_]+)\s+"[^"]*"\s*\]"#, PgnTag::parse)]
    Tag(PgnTag),

    // Move numbers like 1. or 1...
    #[regex(r"([0-9]+)\.+", parse_move_number)]
    MoveNumber(u16),

    // Moves like g4, Nf6, exd5+?!, etc.
    #[regex(r"[PNBRQK]?[a-h]?[1-8]?x?[a-h][1-8](=[NBRQ])?[+#]?[?!]*", PgnNonCastlingMove::parse)]
    NonCastlingMove(PgnNonCastlingMove),

    #[regex(r"((O-O(-O)?)|(0-0(-0)?))[+#]?[?!]*", PgnCastlingMove::parse)]
    CastlingMove(PgnCastlingMove),

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

        if let Some(Ok(PgnToken::Tag(PgnTag { name, value }))) = lex.next() {
            assert_eq!(name, "Event");
            assert_eq!(value, "Chess Championship");
        } else {
            panic!("Failed to parse Event tag");
        }

        if let Some(Ok(PgnToken::Tag(PgnTag { name, value }))) = lex.next() {
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

        if let Some(Ok(PgnToken::MoveNumber(num))) = lex.next() {
            assert_eq!(num, 1);
        } else {
            panic!("Failed to parse first move number");
        }

        if let Some(Ok(PgnToken::MoveNumber(num))) = lex.next() {
            assert_eq!(num, 1);
        } else {
            panic!("Failed to parse second move number");
        }

        if let Some(Ok(PgnToken::MoveNumber(num))) = lex.next() {
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
        if let Some(Ok(PgnToken::NonCastlingMove(PgnNonCastlingMove {
                        disambiguation_file,
                        disambiguation_rank,
                        to,
                        piece_moved,
                        promoted_to,
                        is_capture,
                        common_move_info
                    }))) = lex.next() {
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
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove { to, .. }))) = lex.next() {
            assert_eq!(to, Square::D5);
        } else {
            panic!("Failed to parse d5");
        }

        // exd5
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove {
                        disambiguation_file,
                        is_capture,
                        to,
                        ..
                    }))) = lex.next() {
            assert_eq!(disambiguation_file, Some('e'));
            assert_eq!(is_capture, true);
            assert_eq!(to, Square::D5);
        } else {
            panic!("Failed to parse exd5");
        }

        // c6
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove { to, .. }))) = lex.next() {
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
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove {
                        piece_moved,
                        to,
                        ..
                    }))) = lex.next() {
            assert_eq!(piece_moved, PieceType::Knight);
            assert_eq!(to, Square::F3);
        } else {
            panic!("Failed to parse Nf3");
        }

        // Bb4
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove {
                        piece_moved,
                        to,
                        ..
                    }))) = lex.next() {
            assert_eq!(piece_moved, PieceType::Bishop);
            assert_eq!(to, Square::B4);
        } else {
            panic!("Failed to parse Bb4");
        }

        // Qd8
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove {
                        piece_moved,
                        to,
                        ..
                    }))) = lex.next() {
            assert_eq!(piece_moved, PieceType::Queen);
            assert_eq!(to, Square::D8);
        } else {
            panic!("Failed to parse Qd8");
        }

        // Re1
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove {
                        piece_moved,
                        to,
                        ..
                    }))) = lex.next() {
            assert_eq!(piece_moved, PieceType::Rook);
            assert_eq!(to, Square::E1);
        } else {
            panic!("Failed to parse Re1");
        }

        // Ke2
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove {
                        piece_moved,
                        to,
                        ..
                    }))) = lex.next() {
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
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove {
                        disambiguation_file,
                        disambiguation_rank,
                        piece_moved,
                        to,
                        ..
                    }))) = lex.next() {
            assert_eq!(piece_moved, PieceType::Knight);
            assert_eq!(disambiguation_file, Some('b'));
            assert_eq!(disambiguation_rank, None);
            assert_eq!(to, Square::D7);
        } else {
            panic!("Failed to parse Nbd7");
        }

        // R1e2
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove {
                        disambiguation_file,
                        disambiguation_rank,
                        piece_moved,
                        to,
                        ..
                    }))) = lex.next() {
            assert_eq!(piece_moved, PieceType::Rook);
            assert_eq!(disambiguation_file, None);
            assert_eq!(disambiguation_rank, Some('1'));
            assert_eq!(to, Square::E2);
        } else {
            panic!("Failed to parse R1e2");
        }

        // Qh4g3 (this should be parsed as a queen move from h4 to g3)
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove {
                        disambiguation_file,
                        disambiguation_rank,
                        piece_moved,
                        to,
                        ..
                    }))) = lex.next() {
            assert_eq!(piece_moved, PieceType::Queen);
            assert_eq!(disambiguation_file, Some('h'));
            assert_eq!(disambiguation_rank, Some('4'));
            assert_eq!(to, Square::G3);
        } else {
            panic!("Failed to parse Qh4g3");
        }

        // Rdf8
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove {
                        disambiguation_file,
                        disambiguation_rank,
                        piece_moved,
                        to,
                        ..
                    }))) = lex.next() {
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
        if let Some(Ok(PgnToken::CastlingMove (PgnCastlingMove {
                        is_kingside,
                        common_move_info
                    }))) = lex.next() {
            assert_eq!(is_kingside, true);
            assert_eq!(common_move_info.is_check, false);
            assert_eq!(common_move_info.is_checkmate, false);
        } else {
            panic!("Failed to parse O-O");
        }

        // O-O-O
        if let Some(Ok(PgnToken::CastlingMove (PgnCastlingMove {
                        is_kingside,
                        ..
                    }))) = lex.next() {
            assert_eq!(is_kingside, false);
        } else {
            panic!("Failed to parse O-O-O");
        }

        // 0-0
        if let Some(Ok(PgnToken::CastlingMove (PgnCastlingMove {
                        is_kingside,
                        ..
                    }))) = lex.next() {
            assert_eq!(is_kingside, true);
        } else {
            panic!("Failed to parse 0-0");
        }

        // 0-0-0
        if let Some(Ok(PgnToken::CastlingMove (PgnCastlingMove {
                        is_kingside,
                        ..
                    }))) = lex.next() {
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
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove {
                        common_move_info,
                        ..
                    }))) = lex.next() {
            assert_eq!(common_move_info.is_check, true);
            assert_eq!(common_move_info.is_checkmate, false);
        } else {
            panic!("Failed to parse e4+");
        }

        // Nf6#
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove {
                        common_move_info,
                        ..
                    }))) = lex.next() {
            assert_eq!(common_move_info.is_check, true);
            assert_eq!(common_move_info.is_checkmate, true);
        } else {
            panic!("Failed to parse Nf6#");
        }

        // Qxd7+
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove {
                        is_capture,
                        common_move_info,
                        ..
                    }))) = lex.next() {
            assert_eq!(is_capture, true);
            assert_eq!(common_move_info.is_check, true);
            assert_eq!(common_move_info.is_checkmate, false);
        } else {
            panic!("Failed to parse Qxd7+");
        }

        // Kxd7#
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove {
                        is_capture,
                        common_move_info,
                        ..
                    }))) = lex.next() {
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
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove {
                        to,
                        piece_moved,
                        promoted_to,
                        ..
                    }))) = lex.next() {
            assert_eq!(to, Square::E8);
            assert_eq!(piece_moved, PieceType::Pawn);
            assert_eq!(promoted_to, PieceType::Queen);
        } else {
            panic!("Failed to parse e8=Q");
        }

        // d1=N
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove {
                        to,
                        promoted_to,
                        ..
                    }))) = lex.next() {
            assert_eq!(to, Square::D1);
            assert_eq!(promoted_to, PieceType::Knight);
        } else {
            panic!("Failed to parse d1=N");
        }

        // axb8=R
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove {
                        disambiguation_file,
                        is_capture,
                        to,
                        promoted_to,
                        ..
                    }))) = lex.next() {
            assert_eq!(disambiguation_file, Some('a'));
            assert_eq!(is_capture, true);
            assert_eq!(to, Square::B8);
            assert_eq!(promoted_to, PieceType::Rook);
        } else {
            panic!("Failed to parse axb8=R");
        }

        // c1=B
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove {
                        to,
                        promoted_to,
                        ..
                    }))) = lex.next() {
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
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove {
                        common_move_info,
                        ..
                    }))) = lex.next() {
            assert_eq!(common_move_info.annotation, Some("!".to_string()));
        } else {
            panic!("Failed to parse e4!");
        }

        // d5?
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove {
                        common_move_info,
                        ..
                    }))) = lex.next() {
            assert_eq!(common_move_info.annotation, Some("?".to_string()));
        } else {
            panic!("Failed to parse d5?");
        }

        // Nf3!?
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove {
                        common_move_info,
                        ..
                    }))) = lex.next() {
            assert_eq!(common_move_info.annotation, Some("!?".to_string()));
        } else {
            panic!("Failed to parse Nf3!?");
        }

        // Nc6?!
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove {
                        common_move_info,
                        ..
                    }))) = lex.next() {
            assert_eq!(common_move_info.annotation, Some("?!".to_string()));
        } else {
            panic!("Failed to parse Nc6?!");
        }

        // O-O!!
        if let Some(Ok(PgnToken::CastlingMove (PgnCastlingMove {
                        common_move_info,
                        ..
                    }))) = lex.next() {
            assert_eq!(common_move_info.annotation, Some("!!".to_string()));
        } else {
            panic!("Failed to parse O-O!!");
        }

        // e5??
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove {
                        common_move_info,
                        ..
                    }))) = lex.next() {
            assert_eq!(common_move_info.annotation, Some("??".to_string()));
        } else {
            panic!("Failed to parse e5??");
        }

        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_comments() {
        let mut lex = PgnToken::lexer("{Simple comment} {Multiple\nline\ncomment}");

        if let Some(Ok(PgnToken::Comment(text))) = lex.next() {
            assert_eq!(text, "Simple comment");
        } else {
            panic!("Failed to parse first comment");
        }

        if let Some(Ok(PgnToken::Comment(text))) = lex.next() {
            assert_eq!(text, "Multiple\nline\ncomment");
        } else {
            panic!("Failed to parse second comment");
        }

        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_nag() {
        let mut lex = PgnToken::lexer("$1 $12 $42");

        if let Some(Ok(PgnToken::Nag(num))) = lex.next() {
            assert_eq!(num, 1);
        } else {
            panic!("Failed to parse $1");
        }

        if let Some(Ok(PgnToken::Nag(num))) = lex.next() {
            assert_eq!(num, 12);
        } else {
            panic!("Failed to parse $12");
        }

        if let Some(Ok(PgnToken::Nag(num))) = lex.next() {
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
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove { to, .. }))) = lex.next() {
            assert_eq!(to, Square::E4);
        } else {
            panic!("Failed to parse e4");
        }

        // e5
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove { to, .. }))) = lex.next() {
            assert_eq!(to, Square::E5);
        } else {
            panic!("Failed to parse e5");
        }

        // (
        if let Some(Ok(PgnToken::StartVariation)) = lex.next() {
            // success
        } else {
            panic!("Failed to parse start variation");
        }

        // d5
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove { to, .. }))) = lex.next() {
            assert_eq!(to, Square::D5);
        } else {
            panic!("Failed to parse d5");
        }

        // Nf3
        if let Some(Ok(PgnToken::NonCastlingMove (PgnNonCastlingMove { to, .. }))) = lex.next() {
            assert_eq!(to, Square::F3);
        } else {
            panic!("Failed to parse Nf3");
        }

        // )
        if let Some(Ok(PgnToken::EndVariation)) = lex.next() {
            // success
        } else {
            panic!("Failed to parse end variation");
        }

        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_game_results() {
        let mut lex = PgnToken::lexer("1-0 0-1 1/2-1/2 *");

        if let Some(Ok(PgnToken::Result(winner))) = lex.next() {
            assert_eq!(winner, Some(Color::White));
        } else {
            panic!("Failed to parse 1-0");
        }

        if let Some(Ok(PgnToken::Result(winner))) = lex.next() {
            assert_eq!(winner, Some(Color::Black));
        } else {
            panic!("Failed to parse 0-1");
        }

        if let Some(Ok(PgnToken::Result(winner))) = lex.next() {
            assert_eq!(winner, None); // Draw
        } else {
            panic!("Failed to parse 1/2-1/2");
        }

        if let Some(Ok(PgnToken::Incomplete)) = lex.next() {
            // success
        } else {
            panic!("Failed to parse *");
        }

        assert_eq!(lex.next(), None);
    }
}