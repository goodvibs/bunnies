use regex::Match;
use crate::r#move::Move;
use crate::state::State;

pub trait PgnMove: std::fmt::Debug {
    fn matches_move(&self, mv: Move, from_state: &State) -> bool;

    fn get_common_move_info(&self) -> &PgnCommonMoveInfo;

    fn get_common_move_info_mut(&mut self) -> &mut PgnCommonMoveInfo;
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PgnCommonMoveInfo {
    pub is_check: bool,
    pub is_checkmate: bool,
    pub annotation: Option<String>
}

impl PgnCommonMoveInfo {
    pub fn from(check_or_checkmate: Option<Match>, annotation: Option<Match>) -> PgnCommonMoveInfo {
        let (is_check, is_checkmate) = match check_or_checkmate {
            Some(m) => {
                let check_or_checkmate_char = m.as_str().chars().next().unwrap();
                (true, check_or_checkmate_char == '#')
            },
            None => (false, false)
        };

        PgnCommonMoveInfo { is_check, is_checkmate, annotation: annotation.map(|m| m.as_str().to_string()) }
    }
}