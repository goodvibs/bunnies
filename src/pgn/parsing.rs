use std::cell::RefCell;
use std::rc::Rc;
use logos::Logos;
use crate::color::Color;
use crate::pgn::pgn_castling_move::PgnCastlingMove;
use crate::pgn::parsing_error::PgnParseError;
use crate::pgn::pgn_token::{PgnToken};
use crate::pgn::pgn_object::PgnObject;
use crate::pgn::move_tree_node::{MoveData, MoveTreeNode};
use crate::pgn::pgn_comment::PgnComment;
use crate::pgn::pgn_non_castling_move::PgnNonCastlingMove;
use crate::pgn::pgn_move::PgnMove;
use crate::pgn::pgn_move_number::PgnMoveNumber;
use crate::pgn::pgn_tag::PgnTag;
use crate::state::{State};

#[derive(Debug, PartialEq)]
pub enum PgnParseState {
    Tags,
    Moves {
        move_number_just_seen: bool,
    },
    ResultFound
}

#[derive(Clone)]
pub struct EnrichedMoveTreeNode {
    pub node: Rc<RefCell<MoveTreeNode>>,
    pub state_after_move: State,
}

#[derive(Clone)]
pub struct Context {
    pub current: EnrichedMoveTreeNode,
    pub previous: Option<EnrichedMoveTreeNode>,
}

impl Context {
    pub fn next(&mut self, new_node: &Rc<RefCell<MoveTreeNode>>, new_state: State) {
        self.current.node.borrow_mut().add_continuation(new_node);

        // Create the new value we want to assign to self.current
        let new_current = EnrichedMoveTreeNode {
            node: Rc::clone(new_node),
            state_after_move: new_state,
        };

        // Replace self.current with new_current and get the old value
        let old_current = std::mem::replace(&mut self.current, new_current);

        // Set previous to the old current value
        self.previous = Some(old_current);
    }
}

pub struct ContextManager {
    pub context: Context,
    pub stack: Vec<Context>,
}

impl ContextManager {
    pub fn new(root_node: &Rc<RefCell<MoveTreeNode>>, initial_state: State) -> ContextManager {
        ContextManager {
            context: Context {
                current: EnrichedMoveTreeNode {
                    node: Rc::clone(root_node),
                    state_after_move: initial_state,
                },
                previous: None,
            },
            stack: Vec::new(),
        }
    }

    pub fn create_branch_from_previous(&mut self) {
        let clone_of_previous = self.context.previous.clone().expect("No previous node to create branch from");
        let new_context = Context {
            current: clone_of_previous,
            previous: None,
        };
        let old_context = std::mem::replace(&mut self.context, new_context);
        self.stack.push(old_context);
    }

    pub fn end_branch(&mut self) {
        let previous_context = self.stack.pop().expect("No previous context to return to");
        self.context = previous_context;
    }
}

pub struct PgnParser {
    pub parse_state: PgnParseState,
    pub object: PgnObject,
    pub context_manager: ContextManager
}

impl PgnParser {
    pub fn new() -> PgnParser {
        let pgn_object = PgnObject::new();
        let current_node = &pgn_object.tree_root;
        let context_manager = ContextManager::new(&current_node, State::initial());
        PgnParser {
            parse_state: PgnParseState::Tags,
            object: pgn_object,
            context_manager,
        }
    }

    pub fn parse(&mut self, pgn: &str) -> Result<(), PgnParseError> {
        let mut tokens = PgnToken::lexer(pgn);
        while let Some(token) = tokens.next() {
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

    fn process_tag(&mut self, tag: PgnTag) -> Result<(), PgnParseError> {
        if self.parse_state != PgnParseState::Tags {
            return Err(PgnParseError::UnexpectedToken(format!("Unexpected tag token: {:?}", tag)));
        }
        self.object.add_tag(tag.name, tag.value);
        Ok(())
    }

    fn process_move_number(&mut self, pgn_move_number: PgnMoveNumber) -> Result<(), PgnParseError> {
        match self.parse_state {
            PgnParseState::Tags => {
                self.parse_state = PgnParseState::Moves { move_number_just_seen: false };
                self.process_move_number(pgn_move_number)
            }
            PgnParseState::Moves { move_number_just_seen } => {
                if move_number_just_seen {
                    Err(PgnParseError::UnexpectedToken(format!("Unexpected move number token: {:?}", pgn_move_number)))
                }
                else {
                    let expected_fullmove = self.context_manager.context.current.state_after_move.get_fullmove();
                    if pgn_move_number.fullmove_number == expected_fullmove {
                        self.parse_state = PgnParseState::Moves { move_number_just_seen: true };
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
            PgnParseState::Moves { move_number_just_seen } => {
                let mut current_state = &self.context_manager.context.current.state_after_move;
                if !(move_number_just_seen || current_state.side_to_move == Color::Black) {
                    return Err(PgnParseError::UnexpectedToken(format!("Unexpected move token: {:?}", pgn_move)));
                }
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
                    self.context_manager.context.next(&new_node, new_state);
                    self.parse_state = PgnParseState::Moves { move_number_just_seen: false };
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

    fn process_start_variation(&mut self) -> Result<(), PgnParseError> {
        match self.parse_state {
            PgnParseState::Moves { move_number_just_seen: false } => {
                if self.context_manager.context.previous.is_none() {
                    Err(PgnParseError::UnexpectedToken("Unexpected start variation token".to_string()))
                } else {
                    self.context_manager.create_branch_from_previous();
                    Ok(())
                }
            }
            _ => {
                Err(PgnParseError::UnexpectedToken("Unexpected start variation token".to_string()))
            }
        }
    }

    fn process_end_variation(&mut self) -> Result<(), PgnParseError> {
        match self.parse_state {
            PgnParseState::Moves { move_number_just_seen: false } => {
                if self.context_manager.stack.is_empty() {
                    Err(PgnParseError::UnexpectedToken("Unexpected end variation token".to_string()))
                } else {
                    self.context_manager.end_branch();
                    Ok(())
                }
            }
            _ => {
                Err(PgnParseError::UnexpectedToken("Unexpected end variation token".to_string()))
            }
        }
    }

    fn process_comment(&mut self, _comment: PgnComment) -> Result<(), PgnParseError> {
        Ok(()) // TODO
    }

    fn process_result(&mut self, result: Option<Color>) -> Result<(), PgnParseError> {
        match self.parse_state {
            PgnParseState::Moves { move_number_just_seen: false } => {
                self.parse_state = PgnParseState::ResultFound;
                Ok(())
            }
            _ => {
                Err(PgnParseError::UnexpectedToken("Unexpected result token".to_string()))
            }
        }
    }

    fn process_incomplete(&mut self) -> Result<(), PgnParseError> {
        match self.parse_state {
            PgnParseState::Moves { move_number_just_seen: false } => {
                self.parse_state = PgnParseState::ResultFound;
                Ok(())
            }
            _ => {
                Err(PgnParseError::UnexpectedToken("Unexpected incomplete token".to_string()))
            }
        }
    }
}