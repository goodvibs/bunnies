use logos::Lexer;
use regex::Regex;
use crate::pgn::lexing::PgnToken;
use crate::pgn::pgn_move::{PgnCommonMoveInfo, PgnMove};
use crate::r#move::{Move, MoveFlag};
use crate::state::State;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PgnCastlingMove {
    pub is_kingside: bool,
    pub common_move_info: PgnCommonMoveInfo
}

impl PgnCastlingMove {
    pub fn parse(lex: &mut Lexer<PgnToken>) -> Option<PgnCastlingMove> {
        let text = lex.slice();
        let move_regex = Regex::new(r"(O-O(-O)?)|(0-0(-0)?)([+#])?([?!]*)\s*(\$[0-9]+)?").unwrap();
        if let Some(captures) = move_regex.captures(text) {
            let is_kingside = captures.get(1).is_some();

            Some(
                PgnCastlingMove {
                    is_kingside,
                    common_move_info: PgnCommonMoveInfo::from(captures.get(4), captures.get(5), captures.get(6))
                }
            )
        } else {
            None
        }
    }
}

impl PgnMove for PgnCastlingMove {
    fn matches_move(&self, mv: Move, initial_state: &State) -> bool {
        let flag = mv.get_flag();
        if flag != MoveFlag::Castling {
            return false
        } else if self.is_kingside != (mv.get_destination().get_file() == 6) {
            return false
        }

        true
    }

    fn get_common_move_info(&self) -> &PgnCommonMoveInfo {
        &self.common_move_info
    }

    fn get_common_move_info_mut(&mut self) -> &mut PgnCommonMoveInfo {
        &mut self.common_move_info
    }
}