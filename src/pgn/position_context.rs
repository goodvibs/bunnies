use crate::pgn::move_tree_node::MoveTreeNode;
use crate::state::GameState;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub(crate) struct PgnPositionContext {
    pub(crate) node: Rc<RefCell<MoveTreeNode>>,
    pub(crate) state_after_move: GameState,
}
