use std::cell::RefCell;
use std::rc::Rc;
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
    pub continuations: Vec<Rc<RefCell<MoveTreeNode>>>,
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

    pub fn add_continuation(&mut self, continuation: &Rc<RefCell<MoveTreeNode>>) {
        self.continuations.push(Rc::clone(continuation));
    }

    pub fn has_continuations(&self) -> bool {
        !self.continuations.is_empty()
    }

    pub fn render(&self, include_variations: bool, depth: usize) -> String {
        let mut result = String::new();
        if let Some(move_data) = &self.move_data {
            result.push_str(&format!("{}{}", "  ".repeat(depth), move_data.mv.to_string()));
            if let Some(annotation) = &move_data.annotation {
                result.push_str(&format!(" {{ {} }}", annotation));
            }
            if let Some(nag) = move_data.nag {
                result.push_str(&format!(" ${}", nag));
            }
            result.push_str("\n");
        }
        if let Some(comment) = &self.comment {
            result.push_str(&format!("{}; {}\n", "  ".repeat(depth), comment));
        }
        for continuation in &self.continuations {
            result.push_str(&continuation.borrow().render(include_variations, depth + 1));
        }
        result
    }
}