use regex::Match;
use crate::r#move::Move;
use crate::state::GameState;

pub trait PgnMove: std::fmt::Debug {
    fn matches_move(&self, mv: Move, from_state: &GameState) -> bool;

    fn get_common_move_info(&self) -> &PgnCommonMoveInfo;

    fn get_common_move_info_mut(&mut self) -> &mut PgnCommonMoveInfo;

    fn render(&self, include_annotation: bool, include_nag: bool) -> String;
}

#[derive(Clone, Debug, PartialEq)]
pub struct PgnCommonMoveInfo {
    pub is_check: bool,
    pub is_checkmate: bool,
    pub annotation: Option<String>,
    pub nag: Option<u8>
}

impl PgnCommonMoveInfo {
    pub fn render(&self, include_annotation: bool, include_nag: bool) -> String {
        let check_or_checkmate = if self.is_checkmate {
            "#"
        } else if self.is_check {
            "+"
        } else {
            ""
        };

        let annotation = if include_annotation {
            self.annotation.as_deref().unwrap_or("")
        } else {
            ""
        };

        let nag = if include_nag {
            match self.nag {
                Some(nag) => format!(" ${}", nag),
                None => "".to_string()
            }
        } else {
            "".to_string()
        };

        format!("{}{}{}", check_or_checkmate, annotation, nag)
    }
}

impl PgnCommonMoveInfo {
    pub fn from(check_or_checkmate: Option<Match>, annotation: Option<Match>, nag: Option<Match>) -> PgnCommonMoveInfo {
        let (is_check, is_checkmate) = match check_or_checkmate {
            Some(m) => {
                let check_or_checkmate_char = m.as_str().chars().next().unwrap();
                (true, check_or_checkmate_char == '#')
            },
            None => (false, false)
        };

        let annotation = match annotation {
            Some(m) => Some(m.as_str().to_string()),
            None => None
        };

        let nag = match nag {
            Some(m) => m.as_str().parse().ok(),
            None => None
        };

        PgnCommonMoveInfo {
            is_check,
            is_checkmate,
            annotation,
            nag
        }
    }
}