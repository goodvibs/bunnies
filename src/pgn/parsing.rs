use std::cell::RefCell;
use std::rc::Rc;
use logos::Logos;
use crate::pgn::pgn_castling_move::PgnCastlingMove;
use crate::pgn::parsing_error::PgnParseError;
use crate::pgn::lexing::{PgnToken};
use crate::pgn::pgn_object::PgnObject;
use crate::pgn::move_tree_node::{MoveData, MoveTreeNode};
use crate::pgn::pgn_non_castling_move::PgnNonCastlingMove;
use crate::pgn::pgn_move::PgnMove;
use crate::pgn::pgn_move_number::PgnMoveNumber;
use crate::pgn::pgn_tag::PgnTag;
use crate::state::{State};

#[derive(Debug, PartialEq)]
pub enum PgnParseState {
    Tags,
    Moves {
        is_move_expected: bool,
    },
    ResultFound
}

pub struct EnrichedMoveTreeNode {
    pub node: Rc<RefCell<MoveTreeNode>>,
    pub state_after_move: State,
}

pub struct PgnParser {
    pub parse_state: PgnParseState,
    pub move_tree: PgnObject,
    pub current: EnrichedMoveTreeNode,
    pub stack: Vec<EnrichedMoveTreeNode>,
}

impl PgnParser {
    pub fn new() -> PgnParser {
        let move_tree = PgnObject::new();
        let current_node = Rc::clone(&move_tree.tree_root);
        PgnParser {
            parse_state: PgnParseState::Tags,
            move_tree,
            current: EnrichedMoveTreeNode {
                node: current_node,
                state_after_move: State::initial(),
            },
            stack: Vec::new(),
        }
    }

    pub fn parse(&mut self, pgn: &str) -> Result<(), PgnParseError> {
        let mut tokens = PgnToken::lexer(pgn);
        if let Some(token) = tokens.next() {
            let token = match token {
                Ok(token) => token,
                Err(e) => return Err(PgnParseError::LexingError(format!("Error while lexing: {:?}", e))),
            };
            match token {
                PgnToken::Tag(tag) => {
                    self.process_tag(tag)?;
                }
                PgnToken::MoveNumber(move_number) => {
                    self.process_move_number(move_number)?;
                }
                PgnToken::NonCastlingMove(mv) => {
                    self.process_move::<PgnNonCastlingMove>(mv)?;
                }
                PgnToken::CastlingMove(mv) => {
                    self.process_move::<PgnCastlingMove>(mv)?;
                }
                PgnToken::StartVariation => {
                    // self.process_start_variation()?;
                }
                PgnToken::EndVariation => {
                    // self.process_end_variation()?;
                }
                PgnToken::Comment(comment) => {
                    // self.process_comment(comment)?;
                }
                PgnToken::Result(result) => {
                    // self.process_result(result)?;
                }
                PgnToken::Incomplete => {
                    // self.process_incomplete()?;
                }
            }
        }
        Ok(())
    }

    fn process_tag(&mut self, tag: PgnTag) -> Result<(), PgnParseError> {
        if self.parse_state != PgnParseState::Tags {
            return Err(PgnParseError::UnexpectedToken(format!("Unexpected tag token: {:?}", tag)));
        }
        self.move_tree.add_tag(tag.name, tag.value);
        Ok(())
    }

    fn process_move_number(&mut self, pgn_move_number: PgnMoveNumber) -> Result<(), PgnParseError> {
        match self.parse_state {
            PgnParseState::Tags => {
                self.parse_state = PgnParseState::Moves { is_move_expected: false };
                self.process_move_number(pgn_move_number)
            }
            PgnParseState::Moves { is_move_expected } => {
                if is_move_expected {
                    Err(PgnParseError::UnexpectedToken(format!("Unexpected move number token: {:?}", pgn_move_number)))
                }
                else {
                    let expected_fullmove = self.current.state_after_move.get_fullmove();
                    if pgn_move_number.fullmove_number == expected_fullmove {
                        self.parse_state = PgnParseState::Moves { is_move_expected: true };
                        Ok(())
                    } else {
                        Err(PgnParseError::IncorrectMoveNumber(format!("{:?}", pgn_move_number)))
                    }
                }
            }
            PgnParseState::ResultFound => {
                Err(PgnParseError::UnexpectedToken(format!("Unexpected move number token: {:?}", pgn_move_number)))
            }
        }
    }

    fn process_move<PgnMoveType: PgnMove>(&mut self, pgn_move: PgnMoveType) -> Result<(), PgnParseError> {
        match self.parse_state {
            PgnParseState::Moves { is_move_expected: true } => {
                let mut current_state = &self.current.state_after_move;
                let possible_moves = current_state.calc_legal_moves();

                let mut matched_move = None;
                for possible_move in possible_moves {
                    if pgn_move.matches_move(possible_move, current_state) {
                        if matched_move.is_some() {
                            return Err(PgnParseError::AmbiguousMove(format!("Ambiguous move: {:?}", pgn_move)));
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
                        annotation: pgn_move.get_common_move_info().annotation.clone(),
                        nag: pgn_move.get_common_move_info().nag.clone(),
                    };
                    let new_node = Rc::new(RefCell::new(MoveTreeNode::new(move_data, None)));
                    self.current.node.borrow_mut().add_continuation(&new_node);
                    self.current = EnrichedMoveTreeNode {
                        node: new_node,
                        state_after_move: new_state,
                    };
                    self.parse_state = PgnParseState::Moves { is_move_expected: false };
                    Ok(())
                } else {
                    Err(PgnParseError::IllegalMove(format!("Illegal move: {:?}", pgn_move)))
                }
            }
            _ => {
                Err(PgnParseError::UnexpectedToken(format!("Unexpected move token: {:?}", pgn_move)))
            }
        }
    }
}