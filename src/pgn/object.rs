use crate::Color;
use crate::pgn::move_tree_node::MoveTreeNode;
use crate::pgn::rendering_config::PgnRenderingConfig;
use crate::position::Position;
use indexmap::IndexMap;
use std::cell::RefCell;
use std::rc::Rc;

/// Represents a parsed PGN string.
pub struct PgnObject<const N: usize> {
    pub(crate) tree_root: Rc<RefCell<MoveTreeNode<N, { Color::White }, { Color::Black }>>>,
    pub tags: IndexMap<String, String>,
}

impl<const N: usize> Default for PgnObject<N> {
    fn default() -> Self {
        PgnObject::new()
    }
}

impl<const N: usize> PgnObject<N> {
    /// Creates a new PgnObject representing an empty PGN string.
    pub fn new() -> PgnObject<N> {
        PgnObject {
            tags: IndexMap::new(),
            tree_root: Rc::new(RefCell::new(
                MoveTreeNode::<N, { Color::White }, { Color::Black }>::new_root(
                None,
            ))),
        }
    }

    /// Adds a tag to the PGN object.
    pub fn add_tag(&mut self, key: String, value: String) {
        self.tags.insert(key, value);
    }

    /// Returns a PGN string representation, rendered with the given configuration.
    /// `N` must match the [`Position<N>`] capacity used when parsing (longest line must fit).
    pub fn render(&self, include_variations: bool, config: PgnRenderingConfig) -> String {
        let mut result = String::new();
        for (key, value) in self.tags.iter() {
            result.push_str(&format!("[{} \"{}\"]\n", key, value));
        }
        result.push_str(&self.tree_root.borrow().render(
            Position::<N, { Color::White }>::initial(),
            &[],
            include_variations,
            config,
            0,
            false,
        ));
        result
    }
}
