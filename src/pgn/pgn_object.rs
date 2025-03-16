use indexmap::IndexMap;
use crate::pgn::move_tree_node::MoveTreeNode;

pub struct PgnObject {
    pub tree_root: MoveTreeNode,
    pub tags: IndexMap<String, String>,
}

impl PgnObject {
    pub fn new() -> PgnObject {
        PgnObject {
            tags: IndexMap::new(),
            tree_root: MoveTreeNode::new_root(None),
        }
    }

    pub fn add_tag(&mut self, key: String, value: String) {
        self.tags.insert(key, value);
    }
}