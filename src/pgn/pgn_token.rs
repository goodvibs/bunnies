use logos::{Logos, Lexer};
use crate::color::Color;
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
    #[regex(r"\{(.*)\}", PgnComment::parse)]
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