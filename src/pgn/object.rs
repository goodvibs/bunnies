use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use indexmap::IndexMap;
use crate::pgn::move_tree_node::MoveTreeNode;
use crate::pgn::rendering_config::PgnRenderingConfig;
use crate::state::GameState;

/// Represents a parsed PGN string.
pub struct PgnObject {
    pub tree_root: Rc<RefCell<MoveTreeNode>>,
    pub tags: IndexMap<String, String>,
}

impl Default for PgnObject {
    fn default() -> Self {
        PgnObject::new()
    }
}

impl PgnObject {
    /// Creates a new PgnObject representing an empty PGN string.
    pub fn new() -> PgnObject {
        PgnObject {
            tags: IndexMap::new(),
            tree_root: Rc::new(RefCell::new(MoveTreeNode::new_root(None)))
        }
    }

    /// Adds a tag to the PGN object.
    pub fn add_tag(&mut self, key: String, value: String) {
        self.tags.insert(key, value);
    }

    /// Returns a PGN string representation, rendered with the given configuration.
    pub fn render(&self, include_variations: bool, config: PgnRenderingConfig) -> String {
        let mut result = String::new();
        for (key, value) in self.tags.iter() {
            result.push_str(&format!("[{} \"{}\"]\n", key, value));
        }
        result.push_str(&self.tree_root.borrow().render(
            GameState::initial(),
            &[],
            include_variations,
            config,
            0,
            false
        ));
        result
    }
}

impl Display for PgnObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.render(true, PgnRenderingConfig::default()))
    }
}