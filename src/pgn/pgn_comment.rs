use logos::Lexer;
use regex::Regex;
use crate::pgn::lexing::{ParsablePgnToken, PgnToken};
use crate::pgn::lexing_error::PgnLexingError;

#[derive(Debug, Clone, PartialEq)]
pub struct PgnComment {
    pub comment: String
}

impl ParsablePgnToken for PgnComment {
    fn parse(lex: &mut Lexer<PgnToken>) -> Result<Self, PgnLexingError> {
        let text = lex.slice();
        let comment_regex = Regex::new(r"\{([^}]*)\}").unwrap();

        match comment_regex.captures(text) {
            Some(captures) => {
                let comment = captures.get(1).unwrap().as_str().to_string();
                Ok(Self { comment })
            },
            None => Err(PgnLexingError::InvalidComment(text.to_string()))
        }
    }
}