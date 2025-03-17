use logos::Lexer;
use regex::Regex;
use static_init::dynamic;
use crate::pgn::pgn_token::{ParsablePgnToken, PgnToken};
use crate::pgn::lexing_error::PgnLexingError;

/// The regex pattern for a comment.
/// Capturing groups:
/// 0. Everything
/// 1. The content of the comment
const COMMENT_REGEX: &str = r"\{(.*)\}";

#[dynamic]
static COMPILED_COMMENT_REGEX: Regex = Regex::new(COMMENT_REGEX).unwrap();

#[derive(Debug, Clone, PartialEq)]
pub struct PgnComment {
    pub comment: String
}

impl ParsablePgnToken for PgnComment {
    fn parse(lex: &mut Lexer<PgnToken>) -> Result<Self, PgnLexingError> {
        let text = lex.slice();

        match COMPILED_COMMENT_REGEX.captures(text) {
            Some(captures) => {
                let comment = captures.get(1).unwrap().as_str().to_string();
                Ok(Self { comment })
            },
            None => Err(PgnLexingError::InvalidComment(text.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PgnComment;
    use crate::pgn::pgn_token::ParsablePgnToken;
    use crate::pgn::pgn_token::PgnToken;
    use logos::Logos;

    #[test]
    fn test_comment() {
        let mut lex = PgnToken::lexer("{This is a comment}");
        lex.next();
        let token = PgnComment::parse(&mut lex).unwrap();
        assert_eq!(token.comment, "This is a comment");

        let mut lex = PgnToken::lexer("{This is a comment with a {nested} comment}");
        lex.next();
        let token = PgnComment::parse(&mut lex).unwrap();
        assert_eq!(token.comment, "This is a comment with a {nested} comment");

        let mut lex = PgnToken::lexer("{This is an invalid comment");
        lex.next();
        let token = PgnComment::parse(&mut lex);
        assert!(token.is_err());
    }
}