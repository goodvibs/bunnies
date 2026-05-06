use crate::Color;
use crate::pgn::buffered_position_context::{
    PgnBufferedPositionContext, PgnBufferedPositionContextDyn,
};
use crate::pgn::move_tree_node::MoveTreeNode;
use crate::pgn::position_context::PgnPositionContext;
use crate::position::Position;
use std::cell::RefCell;
use std::rc::Rc;

pub struct PgnBufferedPositionBrancher<const N: usize> {
    pub current_and_previous: PgnBufferedPositionContextDyn<N>,
    pub stack: Vec<PgnBufferedPositionContextDyn<N>>,
}

impl<const N: usize> PgnBufferedPositionBrancher<N> {
    pub fn new(
        root_node: &Rc<RefCell<MoveTreeNode<N, { Color::White }, { Color::Black }>>>,
        initial_state: Position<N, { Color::White }>,
    ) -> PgnBufferedPositionBrancher<N> {
        PgnBufferedPositionBrancher {
            current_and_previous: PgnBufferedPositionContextDyn::White(PgnBufferedPositionContext {
                current: PgnPositionContext::<N, { Color::White }, { Color::Black }> {
                    node: Rc::clone(root_node),
                    state_after_move: initial_state,
                },
                previous: None,
            }),
            stack: Vec::new(),
        }
    }

    pub fn create_branch_from_previous(&mut self) {
        let new_context = self
            .current_and_previous
            .previous_as_current()
            .expect("No previous node to create branch from");
        let old_context = std::mem::replace(&mut self.current_and_previous, new_context);
        self.stack.push(old_context);
    }

    pub fn end_branch(&mut self) {
        let previous_context = self.stack.pop().expect("No previous context to return to");
        self.current_and_previous = previous_context;
    }
}
