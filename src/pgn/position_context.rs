use crate::Color;
use crate::pgn::move_tree_node::MoveTreeNode;
use crate::position::Position;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub(crate) struct PgnPositionContext<const N: usize, const STM: Color, const OPP: Color> {
    pub(crate) node: Rc<RefCell<MoveTreeNode<N, STM, OPP>>>,
    pub(crate) state_after_move: Position<N, STM>,
}
