use logos::{Logos, Lexer};
use crate::utils::Color;
use crate::pgn::lexing_error::PgnLexingError;
use crate::pgn::token_types::PgnCastlingMove;
use crate::pgn::token_types::PgnComment;
use crate::pgn::token_types::PgnMoveNumber;
use crate::pgn::token_types::PgnNonCastlingMove;
use crate::pgn::token_types::PgnTag;

pub trait ParsablePgnToken: Sized {
    fn parse(lex: &mut Lexer<PgnToken>) -> Result<Self, PgnLexingError>;
}

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"\s+")]
#[logos(error = PgnLexingError)]
pub enum PgnToken {
    // Tags [Name "Value"]
    #[regex(r#"\[\s*([A-Za-z0-9_]+)\s+"([^"]*)"\s*\]"#, PgnTag::parse)]
    Tag(PgnTag),

    // Move numbers like 1. or 1...
    #[regex(r"([0-9]+)\.+", PgnMoveNumber::parse)]
    MoveNumber(PgnMoveNumber),

    // Moves like g4, Nf6, exd5+?!, etc.
    #[regex(r"([PNBRQK])?([a-h])?([1-8])?(x)?([a-h])([1-8])(?:=([NBRQ]))?([+#])?([?!]*)\s*(?:\$([0-9]+))?", PgnNonCastlingMove::parse)]
    NonCastlingMove(PgnNonCastlingMove),

    #[regex(r"(?:(O-O-O|0-0-0)|(O-O|0-0))([+#])?([?!]+)?\s*(?:\$([0-9]+))?", PgnCastlingMove::parse)]
    CastlingMove(PgnCastlingMove),

    // Comments in { }
    #[regex(r"\{([^}]*)\}", PgnComment::parse)]
    Comment(PgnComment),

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::PieceType;
    use crate::utils::Square;

    #[test]
    fn test_lexing_variations() {
        let mut lexer = PgnToken::lexer("(");
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::StartVariation))));

        let mut lexer = PgnToken::lexer(")");
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::EndVariation))));
    }

    #[test]
    fn test_lexing_results() {
        let mut lexer = PgnToken::lexer("1-0");
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::Result(Some(Color::White))))));

        let mut lexer = PgnToken::lexer("0-1");
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::Result(Some(Color::Black))))));

        let mut lexer = PgnToken::lexer("1/2-1/2");
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::Result(None)))));
    }

    #[test]
    fn test_lexing_incomplete() {
        let mut lexer = PgnToken::lexer("*");
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::Incomplete))));
    }

    #[test]
    fn test_lexing_sequence() {
        let pgn = r#"[Event "World Championship"]
                    [Site "London, England"]
                    1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 {The Ruy Lopez} 4. Ba4 Nf6 5. O-O *"#;

        let mut lexer = PgnToken::lexer(pgn);

        // Tags
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::Tag(tag))) if tag.name == "Event" && tag.value == "World Championship"));
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::Tag(tag))) if tag.name == "Site" && tag.value == "London, England"));

        // First move
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::MoveNumber(num))) if num.fullmove_number == 1));
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::NonCastlingMove(mv))) if
            mv.piece_moved == PieceType::Pawn && mv.to == Square::E4
        ));

        // First black move
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::NonCastlingMove(mv))) if
            mv.piece_moved == PieceType::Pawn && mv.to == Square::E5
        ));

        // Second move
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::MoveNumber(num))) if num.fullmove_number == 2));
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::NonCastlingMove(mv))) if
            mv.piece_moved == PieceType::Knight && mv.to == Square::F3
        ));

        // Second black move
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::NonCastlingMove(mv))) if
            mv.piece_moved == PieceType::Knight && mv.to == Square::C6
        ));

        // Third move
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::MoveNumber(num))) if num.fullmove_number == 3));
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::NonCastlingMove(mv))) if
            mv.piece_moved == PieceType::Bishop && mv.to == Square::B5
        ));

        // Third black move
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::NonCastlingMove(mv))) if
            mv.piece_moved == PieceType::Pawn && mv.to == Square::A6
        ));

        // Comment
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::Comment(comment))) if
            comment.comment == "The Ruy Lopez".to_string()
        ));

        // Fourth move
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::MoveNumber(num))) if num.fullmove_number == 4));
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::NonCastlingMove(mv))) if
            mv.piece_moved == PieceType::Bishop && mv.to == Square::A4
        ));

        // Fourth black move
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::NonCastlingMove(mv))) if
            mv.piece_moved == PieceType::Knight && mv.to == Square::F6
        ));

        // Fifth move
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::MoveNumber(num))) if num.fullmove_number == 5));
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::CastlingMove(mv))) if mv.is_kingside));

        // Result
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::Incomplete))));
    }

    #[test]
    fn test_lexing_with_variations() {
        let pgn = "1. e4 e5 2. Nf3 (2. f4 exf4 3. Bc4) 2... Nc6 3. Bb5";

        let mut lexer = PgnToken::lexer(pgn);

        // First move and response
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::MoveNumber(num))) if num.fullmove_number == 1));
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::NonCastlingMove(_)))));
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::NonCastlingMove(_)))));

        // Second move
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::MoveNumber(num))) if num.fullmove_number == 2));
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::NonCastlingMove(_)))));

        // Variation start
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::StartVariation))));

        // Alternative second move
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::MoveNumber(num))) if num.fullmove_number == 2));
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::NonCastlingMove(_)))));

        // Response in variation
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::NonCastlingMove(_)))));

        // Third move in variation
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::MoveNumber(num))) if num.fullmove_number == 3));
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::NonCastlingMove(_)))));

        // Variation end
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::EndVariation))));

        // Second black move in main line
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::MoveNumber(num))) if num.fullmove_number == 2));
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::NonCastlingMove(_)))));

        // Third move in main line
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::MoveNumber(num))) if num.fullmove_number == 3));
        assert!(matches!(lexer.next(), Some(Ok(PgnToken::NonCastlingMove(_)))));
    }

    #[test]
    fn test_error_handling() {
        // Invalid Tag
        let mut lexer = PgnToken::lexer(r#"[Event "Missing closing bracket"#);
        assert!(matches!(lexer.next(), Some(Err(_))));

        // Invalid move
        let mut lexer = PgnToken::lexer("X9");
        assert!(matches!(lexer.next(), Some(Err(_))));
    }

    #[test]
    fn test_complex_pgn() {
        let pgn = r#"[Event "F/S Return Match"]
                     [Site "Belgrade, Serbia JUG"]
                     [Date "1992.11.04"]
                     [Round "29"]
                     [White "Fischer, Robert J."]
                     [Black "Spassky, Boris V."]
                     [Result "1/2-1/2"]

                     1. e4 e5 2. Nf3 Nc6 3. Bb5 {This opening is called the Ruy Lopez.}
                     3... a6 4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O
                     9. h3 Nb8 10. d4 Nbd7 11. c4 c6 12. cxb5 axb5 13. Nc3 Bb7
                     14. Bg5 b4 15. Nb1 h6 16. Bh4 c5 17. dxe5 Nxe4 18. Bxe7 Qxe7
                     19. exd6 Qf6 20. Nbd2 Nxd6 21. Nc4 Nxc4 22. Bxc4 Nb6
                     23. Ne5 Rae8 24. Bxf7+ Rxf7 25. Nxf7 Rxe1+ 26. Qxe1 Kxf7
                     27. Qe3 Qg5 28. Qxg5 hxg5 29. b3 Ke6 30. a3 Kd6 31. axb4 cxb4
                     32. Ra5 Nd5 33. f3 Bc8 34. Kf2 Bf5 35. Ra7 g6 36. Ra6+ Kc5
                     37. Ke1 Nf4 38. g3 Nxh3 39. Kd2 Kb5 40. Rd6 Kc5 41. Ra6 Nf2
                     42. g4 Bd3 43. Re6 1/2-1/2"#;

        let mut lexer = PgnToken::lexer(pgn);
        let mut token_count = 0;

        // Just count the tokens to make sure we can lex the entire game without errors
        while let Some(token) = lexer.next() {
            token.expect("Failed to lex token");
            token_count += 1;
        }

        // This complex PGN should have at least 100 tokens
        assert!(token_count > 100, "Expected more than 100 tokens, got {}", token_count);
    }
}