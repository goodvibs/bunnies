use logos::{Lexer, Logos};

use crate::{
    Color,
    r#move::MoveList,
    pgn::{
        buffered_position_brancher::PgnBufferedPositionBrancher,
        buffered_position_context::PgnBufferedPositionContextDyn,
        error::PgnError,
        move_data::PgnMoveData,
        object::PgnObject,
        parsing_state::PgnParsingState,
        token::PgnToken,
        token_types::{
            PgnCastlingMove,
            PgnComment,
            PgnMove,
            PgnMoveNumber,
            PgnNonCastlingMove,
            PgnTag,
        },
    },
    position::Position,
};

/// The main parser for PGN strings. `N` is the context stack capacity for [`Position<N>`] used
/// while parsing (must fit the longest half-move path in the game, including variations).
pub struct PgnParser<'a, const N: usize> {
    pub lexer: Lexer<'a, PgnToken>,
    pub parse_state: PgnParsingState,
    pub constructed_object: PgnObject<N>,
    buffered_position_manager: PgnBufferedPositionBrancher<N>,
}

impl<'a, const N: usize> PgnParser<'a, N> {
    pub fn new(pgn: &str) -> PgnParser<'_, N> {
        let lexer = PgnToken::lexer(pgn);
        let pgn_object = PgnObject::new();
        let current_node = &pgn_object.tree_root;
        let buffered_position_manager = PgnBufferedPositionBrancher::new(
            current_node,
            Position::<N, { Color::White }>::initial(),
        );
        PgnParser {
            lexer,
            parse_state: PgnParsingState::Tags,
            constructed_object: pgn_object,
            buffered_position_manager,
        }
    }

    pub fn parse(&mut self) -> Result<(), PgnError> {
        while let Some(token) = self.lexer.next() {
            let token = token?;
            match token {
                PgnToken::Tag(tag) => {
                    self.process_tag(tag)?;
                }
                PgnToken::MoveNumber(move_number) => {
                    self.process_move_number(move_number)?;
                }
                PgnToken::NonCastlingMove(pgn_move_value) => {
                    self.process_move::<PgnNonCastlingMove>(pgn_move_value)?;
                }
                PgnToken::CastlingMove(pgn_move_value) => {
                    self.process_move::<PgnCastlingMove>(pgn_move_value)?;
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
            Err(PgnError::UnexpectedEndOfInput(
                "Unclosed variation".to_string(),
            ))
        } else if let PgnParsingState::Moves {
            move_number_just_seen: true,
        } = self.parse_state
        {
            Err(PgnError::UnexpectedEndOfInput(
                "End of input after move number".to_string(),
            ))
        } else {
            Ok(())
        }
    }

    fn process_tag(&mut self, tag: PgnTag) -> Result<(), PgnError> {
        if self.parse_state != PgnParsingState::Tags {
            return Err(PgnError::UnexpectedToken(format!(
                "Unexpected tag token: {:?}",
                tag
            )));
        }
        self.constructed_object.add_tag(tag.name, tag.value);
        Ok(())
    }

    fn process_move_number(&mut self, pgn_move_number: PgnMoveNumber) -> Result<(), PgnError> {
        match self.parse_state {
            PgnParsingState::Tags => {
                self.parse_state = PgnParsingState::Moves {
                    move_number_just_seen: false,
                };
                self.process_move_number(pgn_move_number)
            }
            PgnParsingState::Moves {
                move_number_just_seen,
            } => {
                if move_number_just_seen {
                    Err(PgnError::UnexpectedToken(format!(
                        "Unexpected move number token: {:?}",
                        pgn_move_number
                    )))
                } else {
                    let expected_fullmove = self
                        .buffered_position_manager
                        .current_and_previous
                        .fullmove();
                    if pgn_move_number.fullmove_number == expected_fullmove {
                        self.parse_state = PgnParsingState::Moves {
                            move_number_just_seen: true,
                        };
                        Ok(())
                    } else {
                        Err(PgnError::IncorrectMoveNumber(format!(
                            "{:?}",
                            pgn_move_number
                        )))
                    }
                }
            }
            PgnParsingState::ResultFound => Err(PgnError::UnexpectedToken(format!(
                "Unexpected move number token: {:?}",
                pgn_move_number
            ))),
        }
    }

    fn process_move<PgnMoveType: PgnMove>(
        &mut self,
        pgn_move: PgnMoveType,
    ) -> Result<(), PgnError> {
        match self.parse_state {
            PgnParsingState::Moves {
                move_number_just_seen,
            } => {
                let current_state = &self.buffered_position_manager.current_and_previous;
                let side_to_move = current_state.side_to_move();
                if !move_number_just_seen && side_to_move == Color::White {
                    return Err(PgnError::UnexpectedToken(format!(
                        "Unexpected move token: {:?}",
                        pgn_move
                    )));
                }
                let mut possible_moves = MoveList::new();
                match current_state {
                    PgnBufferedPositionContextDyn::White(ctx) => ctx
                        .current
                        .state_after_move
                        .generate_moves(&mut possible_moves),
                    PgnBufferedPositionContextDyn::Black(ctx) => ctx
                        .current
                        .state_after_move
                        .generate_moves(&mut possible_moves),
                }

                let mut matched_move = None;
                for &possible_move in possible_moves.as_slice() {
                    let is_match = match current_state {
                        PgnBufferedPositionContextDyn::White(ctx) => pgn_move
                            .matches_move(possible_move, &ctx.current.state_after_move.board),
                        PgnBufferedPositionContextDyn::Black(ctx) => pgn_move
                            .matches_move(possible_move, &ctx.current.state_after_move.board),
                    };
                    if is_match {
                        if matched_move.is_some() {
                            return Err(PgnError::AmbiguousMove(format!(
                                "Ambiguous move: {:?}",
                                pgn_move
                            )));
                        } else {
                            matched_move = Some(possible_move);
                        }
                    }
                }

                if let Some(matched_move) = matched_move {
                    let move_data = PgnMoveData {
                        move_: matched_move,
                        annotation: pgn_move.get_common_move_info().annotation.clone(),
                        nag: pgn_move.get_common_move_info().nag,
                    };
                    let new_context = self
                        .buffered_position_manager
                        .current_and_previous
                        .clone()
                        .append_move(move_data);
                    self.buffered_position_manager.current_and_previous = new_context;
                    self.parse_state = PgnParsingState::Moves {
                        move_number_just_seen: false,
                    };
                    Ok(())
                } else {
                    Err(PgnError::IllegalMove(format!(
                        "Illegal move: {:?}",
                        pgn_move
                    )))
                }
            }
            _ => Err(PgnError::UnexpectedToken(format!(
                "Unexpected move token: {:?}",
                pgn_move
            ))),
        }
    }

    fn process_start_variation(&mut self) -> Result<(), PgnError> {
        match self.parse_state {
            PgnParsingState::Moves {
                move_number_just_seen: false,
            } => {
                if self
                    .buffered_position_manager
                    .current_and_previous
                    .previous_as_current()
                    .is_none()
                {
                    Err(PgnError::UnexpectedToken(
                        "Unexpected start variation token".to_string(),
                    ))
                } else {
                    self.buffered_position_manager.create_branch_from_previous();
                    Ok(())
                }
            }
            _ => Err(PgnError::UnexpectedToken(
                "Unexpected start variation token".to_string(),
            )),
        }
    }

    fn process_end_variation(&mut self) -> Result<(), PgnError> {
        match self.parse_state {
            PgnParsingState::Moves {
                move_number_just_seen: false,
            } => {
                if self.buffered_position_manager.stack.is_empty() {
                    Err(PgnError::UnexpectedToken(
                        "Unexpected end variation token".to_string(),
                    ))
                } else {
                    self.buffered_position_manager.end_branch();
                    Ok(())
                }
            }
            _ => Err(PgnError::UnexpectedToken(
                "Unexpected end variation token".to_string(),
            )),
        }
    }

    fn process_comment(&mut self, _comment: PgnComment) -> Result<(), PgnError> {
        Ok(()) // TODO
    }

    fn process_result(&mut self, _result: Option<Color>) -> Result<(), PgnError> {
        match self.parse_state {
            PgnParsingState::Moves {
                move_number_just_seen: false,
            } => {
                self.parse_state = PgnParsingState::ResultFound;
                Ok(())
            }
            _ => Err(PgnError::UnexpectedToken(
                "Unexpected result token".to_string(),
            )),
        }
    }

    fn process_incomplete(&mut self) -> Result<(), PgnError> {
        match self.parse_state {
            PgnParsingState::Moves {
                move_number_just_seen: false,
            } => {
                self.parse_state = PgnParsingState::ResultFound;
                Ok(())
            }
            _ => Err(PgnError::UnexpectedToken(
                "Unexpected incomplete token".to_string(),
            )),
        }
    }
}
