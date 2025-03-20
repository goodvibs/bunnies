//! Contains the State struct, which is the main struct for representing a position in a chess game.

use std::cell::RefCell;
use std::rc::Rc;
use crate::utils::Color;
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
    /// Creates a blank state with no pieces on the board.
    pub fn blank() -> State {
        let board = Board::blank();
        let zobrist_hash = board.zobrist_hash;
        State {
            board,
            side_to_move: Color::White,
            halfmove: 0,
            result: GameResult::None,
            context: Rc::new(RefCell::new(GameContext::initial_no_castling(zobrist_hash))),
        }
    }

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