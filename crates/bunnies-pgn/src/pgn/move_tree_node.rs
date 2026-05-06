use crate::Color;
use crate::Piece;
use crate::r#move::{MoveFlag, MoveList};
use crate::pgn::move_data::PgnMoveData;
use crate::pgn::rendering_config::PgnRenderingConfig;
use crate::position::Position;
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) struct MoveTreeNode<const N: usize, const STM: Color, const OPP: Color> {
    move_data: Option<PgnMoveData>, // None for the root node
    comment: Option<String>,        // Root node may have a comment, so this is not part of MoveData
    continuations: Vec<Rc<RefCell<MoveTreeNode<N, OPP, STM>>>>,
}

impl<const N: usize, const STM: Color, const OPP: Color> MoveTreeNode<N, STM, OPP> {
    pub(crate) fn new_root(comment: Option<String>) -> MoveTreeNode<N, STM, OPP> {
        MoveTreeNode {
            move_data: None,
            comment,
            continuations: Vec::new(),
        }
    }
    pub(crate) fn new(
        move_data: PgnMoveData,
        comment: Option<String>,
    ) -> MoveTreeNode<N, STM, OPP> {
        MoveTreeNode {
            move_data: Some(move_data),
            comment,
            continuations: Vec::new(),
        }
    }

    pub(crate) fn add_continuation(
        &mut self,
        continuation: &Rc<RefCell<MoveTreeNode<N, OPP, STM>>>,
    ) {
        self.continuations.push(Rc::clone(continuation));
    }

    pub(crate) fn has_continuations(&self) -> bool {
        !self.continuations.is_empty()
    }

    pub(crate) fn get_main_continuation(&self) -> Option<Rc<RefCell<MoveTreeNode<N, OPP, STM>>>> {
        self.continuations.first().map(|c| Rc::clone(c))
    }

    pub(crate) fn has_multiple_continuations(&self) -> bool {
        self.continuations.len() > 1
    }

    pub(crate) fn get_alternative_continuations(
        &self,
    ) -> Vec<Rc<RefCell<MoveTreeNode<N, OPP, STM>>>> {
        self.continuations
            .iter()
            .skip(1)
            .map(|c| Rc::clone(c))
            .collect()
    }

    fn render_white(
        &self,
        state: Position<N, { Color::White }>,
        last_continuations: &[Rc<RefCell<MoveTreeNode<N, STM, OPP>>>],
        include_variations: bool,
        config: PgnRenderingConfig,
        depth: u16,
        _remind_fullmove: bool,
    ) -> String {
        let rendered_last_continuations = {
            let mut result = String::new();
            for continuation in last_continuations {
                let rendered_continuation = &continuation.borrow().render_white(
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

        let mut next_state_after_move: Option<Position<N, { Color::Black }>> = None;
        let mut moved_here = false;
        let rendered_move = if let Some(move_data) = &self.move_data {
            moved_here = true;
            let mv = move_data.mv;
            let mv_source = mv.source();
            let mv_dest = mv.destination();
            let moved_piece = state.board.piece_at(mv_source);

            // Add move number for white's move or at the start of a variation
            let move_number_str = format!("{}. ", state.get_fullmove());

            let disambiguation_str = match moved_piece {
                Piece::Pawn | Piece::King => "".to_string(),
                Piece::Null => panic!("Invalid piece type"),
                _ => {
                    let mut legal = MoveList::new();
                    state.generate_legal_moves(&mut legal);
                    let mut disambiguation_moves: MoveList = MoveList::new();
                    for m in legal.as_slice().iter().copied() {
                        if m == mv {
                            continue;
                        }
                        if m.destination() == mv_dest
                            && state.board.piece_at(m.source()) == moved_piece
                        {
                            disambiguation_moves.push(m);
                        }
                    }
                    match disambiguation_moves.len() {
                        0 => "".to_string(),
                        _ => {
                            let file = mv_source.file();
                            let rank = mv_source.rank();
                            let is_file_ambiguous = disambiguation_moves
                                .as_slice()
                                .iter()
                                .any(|m| m.source().file() == file);
                            let is_rank_ambiguous = disambiguation_moves
                                .as_slice()
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
                MoveFlag::EnPassant => true,
                MoveFlag::Castling => false,
                MoveFlag::NormalMove | MoveFlag::Promotion => {
                    state.board.piece_at(mv_dest) != Piece::Null
                }
            };
            let (next_position, is_check, is_checkmate) = apply_white_move(state.clone(), mv);
            next_state_after_move = Some(next_position);

            // Combine move number and move
            move_number_str
                + &move_data.render(
                    moved_piece,
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
            let rendered_main_continuation = if moved_here {
                main_continuation.borrow().render_black(
                    next_state_after_move.expect("state after move"),
                    &alternative_continuations,
                    include_variations,
                    config,
                    depth + 1,
                    !last_continuations.is_empty(),
                )
            } else {
                main_continuation.borrow().render_white(
                    state,
                    &alternative_continuations,
                    include_variations,
                    config,
                    depth + 1,
                    !last_continuations.is_empty(),
                )
            };

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

    fn render_black(
        &self,
        state: Position<N, { Color::Black }>,
        last_continuations: &[Rc<RefCell<MoveTreeNode<N, STM, OPP>>>],
        include_variations: bool,
        config: PgnRenderingConfig,
        depth: u16,
        remind_fullmove: bool,
    ) -> String {
        let rendered_last_continuations = {
            let mut result = String::new();
            for continuation in last_continuations {
                let rendered_continuation = &continuation.borrow().render_black(
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

        let mut next_state_after_move: Option<Position<N, { Color::White }>> = None;
        let mut moved_here = false;
        let rendered_move = if let Some(move_data) = &self.move_data {
            moved_here = true;
            let mv = move_data.mv;
            let mv_source = mv.source();
            let mv_dest = mv.destination();
            let moved_piece = state.board.piece_at(mv_source);

            let move_number_str = if remind_fullmove {
                format!("{}... ", state.get_fullmove())
            } else {
                "".to_string()
            };

            let disambiguation_str = match moved_piece {
                Piece::Pawn | Piece::King => "".to_string(),
                Piece::Null => panic!("Invalid piece type"),
                _ => {
                    let mut legal = MoveList::new();
                    state.generate_legal_moves(&mut legal);
                    let mut disambiguation_moves: MoveList = MoveList::new();
                    for m in legal.as_slice().iter().copied() {
                        if m == mv {
                            continue;
                        }
                        if m.destination() == mv_dest
                            && state.board.piece_at(m.source()) == moved_piece
                        {
                            disambiguation_moves.push(m);
                        }
                    }
                    match disambiguation_moves.len() {
                        0 => "".to_string(),
                        _ => {
                            let file = mv_source.file();
                            let rank = mv_source.rank();
                            let is_file_ambiguous = disambiguation_moves
                                .as_slice()
                                .iter()
                                .any(|m| m.source().file() == file);
                            let is_rank_ambiguous = disambiguation_moves
                                .as_slice()
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
                MoveFlag::EnPassant => true,
                MoveFlag::Castling => false,
                MoveFlag::NormalMove | MoveFlag::Promotion => {
                    state.board.piece_at(mv_dest) != Piece::Null
                }
            };
            let (next_position, is_check, is_checkmate) = apply_black_move(state.clone(), mv);
            next_state_after_move = Some(next_position);

            move_number_str
                + &move_data.render(
                    moved_piece,
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
            let rendered_main_continuation = if moved_here {
                main_continuation.borrow().render_white(
                    next_state_after_move.expect("state after move"),
                    &alternative_continuations,
                    include_variations,
                    config,
                    depth + 1,
                    !last_continuations.is_empty(),
                )
            } else {
                main_continuation.borrow().render_black(
                    state,
                    &alternative_continuations,
                    include_variations,
                    config,
                    depth + 1,
                    !last_continuations.is_empty(),
                )
            };

            if up_till_now.is_empty() {
                rendered_main_continuation
            } else {
                format!("{} {}", up_till_now, rendered_main_continuation)
            }
        } else {
            up_till_now
        }
    }

    pub(crate) fn render(
        &self,
        state: Position<N, { Color::White }>,
        last_continuations: &[Rc<RefCell<MoveTreeNode<N, STM, OPP>>>],
        include_variations: bool,
        config: PgnRenderingConfig,
        depth: u16,
        remind_fullmove: bool,
    ) -> String {
        self.render_white(
            state,
            last_continuations,
            include_variations,
            config,
            depth,
            remind_fullmove,
        )
    }
}

fn apply_white_move<const N: usize>(
    mut state: Position<N, { Color::White }>,
    mv: crate::r#move::Move,
) -> (Position<N, { Color::Black }>, bool, bool) {
    state.make_move(mv);
    let next = state.rebrand_stm::<{ Color::Black }>();
    let is_check = next.is_current_side_in_check();
    let is_checkmate = if is_check {
        let mut replies = MoveList::new();
        next.generate_legal_moves(&mut replies);
        replies.is_empty()
    } else {
        false
    };
    (next, is_check, is_checkmate)
}

fn apply_black_move<const N: usize>(
    mut state: Position<N, { Color::Black }>,
    mv: crate::r#move::Move,
) -> (Position<N, { Color::White }>, bool, bool) {
    state.make_move(mv);
    let next = state.rebrand_stm::<{ Color::White }>();
    let is_check = next.is_current_side_in_check();
    let is_checkmate = if is_check {
        let mut replies = MoveList::new();
        next.generate_legal_moves(&mut replies);
        replies.is_empty()
    } else {
        false
    };
    (next, is_check, is_checkmate)
}
