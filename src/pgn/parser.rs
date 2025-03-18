use logos::{Lexer, Logos};
use crate::color::Color;
use crate::pgn::token_types::PgnCastlingMove;
use crate::pgn::parsing_error::PgnParsingError;
use crate::pgn::pgn_token::{PgnToken};
use crate::pgn::pgn_object::PgnObject;
use crate::pgn::parsing_state::PgnParsingState;
use crate::pgn::pgn_buffered_position_brancher::PgnBufferedPositionBrancher;
use crate::pgn::pgn_move_data::PgnMoveData;
use crate::pgn::token_types::PgnComment;
use crate::pgn::token_types::PgnNonCastlingMove;
use crate::pgn::token_types::PgnMove;
use crate::pgn::token_types::PgnMoveNumber;
use crate::pgn::token_types::PgnTag;
use crate::state::{State};

pub struct PgnParser<'a> {
    pub lexer: Lexer<'a, PgnToken>,
    pub parse_state: PgnParsingState,
    pub constructed_object: PgnObject,
    pub buffered_position_manager: PgnBufferedPositionBrancher
}

impl<'a> PgnParser<'a> {
    pub fn new(pgn: &str) -> PgnParser {
        let lexer = PgnToken::lexer(pgn);
        let pgn_object = PgnObject::new();
        let current_node = &pgn_object.tree_root;
        let buffered_position_manager = PgnBufferedPositionBrancher::new(&current_node, State::initial());
        PgnParser {
            lexer,
            parse_state: PgnParsingState::Tags,
            constructed_object: pgn_object,
            buffered_position_manager,
        }
    }

    pub fn parse(&mut self) -> Result<(), PgnParsingError> {
        while let Some(token) = self.lexer.next() {
            let token = match token {
                Ok(token) => token,
                Err(e) => return Err(PgnParsingError::LexingError(format!("Error while lexing: {:?}", e))),
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

        if !self.buffered_position_manager.stack.is_empty() {
            Err(PgnParsingError::UnexpectedEndOfInput("Unclosed variation".to_string()))
        } else if let PgnParsingState::Moves { move_number_just_seen: true } = self.parse_state {
            Err(PgnParsingError::UnexpectedEndOfInput("End of input after move number".to_string()))
        } else {
            Ok(())
        }
    }

    fn process_tag(&mut self, tag: PgnTag) -> Result<(), PgnParsingError> {
        if self.parse_state != PgnParsingState::Tags {
            return Err(PgnParsingError::UnexpectedToken(format!("Unexpected tag token: {:?}", tag)));
        }
        self.constructed_object.add_tag(tag.name, tag.value);
        Ok(())
    }

    fn process_move_number(&mut self, pgn_move_number: PgnMoveNumber) -> Result<(), PgnParsingError> {
        match self.parse_state {
            PgnParsingState::Tags => {
                self.parse_state = PgnParsingState::Moves { move_number_just_seen: false };
                self.process_move_number(pgn_move_number)
            }
            PgnParsingState::Moves { move_number_just_seen } => {
                if move_number_just_seen {
                    Err(PgnParsingError::UnexpectedToken(format!("Unexpected move number token: {:?}", pgn_move_number)))
                }
                else {
                    let expected_fullmove = self.buffered_position_manager.current_and_previous.current.state_after_move.get_fullmove();
                    if pgn_move_number.fullmove_number == expected_fullmove {
                        self.parse_state = PgnParsingState::Moves { move_number_just_seen: true };
                        Ok(())
                    } else {
                        Err(PgnParsingError::IncorrectMoveNumber(format!("{:?}", pgn_move_number)))
                    }
                }
            }
            PgnParsingState::ResultFound => {
                Err(PgnParsingError::UnexpectedToken(format!("Unexpected move number token: {:?}", pgn_move_number)))
            }
        }
    }

    fn process_move<PgnMoveType: PgnMove>(&mut self, pgn_move: PgnMoveType) -> Result<(), PgnParsingError> {
        match self.parse_state {
            PgnParsingState::Moves { move_number_just_seen } => {
                let current_state = &self.buffered_position_manager.current_and_previous.current.state_after_move;
                if !move_number_just_seen && current_state.side_to_move == Color::White {
                    return Err(PgnParsingError::UnexpectedToken(format!("Unexpected move token: {:?}", pgn_move)));
                }
                let possible_moves = current_state.calc_legal_moves();

                let mut matched_move = None;
                for possible_move in possible_moves {
                    if pgn_move.matches_move(possible_move, current_state) {
                        if matched_move.is_some() {
                            return Err(PgnParsingError::AmbiguousMove(format!("Ambiguous move: {:?}", pgn_move)));
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
                    let move_data = PgnMoveData {
                        mv: matched_move,
                        annotation: pgn_move.get_common_move_info().annotation.clone(),
                        nag: pgn_move.get_common_move_info().nag.clone(),
                    };
                    self.buffered_position_manager.current_and_previous.append_new_move(move_data, new_state);
                    self.parse_state = PgnParsingState::Moves { move_number_just_seen: false };
                    Ok(())
                } else {
                    Err(PgnParsingError::IllegalMove(format!("Illegal move: {:?}", pgn_move)))
                }
            }
            _ => {
                Err(PgnParsingError::UnexpectedToken(format!("Unexpected move token: {:?}", pgn_move)))
            }
        }
    }

    fn process_start_variation(&mut self) -> Result<(), PgnParsingError> {
        match self.parse_state {
            PgnParsingState::Moves { move_number_just_seen: false } => {
                if self.buffered_position_manager.current_and_previous.previous.is_none() {
                    Err(PgnParsingError::UnexpectedToken("Unexpected start variation token".to_string()))
                } else {
                    self.buffered_position_manager.create_branch_from_previous();
                    Ok(())
                }
            }
            _ => {
                Err(PgnParsingError::UnexpectedToken("Unexpected start variation token".to_string()))
            }
        }
    }

    fn process_end_variation(&mut self) -> Result<(), PgnParsingError> {
        match self.parse_state {
            PgnParsingState::Moves { move_number_just_seen: false } => {
                if self.buffered_position_manager.stack.is_empty() {
                    Err(PgnParsingError::UnexpectedToken("Unexpected end variation token".to_string()))
                } else {
                    self.buffered_position_manager.end_branch();
                    Ok(())
                }
            }
            _ => {
                Err(PgnParsingError::UnexpectedToken("Unexpected end variation token".to_string()))
            }
        }
    }

    fn process_comment(&mut self, _comment: PgnComment) -> Result<(), PgnParsingError> {
        Ok(()) // TODO
    }

    fn process_result(&mut self, result: Option<Color>) -> Result<(), PgnParsingError> {
        match self.parse_state {
            PgnParsingState::Moves { move_number_just_seen: false } => {
                self.parse_state = PgnParsingState::ResultFound;
                Ok(())
            }
            _ => {
                Err(PgnParsingError::UnexpectedToken("Unexpected result token".to_string()))
            }
        }
    }

    fn process_incomplete(&mut self) -> Result<(), PgnParsingError> {
        match self.parse_state {
            PgnParsingState::Moves { move_number_just_seen: false } => {
                self.parse_state = PgnParsingState::ResultFound;
                Ok(())
            }
            _ => {
                Err(PgnParsingError::UnexpectedToken("Unexpected incomplete token".to_string()))
            }
        }
    }
}