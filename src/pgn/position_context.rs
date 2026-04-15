use crate::pgn::move_tree_node::MoveTreeNode;
use crate::position::TypedPosition;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub(crate) struct PgnPositionContext<const N: usize> {
    pub(crate) node: Rc<RefCell<MoveTreeNode>>,
    pub(crate) state_after_move: TypedPosition<N>,
}
