use logos::Lexer;
use regex::Regex;
use static_init::dynamic;
use crate::pgn::pgn_token::{ParsablePgnToken, PgnToken};
use crate::pgn::lexing_error::PgnLexingError;

const MOVE_NUMBER_REGEX: &str = r"([0-9]+)\.+";

#[dynamic]
static COMPILED_MOVE_NUMBER_REGEX: Regex = Regex::new(MOVE_NUMBER_REGEX).unwrap();

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PgnMoveNumber {
    pub fullmove_number: u16
}

impl ParsablePgnToken for PgnMoveNumber {
    fn parse(lex: &mut Lexer<PgnToken>) -> Result<Self, PgnLexingError> {
        let text = lex.slice();

        if let Some(captures) = COMPILED_MOVE_NUMBER_REGEX.captures(text) {
            let fullmove_number = match captures.get(1).unwrap().as_str().parse::<u16>() {
                Ok(num) => num,
                Err(_) => return Err(PgnLexingError::InvalidMoveNumber(text.to_string()))
            };
            Ok(Self { fullmove_number })
        } else {
            Err(PgnLexingError::InvalidMoveNumber(text.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use logos::Logos;
    use super::*;
    use crate::pgn::pgn_token::PgnToken;

    #[test]
    fn test_parse_simple_move_number() {
        let mut lex = PgnToken::lexer("1.");
        lex.next();
        let move_number = PgnMoveNumber::parse(&mut lex).unwrap();
        assert_eq!(move_number.fullmove_number, 1);
    }

    #[test]
    fn test_parse_move_number_with_multiple_dots() {
        let mut lex = PgnToken::lexer("12...");
        lex.next();
        let move_number = PgnMoveNumber::parse(&mut lex).unwrap();
        assert_eq!(move_number.fullmove_number, 12);
    }

    #[test]
    fn test_parse_large_move_number() {
        let mut lex = PgnToken::lexer("9999.");
        lex.next();
        let move_number = PgnMoveNumber::parse(&mut lex).unwrap();
        assert_eq!(move_number.fullmove_number, 9999);
    }

    #[test]
    fn test_parse_move_number_with_whitespace() {
        // Note: This test assumes your lexer handles whitespace correctly
        // If it doesn't, you might need to adjust your PgnToken lexer
        let mut lex = PgnToken::lexer("42. ");
        lex.next();
        let move_number = PgnMoveNumber::parse(&mut lex).unwrap();
        assert_eq!(move_number.fullmove_number, 42);
    }

    #[test]
    fn test_parse_invalid_move_number_format() {
        let mut lex = PgnToken::lexer("a.");
        lex.next();
        let result = PgnMoveNumber::parse(&mut lex);
        assert!(result.is_err());

        match result {
            Err(PgnLexingError::InvalidMoveNumber(_)) => (),
            _ => panic!("Expected InvalidMoveNumber error")
        }
    }

    #[test]
    fn test_parse_move_number_overflow() {
        // This should exceed u16 range
        let mut lex = PgnToken::lexer("65536.");
        lex.next();
        let result = PgnMoveNumber::parse(&mut lex);
        assert!(result.is_err());
    }
}