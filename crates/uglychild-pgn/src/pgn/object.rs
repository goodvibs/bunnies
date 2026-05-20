//! Parsed PGN game object with tag pairs and move tree.

use std::{cell::RefCell, rc::Rc};

use indexmap::IndexMap;

use crate::{
    Color,
    pgn::{move_tree_node::MoveTreeNode, rendering_config::PgnRenderingConfig},
    position::Position,
};

/// A fully parsed PGN game with metadata tags and move tree.
///
/// `N` is the position stack capacity that must fit the longest variation
/// in the parsed game. Use [`PgnParser`](crate::pgn::PgnParser) to construct.
pub struct PgnObject<const N: usize> {
    pub(crate) tree_root: Rc<RefCell<MoveTreeNode<N, { Color::White }, { Color::Black }>>>,
    /// PGN tag pairs (e.g., `[Event "World Championship"]`).
    pub tags: IndexMap<String, String>,
}

impl<const N: usize> Default for PgnObject<N> {
    fn default() -> Self {
        PgnObject::new()
    }
}

impl<const N: usize> PgnObject<N> {
    /// Creates an empty PGN object with no moves and no tags.
    pub fn new() -> PgnObject<N> {
        PgnObject {
            tags: IndexMap::new(),
            tree_root: Rc::new(RefCell::new(MoveTreeNode::<
                N,
                { Color::White },
                { Color::Black },
            >::new_root(None))),
        }
    }

    /// Inserts a tag pair (overwrites existing key).
    pub fn add_tag(&mut self, key: String, value: String) {
        self.tags.insert(key, value);
    }

    /// Renders the game back to PGN format.
    ///
    /// Set `include_variations` to `false` for main line only.
    /// `N` must match the position stack capacity used during parsing.
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
