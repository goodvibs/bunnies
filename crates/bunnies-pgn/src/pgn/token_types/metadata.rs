use std::sync::LazyLock;

use logos::Lexer;
use regex::Regex;

use crate::pgn::{
    error::PgnError,
    token::{COMMENT_REGEX, MOVE_NUMBER_REGEX, ParsablePgnToken, PgnToken, TAG_REGEX},
};

static COMPILED_TAG_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(TAG_REGEX).unwrap());
static COMPILED_MOVE_NUMBER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(MOVE_NUMBER_REGEX).unwrap());
static COMPILED_COMMENT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(COMMENT_REGEX).unwrap());

#[derive(Clone, Debug, PartialEq)]
pub struct PgnTag {
    pub name: String,
    pub value: String,
}

impl PgnTag {
    pub fn render(&self) -> String {
        format!("[{} \"{}\"]", self.name, self.value)
    }
}

impl ParsablePgnToken for PgnTag {
    fn parse(lex: &mut Lexer<PgnToken>) -> Result<Self, PgnError> {
        let text = lex.slice();

        if let Some(captures) = COMPILED_TAG_REGEX.captures(text) {
            let name = captures.get(1).unwrap().as_str().to_string();
            let value = captures.get(2).unwrap().as_str().to_string();
            Ok(Self { name, value })
        } else {
            Err(PgnError::InvalidTag(text.to_string()))
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[derive_const(PartialEq)]
pub struct PgnMoveNumber {
    pub fullmove_number: u16,
}

impl PgnMoveNumber {
    pub fn render(&self, num_periods: usize) -> String {
        format!("{}{}", self.fullmove_number, ".".repeat(num_periods))
    }
}

impl ParsablePgnToken for PgnMoveNumber {
    fn parse(lex: &mut Lexer<PgnToken>) -> Result<Self, PgnError> {
        let text = lex.slice();

        if let Some(captures) = COMPILED_MOVE_NUMBER_REGEX.captures(text) {
            let fullmove_number = match captures.get(1).unwrap().as_str().parse::<u16>() {
                Ok(num) => num,
                Err(_) => return Err(PgnError::InvalidMoveNumber(text.to_string())),
            };
            Ok(Self { fullmove_number })
        } else {
            Err(PgnError::InvalidMoveNumber(text.to_string()))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PgnComment {
    pub comment: String,
}

impl PgnComment {
    pub fn render(&self) -> String {
        format!("{{{}}}", self.comment)
    }
}

impl ParsablePgnToken for PgnComment {
    fn parse(lex: &mut Lexer<PgnToken>) -> Result<Self, PgnError> {
        let text = lex.slice();

        match COMPILED_COMMENT_REGEX.captures(text) {
            Some(captures) => {
                let comment = captures.get(1).unwrap().as_str().to_string();
                Ok(Self { comment })
            }
            None => Err(PgnError::InvalidComment(text.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use logos::Logos;

    use super::*;
    use crate::pgn::token::PgnToken;

    #[test]
    fn test_pgn_tag() {
        let mut lex = PgnToken::lexer(" [Event \"F/S Return Match\"] ");
        lex.next();
        let tag = PgnTag::parse(&mut lex).unwrap();
        assert_eq!(tag.name, "Event");
        assert_eq!(tag.value, "F/S Return Match");
    }

    #[test]
    fn test_pgn_tag_invalid() {
        let mut lex = PgnToken::lexer(" [Event \"F/S Return Match] ");
        lex.next();
        let result = PgnTag::parse(&mut lex);
        assert!(result.is_err());
    }

    #[test]
    fn test_comment() {
        let mut lex = PgnToken::lexer("{This is a comment}");
        lex.next();
        let token = PgnComment::parse(&mut lex).unwrap();
        assert_eq!(token.comment, "This is a comment");

        let mut lex = PgnToken::lexer("{This is an invalid comment");
        lex.next();
        let token = PgnComment::parse(&mut lex);
        assert!(token.is_err());
    }

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
    fn test_parse_invalid_move_number_format() {
        let mut lex = PgnToken::lexer("a.");
        lex.next();
        let result = PgnMoveNumber::parse(&mut lex);
        assert!(result.is_err());
        assert!(matches!(result, Err(PgnError::InvalidMoveNumber(_))));
    }
}
