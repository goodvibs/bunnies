use std::cell::RefCell;
use std::rc::Rc;
use crate::pgn::move_tree_node::MoveTreeNode;
use crate::state::State;

#[derive(Clone)]
pub struct PgnPositionContext {
    pub node: Rc<RefCell<MoveTreeNode>>,
    pub state_after_move: State,
}