use logos::Logos;
use crate::pgn::error::PgnParseError;

/// Represents a token in a PGN string
#[derive(Debug, PartialEq, Clone)]
pub enum PgnToken {
    Tag(String),                       // Represents a tag (e.g., "[Event "F/S Return Match"]")
    Move(String),                      // Represents a move (e.g., "e4", "Nf3#")
    MoveNumberAndPeriods(u16, usize),  // Represents a move number and its periods (e.g., "1.", "2...")
    StartVariation,                    // Represents the start of a variation ('(')
    EndVariation,                      // Represents the end of a variation (')')
    Comment(String),                   // Represents a comment (e.g., "{This is a comment}")
    Annotation(String),                // Represents an annotation (e.g., "!", "?", "!?", etc.)
    Result(String),                    // Represents a game result (e.g., "1-0", "0-1", "1/2-1/2", "*")
}

/// Logos token type for internal lexer implementation
#[derive(Logos, Debug, PartialEq)]
enum LogosToken<'a> {
    // Tags are enclosed in square brackets
    #[regex(r"\[[^\]]*\]")]
    Tag(&'a str),

    // Move notations
    #[regex(r"([RNBQKP])?[a-h]?[1-8]?x?[a-h][1-8](=[RNBQK])?[+#]?")]
    #[regex(r"([RNBQKP])[a-h][1-8]x?[a-h][1-8](=[RNBQK])?[+#]?")]
    #[regex(r"([RNBQK])[a-h1-8]?x?[a-h][1-8][+#]?")]
    #[regex(r"O-O(-O)?[+#]?")]
    Move(&'a str),

    // Move numbers and dots (e.g., "1.", "2...", etc.)
    #[regex(r"[0-9]+\.+")]
    MoveNumber(&'a str),

    // Special characters for variations
    #[token("(")]
    StartVariation,

    #[token(")")]
    EndVariation,

    // Comments in braces
    #[regex(r"\{[^}]*\}")]
    Comment(&'a str),

    // Annotations
    #[regex(r"(!+|\?+|!\?|\?!|\$[0-9]+)")]
    Annotation(&'a str),

    // Game results
    #[token("1-0")]
    #[token("0-1")]
    #[token("1/2-1/2")]
    #[token("*")]
    Result(&'a str),

    // Whitespace and newlines are ignored
    #[regex(r"[ \t\r\n]+", logos::skip)]
    Whitespace,
}

/// Tokenizes a PGN string into a list of PgnTokens
pub fn tokenize_pgn(pgn: &str) -> Result<Vec<PgnToken>, PgnParseError> {
    let mut tokens = Vec::new();
    let mut lex = LogosToken::lexer(pgn);

    while let Some(token) = lex.next() {
        match token {
            Ok(LogosToken::Tag(tag_text)) => {
                // Remove the brackets and capture just the tag content
                let tag_content = &tag_text[1..tag_text.len()-1];
                tokens.push(PgnToken::Tag(tag_content.to_string()));
            },

            Ok(LogosToken::Move(move_text)) => {
                tokens.push(PgnToken::Move(move_text.to_string()));
            },

            Ok(LogosToken::MoveNumber(num_text)) => {
                // Parse the move number and count the periods
                let num_end = num_text.find('.').unwrap_or(num_text.len());
                let move_number = match num_text[..num_end].parse::<u16>() {
                    Ok(n) => n,
                    Err(_) => return Err(PgnParseError::InvalidToken(num_text.to_string())),
                };
                let period_count = num_text.len() - num_end;
                tokens.push(PgnToken::MoveNumberAndPeriods(move_number, period_count));
            },

            Ok(LogosToken::StartVariation) => {
                tokens.push(PgnToken::StartVariation);
            },

            Ok(LogosToken::EndVariation) => {
                tokens.push(PgnToken::EndVariation);
            },

            Ok(LogosToken::Comment(comment_text)) => {
                // Remove the braces and capture just the comment content
                let comment_content = &comment_text[1..comment_text.len()-1];
                tokens.push(PgnToken::Comment(comment_content.to_string()));
            },

            Ok(LogosToken::Annotation(annotation_text)) => {
                tokens.push(PgnToken::Annotation(annotation_text.to_string()));
            },

            Ok(LogosToken::Result(result_text)) => {
                tokens.push(PgnToken::Result(result_text.to_string()));
            },

            Ok(LogosToken::Whitespace) => {
                // Skip whitespace - should never reach here due to logos::skip
            },

            _ => {
                // Handle error token with span information for better diagnostics
                let span = lex.span();
                let invalid_text = &pgn[span.start..span.end];
                return Err(PgnParseError::InvalidToken(invalid_text.to_string()));
            },
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let pgn = "[Event \"Test\"] 1. e4 e5 2. Nf3 Nc6 *";
        let tokens = tokenize_pgn(pgn).unwrap();

        assert_eq!(tokens, vec![
            PgnToken::Tag("Event \"Test\"".to_string()),
            PgnToken::MoveNumberAndPeriods(1, 1),
            PgnToken::Move("e4".to_string()),
            PgnToken::Move("e5".to_string()),
            PgnToken::MoveNumberAndPeriods(2, 1),
            PgnToken::Move("Nf3".to_string()),
            PgnToken::Move("Nc6".to_string()),
            PgnToken::Result("*".to_string()),
        ]);
    }

    #[test]
    fn test_complex_moves() {
        let pgn = "1. Nbd7 Qxe4+ Bb5+ O-O-O# Raxc3 e8=Q+";
        let tokens = tokenize_pgn(pgn).unwrap();

        assert_eq!(tokens, vec![
            PgnToken::MoveNumberAndPeriods(1, 1),
            PgnToken::Move("Nbd7".to_string()),
            PgnToken::Move("Qxe4+".to_string()),
            PgnToken::Move("Bb5+".to_string()),
            PgnToken::Move("O-O-O#".to_string()),
            PgnToken::Move("Raxc3".to_string()),
            PgnToken::Move("e8=Q+".to_string()),
        ]);
    }

    #[test]
    fn test_variations_and_comments() {
        let pgn = "1. e4 e5 (1... c5 {Sicilian}) 2. Nf3 {Main line} Nc6";
        let tokens = tokenize_pgn(pgn).unwrap();

        assert_eq!(tokens, vec![
            PgnToken::MoveNumberAndPeriods(1, 1),
            PgnToken::Move("e4".to_string()),
            PgnToken::Move("e5".to_string()),
            PgnToken::StartVariation,
            PgnToken::MoveNumberAndPeriods(1, 3),
            PgnToken::Move("c5".to_string()),
            PgnToken::Comment("Sicilian".to_string()),
            PgnToken::EndVariation,
            PgnToken::MoveNumberAndPeriods(2, 1),
            PgnToken::Move("Nf3".to_string()),
            PgnToken::Comment("Main line".to_string()),
            PgnToken::Move("Nc6".to_string()),
        ]);
    }

    #[test]
    fn test_annotations() {
        let pgn = "1. e4! e5? 2. Nf3!! Nc6?? 3. Bb5!? c6?!";
        let tokens = tokenize_pgn(pgn).unwrap();

        assert_eq!(tokens, vec![
            PgnToken::MoveNumberAndPeriods(1, 1),
            PgnToken::Move("e4".to_string()),
            PgnToken::Annotation("!".to_string()),
            PgnToken::Move("e5".to_string()),
            PgnToken::Annotation("?".to_string()),
            PgnToken::MoveNumberAndPeriods(2, 1),
            PgnToken::Move("Nf3".to_string()),
            PgnToken::Annotation("!!".to_string()),
            PgnToken::Move("Nc6".to_string()),
            PgnToken::Annotation("??".to_string()),
            PgnToken::MoveNumberAndPeriods(3, 1),
            PgnToken::Move("Bb5".to_string()),
            PgnToken::Annotation("!?".to_string()),
            PgnToken::Move("c6".to_string()),
            PgnToken::Annotation("?!".to_string()),
        ]);
    }

    #[test]
    fn test_results() {
        let pgn = "1. e4 e5 2. Nf3 Nc6 1-0\n1. d4 d5 0-1\n1. c4 e5 1/2-1/2";
        let tokens = tokenize_pgn(pgn).unwrap();

        assert!(tokens.contains(&PgnToken::Result("1-0".to_string())));
        assert!(tokens.contains(&PgnToken::Result("0-1".to_string())));
        assert!(tokens.contains(&PgnToken::Result("1/2-1/2".to_string())));
    }

    #[test]
    fn test_numeric_annotations() {
        let pgn = "1. e4 $1 e5 $13 2. Nf3 $40";
        let tokens = tokenize_pgn(pgn).unwrap();

        assert_eq!(tokens, vec![
            PgnToken::MoveNumberAndPeriods(1, 1),
            PgnToken::Move("e4".to_string()),
            PgnToken::Annotation("$1".to_string()),
            PgnToken::Move("e5".to_string()),
            PgnToken::Annotation("$13".to_string()),
            PgnToken::MoveNumberAndPeriods(2, 1),
            PgnToken::Move("Nf3".to_string()),
            PgnToken::Annotation("$40".to_string()),
        ]);
    }

    #[test]
    fn test_invalid_token() {
        let pgn = "1. e4 e5 2. #invalid Nc6";
        let result = tokenize_pgn(pgn);

        assert!(result.is_err());
        if let Err(PgnParseError::InvalidToken(token)) = result {
            assert_eq!(token, "#");
        } else {
            panic!("Expected InvalidToken error");
        }
    }

    #[test]
    fn test_full_game() {
        let pgn = r#"
            [Event "F/S Return Match"]
            [Site "Belgrade, Serbia JUG"]
            [Date "1992.11.04"]
            [Round "29"]
            [White "Fischer, Robert J."]
            [Black "Spassky, Boris V."]
            [Result "1/2-1/2"]

            1. e4 e5 2. Nf3 Nc6 3. Bb5 a6
            4. Ba4 Nf6 5. O-O Be7 6. Re1 b5
            7. Bb3 d6 8. c3 O-O 9. h3 Nb8
            10. d4 Nbd7 11. c4 c6 12. cxb5 axb5
            13. Nc3 Bb7 14. Bg5 b4 15. Nb1 h6
            16. Bh4 c5 17. dxe5 Nxe4 18. Bxe7 Qxe7
            19. exd6 Qf6 20. Nbd2 Nxd6 21. Nc4 Nxc4
            22. Bxc4 Nb6 23. Ne5 Rae8 24. Bxf7+ Rxf7
            25. Nxf7 Rxe1+ 26. Qxe1 Kxf7 27. Qe3 Qg5
            28. Qxg5 hxg5 29. b3 Ke6 30. a3 Kd6
            31. axb4 cxb4 32. Ra5 Nd5 33. f3 Bc8
            34. Kf2 Bf5 35. Ra7 g6 36. Ra6+ Kc5
            37. Ke1 Nf4 38. g3 Nxh3 39. Kd2 Kb5
            40. Rd6 Kc5 41. Ra6 Nf2 42. g4 Bd3
            43. Re6 1/2-1/2
        "#;

        let tokens = tokenize_pgn(pgn).unwrap();

        // Verify the token count and check a few key tokens
        assert!(tokens.len() > 100); // Should have lots of tokens for a full game
        assert_eq!(tokens[0], PgnToken::Tag("Event \"F/S Return Match\"".to_string()));
        assert_eq!(tokens[tokens.len()-1], PgnToken::Result("1/2-1/2".to_string()));
    }
}