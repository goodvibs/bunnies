use std::cell::RefCell;
use std::rc::Rc;
use indexmap::IndexMap;
use crate::pgn::move_tree_node::MoveTreeNode;
use crate::state::State;

pub struct PgnObject {
    pub tree_root: Rc<RefCell<MoveTreeNode>>,
    pub tags: IndexMap<String, String>,
}

impl PgnObject {
    pub fn new() -> PgnObject {
        PgnObject {
            tags: IndexMap::new(),
            tree_root: Rc::new(RefCell::new(MoveTreeNode::new_root(None)))
        }
    }

    pub fn add_tag(&mut self, key: String, value: String) {
        self.tags.insert(key, value);
    }

    pub fn render(&self, include_variations: bool, include_annotations: bool, include_nags: bool, include_comments: bool) -> String {
        let mut result = String::new();
        for (key, value) in self.tags.iter() {
            result.push_str(&format!("[{} \"{}\"]\n", key, value));
        }
        result.push_str(&self.tree_root.borrow().render(
            State::initial(),
            &[],
            include_variations,
            include_annotations,
            include_nags,
            include_comments,
            0
        ));
        result
    }
}