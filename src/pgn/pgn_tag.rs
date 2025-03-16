use logos::Lexer;
use regex::Regex;
use crate::pgn::lexing::PgnToken;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PgnTag {
    pub name: String,
    pub value: String
}

impl PgnTag {
    pub fn parse(lex: &mut Lexer<PgnToken>) -> Option<PgnTag> {
        let text = lex.slice();
        let tag_regex = Regex::new(r#"\[\s*([A-Za-z0-9_]+)\s+"([^"]*)"\s*\]"#).unwrap();

        if let Some(captures) = tag_regex.captures(text) {
            let name = captures.get(1).unwrap().as_str().to_string();
            let value = captures.get(2).unwrap().as_str().to_string();
            return Some(PgnTag { name, value });
        }

        None
    }
}