use crate::pgn::MoveTreeNode;
use crate::pgn::move_data::PgnMoveData;
use crate::pgn::position_context::PgnPositionContext;
use crate::state::GameState;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub(crate) struct PgnBufferedPositionContext {
    pub(crate) current: PgnPositionContext,
    pub(crate) previous: Option<PgnPositionContext>,
}

impl PgnBufferedPositionContext {
    pub(crate) fn append_new_move(&mut self, new_move_data: PgnMoveData, new_state: GameState) {
        let new_node = Rc::new(RefCell::new(MoveTreeNode::new(new_move_data, None)));
        self.current.node.borrow_mut().add_continuation(&new_node);

        // Create the new value we want to assign to self.current
        let new_current = PgnPositionContext {
            node: new_node,
            state_after_move: new_state,
        };

        // Replace self.current with new_current and get the old value
        let old_current = std::mem::replace(&mut self.current, new_current);

        // Set previous to the old current value
        self.previous = Some(old_current);
    }
}
