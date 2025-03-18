use logos::Lexer;
use regex::Regex;
use static_init::dynamic;
use crate::pgn::token::{ParsablePgnToken, PgnToken};
use crate::pgn::lexing_error::PgnLexingError;

/// Regex for parsing PGN tags.
/// Capturing groups:
/// 0. Everything
/// 1. Tag name
/// 2. Tag value (inside quotes)
const TAG_REGEX: &str = r#"\[\s*([A-Za-z0-9_]+)\s+"([^"]*)"\s*\]"#;

#[dynamic]
static COMPILED_TAG_REGEX: Regex = Regex::new(TAG_REGEX).unwrap();

#[derive(Clone, Debug, PartialEq)]
pub struct PgnTag {
    pub name: String,
    pub value: String
}

impl PgnTag {
    pub fn render(&self) -> String {
        format!("[{} \"{}\"]", self.name, self.value)
    }
}

impl ParsablePgnToken for PgnTag {
    fn parse(lex: &mut Lexer<PgnToken>) -> Result<Self, PgnLexingError> {
        let text = lex.slice();

        if let Some(captures) = COMPILED_TAG_REGEX.captures(text) {
            let name = captures.get(1).unwrap().as_str().to_string();
            let value = captures.get(2).unwrap().as_str().to_string();
            Ok(Self { name, value })
        } else {
            Err(PgnLexingError::InvalidTag(text.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use logos::Logos;
    use super::PgnTag;
    use crate::pgn::token::ParsablePgnToken;
    use crate::pgn::PgnToken;

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
    fn test_pgn_tag_invalid_no_value() {
        let mut lex = PgnToken::lexer(" [Event \"F/S Return Match] ");
        lex.next();
        let result = PgnTag::parse(&mut lex);
        assert!(result.is_err());
    }
}