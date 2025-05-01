use crate::Color;
use crate::Piece;
use crate::r#move::{Move};
use crate::pgn::move_data::PgnMoveData;
use crate::pgn::rendering_config::PgnRenderingConfig;
use crate::position::{GameResult, Position};
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) struct MoveTreeNode {
    move_data: Option<PgnMoveData>, // None for the root node
    comment: Option<String>,        // Root node may have a comment, so this is not part of MoveData
    continuations: Vec<Rc<RefCell<MoveTreeNode>>>,
}

impl MoveTreeNode {
    pub(crate) fn new_root(comment: Option<String>) -> MoveTreeNode {
        MoveTreeNode {
            move_data: None,
            comment,
            continuations: Vec::new(),
        }
    }
    pub(crate) fn new(move_data: PgnMoveData, comment: Option<String>) -> MoveTreeNode {
        MoveTreeNode {
            move_data: Some(move_data),
            comment,
            continuations: Vec::new(),
        }
    }

    pub(crate) fn add_continuation(&mut self, continuation: &Rc<RefCell<MoveTreeNode>>) {
        self.continuations.push(Rc::clone(continuation));
    }

    pub(crate) fn has_continuations(&self) -> bool {
        !self.continuations.is_empty()
    }

    pub(crate) fn get_main_continuation(&self) -> Option<Rc<RefCell<MoveTreeNode>>> {
        self.continuations.first().map(|c| Rc::clone(c))
    }

    pub(crate) fn has_multiple_continuations(&self) -> bool {
        self.continuations.len() > 1
    }

    pub(crate) fn get_alternative_continuations(&self) -> Vec<Rc<RefCell<MoveTreeNode>>> {
        self.continuations
            .iter()
            .skip(1)
            .map(|c| Rc::clone(c))
            .collect()
    }

    pub(crate) fn render(
        &self,
        mut state: Position,
        last_continuations: &[Rc<RefCell<MoveTreeNode>>],
        include_variations: bool,
        config: PgnRenderingConfig,
        depth: u16,
        remind_fullmove: bool,
    ) -> String {
        let rendered_last_continuations = {
            let mut result = String::new();
            for continuation in last_continuations {
                let rendered_continuation = &continuation.borrow().render(
                    state.clone(),
                    &[],
                    include_variations,
                    config,
                    depth + 1,
                    true,
                );
                result += &format!(" ({})", rendered_continuation);
            }
            result
        };

        let rendered_move = if let Some(move_data) = &self.move_data {
            let mv = move_data.mv;
            let mv_source = mv.source();
            let mv_dest = mv.destination();
            let moved_piece = state.board.piece_at(mv_source);
            let side_to_move = state.side_to_move;

            // Add move number for white's move or at the start of a variation
            let move_number_str = if side_to_move == Color::White {
                format!("{}. ", state.get_fullmove())
            } else if remind_fullmove {
                format!("{}... ", state.get_fullmove())
            } else {
                "".to_string()
            };

            let disambiguation_str = match moved_piece {
                Piece::Pawn | Piece::King => "".to_string(),
                Piece::Null => panic!("Invalid piece type"),
                _ => {
                    let all_moves = state.moves();
                    let all_other_moves: Vec<Move> =
                        all_moves.iter().filter(|m| **m != mv).cloned().collect();
                    let disambiguation_moves: Vec<Move> = all_other_moves
                        .iter()
                        .filter(|m| {
                            m.destination() == mv_dest
                                && state.board.piece_at(m.source()) == moved_piece
                        })
                        .cloned()
                        .collect::<Vec<Move>>();
                    match disambiguation_moves.len() {
                        0 => "".to_string(),
                        _ => {
                            let file = mv_source.file();
                            let rank = mv_source.rank();
                            let is_file_ambiguous = disambiguation_moves
                                .iter()
                                .any(|m| m.source().file() == file);
                            let is_rank_ambiguous = disambiguation_moves
                                .iter()
                                .any(|m| m.source().rank() == rank);
                            match (is_file_ambiguous, is_rank_ambiguous) {
                                (true, true) => mv_source.to_string(),
                                (true, false) => mv_source.rank_char().to_string(),
                                (false, true) => mv_source.file_char().to_string(),
                                (false, false) => "".to_string(),
                            }
                        }
                    }
                }
            };

            let is_capture = match mv.flag() {
                flag if flag.is_guaranteed_capture() => true,
                flag if flag.is_guaranteed_non_capture() => false,
                _ => {
                    state.board.piece_at(mv_dest) != Piece::Null
                }
            };
            state.make_move(mv); // if attacks_mask is 0, then it will be filled in automatically
            let is_check = state.is_current_side_in_check();
            let is_checkmate = match is_check {
                true => {
                    let all_moves = state.moves();
                    let is_checkmate = all_moves.is_empty();
                    if is_checkmate {
                        state.result = GameResult::Checkmate;
                    }
                    is_checkmate
                }
                false => false,
            };

            // Combine move number and move
            move_number_str
                + &move_data.render(
                    disambiguation_str.as_str(),
                    is_check,
                    is_checkmate,
                    is_capture,
                    config.include_annotations,
                    config.include_nags,
                )
        } else {
            "".to_string()
        };

        let rendered_comment = if config.include_comments {
            if let Some(comment) = &self.comment {
                format!(" {{ {} }}", comment)
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };

        let up_till_now = format!(
            "{}{}{}",
            rendered_move, rendered_comment, rendered_last_continuations
        );

        if self.has_continuations() {
            let main_continuation = self.get_main_continuation().unwrap();
            let alternative_continuations = match include_variations {
                true => self.get_alternative_continuations(),
                false => Vec::with_capacity(0),
            };
            let rendered_main_continuation = main_continuation.borrow().render(
                state,
                &alternative_continuations,
                include_variations,
                config,
                depth + 1,
                !last_continuations.is_empty(),
            );

            // Add appropriate spacing before the next move
            if up_till_now.is_empty() {
                rendered_main_continuation
            } else {
                format!("{} {}", up_till_now, rendered_main_continuation)
            }
        } else {
            up_till_now
        }
    }
}
