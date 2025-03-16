use std::cmp::{Ordering, PartialOrd};
use logos::Logos;
use crate::color::Color;
use crate::pgn::error::PgnParseError;
use crate::pgn::lexing::{PgnToken};
use crate::pgn::pgn_object::PgnObject;
use crate::pgn::move_tree_node::{MoveData, MoveTreeNode};
use crate::pgn::{NonCastlingMove, Tag};
use crate::r#move::Move;
use crate::state::{State, Termination};

#[derive(Debug, PartialEq)]
pub enum PgnParseState {
    Tags,
    Moves {
        is_move_expected: bool,
    },
    ResultFound
}

pub struct EnrichedMoveTreeNode<'a> {
    pub node: &'a MoveTreeNode,
    pub state_after_move: State,
}

pub struct PgnParser<'a> {
    pub parse_state: PgnParseState,

    pub move_tree: PgnObject,
    pub current: EnrichedMoveTreeNode<'a>,
    pub stack: Vec<EnrichedMoveTreeNode<'a>>,
}

impl PgnParser {
    pub fn new() -> PgnParser {
        let move_tree = PgnObject::new();
        let current_node = move_tree.tree_root.borrow();
        PgnParser {
            parse_state: PgnParseState::Tags,
            move_tree,
            current: EnrichedMoveTreeNode {
                node: &current_node,
                state_after_move: State::initial(),
            },
            stack: Vec::new(),
        }
    }

    pub fn parse(&mut self, pgn: &str) -> Result<(), PgnParseError> {
        let mut tokens = PgnToken::lexer(pgn);
        if let Some(token) = tokens.next() {
            match token {
                PgnToken::Tag(tag) => {
                    self.process_tag(tag)?;
                }
                PgnToken::MoveNumber(move_number) => {
                    self.process_move_number(move_number)?;
                }
                PgnToken::NonCastlingMove(mv) => {
                    self.process_non_castling_move(mv)?;
                }
                PgnToken::CastlingMove(mv) => {
                    self.process_castling_move(mv)?;
                }
                PgnToken::Nag(nag) => {
                    self.process_nag(nag)?;
                }
                PgnToken::StartVariation => {
                    self.process_start_variation()?;
                }
                PgnToken::EndVariation => {
                    self.process_end_variation()?;
                }
                PgnToken::Comment(comment) => {
                    self.process_comment(comment)?;
                }
                PgnToken::Result(result) => {
                    self.process_result(result)?;
                }
                PgnToken::Incomplete => {
                    self.process_incomplete()?;
                }
            }
        }
        Ok(())
    }

    fn process_tag(&mut self, tag: Tag) -> Result<(), PgnParseError> {
        if self.parse_state != PgnParseState::Tags {
            return Err(PgnParseError::UnexpectedToken(format!("Unexpected tag token: {:?}", tag)));
        }
        self.move_tree.add_tag(tag.name, tag.value);
        Ok(())
    }

    fn process_move_number(&mut self, move_number: u16) -> Result<(), PgnParseError> {
        match self.parse_state {
            PgnParseState::Tags => {
                self.parse_state = PgnParseState::Moves { is_move_expected: true };
                let expected_fullmove = self.current.state_after_move.get_fullmove();
                if move_number == expected_fullmove {
                    Ok(())
                } else {
                    Err(PgnParseError::IncorrectMoveNumber(move_number.to_string()))
                }
            }
            PgnParseState::Moves { is_move_expected } => {
                if is_move_expected {
                    Err(PgnParseError::UnexpectedToken(format!("Unexpected move number token: {}", move_number)))
                }
                else {
                    let expected_fullmove = self.current.state_after_move.get_fullmove();
                    if move_number == expected_fullmove {
                        self.parse_state = PgnParseState::Moves { is_move_expected: true };
                        Ok(())
                    } else {
                        Err(PgnParseError::IncorrectMoveNumber(move_number.to_string()))
                    }
                }
            }
            PgnParseState::ResultFound => {
                Err(PgnParseError::UnexpectedToken(format!("Unexpected move number token: {}", move_number)))
            }
        }
    }

    fn process_non_castling_move(&mut self, non_castling_move: NonCastlingMove) -> Result<(), PgnParseError> {
        if let Some(PgnParseState::Moves { is_move_expected }) = self.parse_state {
            if is_move_expected {
                let mut current_state = &self.current.state_after_move;
                let possible_moves = current_state.calc_legal_moves();
                let mut matched_move = None;
                for possible_move in possible_moves {
                    if non_castling_move.matches_move(&possible_move, current_state) {
                        if matched_move.is_some() {
                            return Err(PgnParseError::AmbiguousMove(format!("Ambiguous move: {:?}", non_castling_move)));
                        } else {
                            matched_move = Some(possible_move);
                        }
                    }
                }
                if let Some(matched_move) = matched_move {
                    let new_state = {
                        let mut state = current_state.clone();
                        state.make_move(matched_move);
                        state
                    };
                    let move_data = MoveData {
                        mv: matched_move,
                        annotation: non_castling_move.common_move_info.annotation,
                        nag: non_castling_move.common_move_info.nag,
                    };
                    let new_node = MoveTreeNode::new(move_data, None);
                    self.current.node.add_continuation(new_node);
                    self.current.state_after_move = new_state;
                    self.parse_state = PgnParseState::Moves { is_move_expected: false };
                    Ok(())
                } else {
                    Err(PgnParseError::IllegalMove(format!("Illegal move: {:?}", non_castling_move)))
                }
            } else {
                Err(PgnParseError::UnexpectedToken(format!("Unexpected move token: {:?}", non_castling_move)))
            }
        } else {
            Err(PgnParseError::UnexpectedToken(format!("Unexpected move token: {:?}", non_castling_move)))
        }
    }
}
