use crate::pgn::buffered_position_context::PgnBufferedPositionContext;
use crate::pgn::move_tree_node::MoveTreeNode;
use crate::pgn::position_context::PgnPositionContext;
use crate::state::GameState;
use std::cell::RefCell;
use std::rc::Rc;

pub struct PgnBufferedPositionBrancher {
    pub current_and_previous: PgnBufferedPositionContext,
    pub stack: Vec<PgnBufferedPositionContext>,
}

impl PgnBufferedPositionBrancher {
    pub fn new(
        root_node: &Rc<RefCell<MoveTreeNode>>,
        initial_state: GameState,
    ) -> PgnBufferedPositionBrancher {
        PgnBufferedPositionBrancher {
            current_and_previous: PgnBufferedPositionContext {
                current: PgnPositionContext {
                    node: Rc::clone(root_node),
                    state_after_move: initial_state,
                },
                previous: None,
            },
            stack: Vec::new(),
        }
    }

    pub fn create_branch_from_previous(&mut self) {
        let clone_of_previous = self
            .current_and_previous
            .previous
            .clone()
            .expect("No previous node to create branch from");
        let new_context = PgnBufferedPositionContext {
            current: clone_of_previous,
            previous: None,
        };
        let old_context = std::mem::replace(&mut self.current_and_previous, new_context);
        self.stack.push(old_context);
    }

    pub fn end_branch(&mut self) {
        let previous_context = self.stack.pop().expect("No previous context to return to");
        self.current_and_previous = previous_context;
    }
}
