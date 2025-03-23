//! Contains the State struct, which is the main struct for representing a position in a chess game.

use std::cell::RefCell;
use std::rc::Rc;
use crate::utils::{Bitboard, Color, PieceType};
use crate::state::{Board, GameContext, GameResult};

/// A struct containing all the information needed to represent a position in a chess game.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct State {
    pub board: Board,
    pub side_to_move: Color,
    pub halfmove: u16,
    pub result: GameResult,
    pub context: Rc<RefCell<GameContext>>,
}

impl State {
    /// Creates an initial state with the standard starting position.
    pub fn initial() -> State {
        let board = Board::initial();
        let zobrist_hash = board.zobrist_hash;
        State {
            board,
            side_to_move: Color::White,
            halfmove: 0,
            result: GameResult::None,
            context: Rc::new(RefCell::new(GameContext::initial(zobrist_hash))),
        }
    }

    /// Gets the fullmove number of the position. 1-based.
    pub const fn get_fullmove(&self) -> u16 {
        self.halfmove / 2 + 1
    }

    pub fn current_side_attacks(&self) -> Bitboard {
        match self.context.borrow().current_side_attacks {
            0 => self.board.calc_attacks_mask(self.side_to_move),
            attacks => attacks,
        }
    }

    pub fn opposite_side_attacks(&self) -> Bitboard {
        match &self.context.borrow().previous {
            // Some(previous) => previous.borrow().current_side_attacks,
            _ => self.board.calc_attacks_mask(self.side_to_move.flip())
        }
    }

    pub fn is_current_side_in_check(&self) -> bool {
        let kings_bb = self.board.piece_type_masks[PieceType::King as usize];
        let current_side_bb = self.board.color_masks[self.side_to_move as usize];
        kings_bb & current_side_bb & self.opposite_side_attacks() != 0
    }

    pub fn is_opposite_side_in_check(&self) -> bool {
        let kings_bb = self.board.piece_type_masks[PieceType::King as usize];
        let opposite_side_bb = self.board.color_masks[self.side_to_move.flip() as usize];
        kings_bb & opposite_side_bb & self.current_side_attacks() != 0
    }

    pub fn update_insufficient_material(&mut self, use_uscf_rules: bool) {
        if self.board.are_both_sides_insufficient_material(use_uscf_rules) {
            self.result = GameResult::InsufficientMaterial;
        }
    }

    pub fn update_halfmove_clock(&mut self) {
        if self.context.borrow().halfmove_clock < 100 {
            self.result = GameResult::FiftyMoveRule;
        }
    }

    pub fn update_threefold_repetition(&mut self) {
        if self.context.borrow().has_threefold_repetition_occurred() {
            self.result = GameResult::ThreefoldRepetition;
        }
    }
}