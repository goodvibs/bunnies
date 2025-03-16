use crate::r#move::Move;

#[derive(Debug, Clone)]
pub struct MoveData {
    pub mv: Move,
    pub annotation: Option<String>,
    pub nag: Option<u8>,
}

pub struct MoveTreeNode {
    pub move_data: Option<MoveData>, // None for the root node
    pub comment: Option<String>, // Root node may have a comment, so this is not part of MoveData
    pub continuations: Vec<Box<MoveTreeNode>>,
}

impl MoveTreeNode {
    pub fn new_root(comment: Option<String>) -> MoveTreeNode {
        MoveTreeNode {
            move_data: None,
            comment,
            continuations: Vec::new(),
        }
    }
    pub fn new(move_data: MoveData, comment: Option<String>) -> MoveTreeNode {
        MoveTreeNode {
            move_data: Some(move_data),
            comment,
            continuations: Vec::new(),
        }
    }

    pub fn add_continuation(&mut self, continuation: MoveTreeNode) {
        self.continuations.push(Box::new(continuation));
    }

    pub fn has_continuations(&self) -> bool {
        !self.continuations.is_empty()
    }
}