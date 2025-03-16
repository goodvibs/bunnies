use std::fmt::{Display};
use logos::{Logos, Lexer};
use regex::{Regex};
use crate::color::Color;
use crate::pgn::lexing_error::PgnLexingError;
use crate::pgn::pgn_castling_move::PgnCastlingMove;
use crate::pgn::pgn_move_number::PgnMoveNumber;
use crate::pgn::pgn_non_castling_move::PgnNonCastlingMove;
use crate::pgn::pgn_tag::PgnTag;

pub trait ParsablePgnToken: Sized {
    fn parse(lex: &mut Lexer<PgnToken>) -> Result<Self, PgnLexingError>;
}

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[\s\t\n]+")]
#[logos(error = PgnLexingError)]
pub enum PgnToken {
    // Tags [TagName "TagValue"]
    #[regex(r#"\[\s*([A-Za-z0-9_]+)\s+"[^"]*"\s*\]"#, PgnTag::parse)]
    Tag(PgnTag),

    // Move numbers like 1. or 1...
    #[regex(r"([0-9]+)\.+", PgnMoveNumber::parse)]
    MoveNumber(PgnMoveNumber),

    // Moves like g4, Nf6, exd5+?!, etc.
    #[regex(r"[PNBRQK]?[a-h]?[1-8]?x?[a-h][1-8](=[NBRQ])?[+#]?[?!]*(\s+\$[0-9]+)?", PgnNonCastlingMove::parse)]
    NonCastlingMove(PgnNonCastlingMove),

    #[regex(r"((O-O(-O)?)|(0-0(-0)?))[+#]?[?!]*(\s+\$[0-9]+)?", PgnCastlingMove::parse)]
    CastlingMove(PgnCastlingMove),

    // Comments in { }
    #[regex(r"\{[^}]*\}", parse_comment)]
    Comment(String), // Regular comment

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

fn parse_comment(lex: &mut Lexer<PgnToken>) -> Option<String> {
    let text = lex.slice();
    let comment_regex = Regex::new(r"\{([^}]*)\}").unwrap();

    match comment_regex.captures(text) {
        Some(captures) => Some(captures.get(1).unwrap().as_str().to_string()),
        None => None
    }
}