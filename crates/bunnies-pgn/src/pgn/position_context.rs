use std::{cell::RefCell, rc::Rc};

use crate::{Color, pgn::move_tree_node::MoveTreeNode, position::Position};

#[derive(Clone)]
pub(crate) struct PgnPositionContext<const N: usize, const STM: Color, const OPP: Color> {
    pub(crate) node: Rc<RefCell<MoveTreeNode<N, STM, OPP>>>,
    pub(crate) state_after_move: Position<N, STM>,
}
