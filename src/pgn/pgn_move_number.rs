use logos::Lexer;
use regex::Regex;
use crate::pgn::pgn_token::{ParsablePgnToken, PgnToken};
use crate::pgn::lexing_error::PgnLexingError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PgnMoveNumber {
    pub fullmove_number: u16
}

impl ParsablePgnToken for PgnMoveNumber {
    fn parse(lex: &mut Lexer<PgnToken>) -> Result<Self, PgnLexingError> {
        let text = lex.slice();
        let number_regex = Regex::new(r"[0-9]+").unwrap();

        match number_regex.find(text) {
            Some(m) => {
                let fullmove_number = m.as_str().parse::<u16>().unwrap();
                Ok(Self { fullmove_number })
            },
            None => Err(PgnLexingError::InvalidMoveNumber(text.to_string()))
        }
    }
}