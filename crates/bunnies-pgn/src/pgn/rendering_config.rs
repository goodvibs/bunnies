//! Configuration for PGN output formatting.

/// Controls which annotations and metadata are included when rendering PGN.
#[derive(Debug, Clone, Copy, Eq)]
#[derive_const(PartialEq)]
pub struct PgnRenderingConfig {
    /// Include textual annotations like `[!]` or `[?]`.
    pub include_annotations: bool,
    /// Include Numeric Annotation Glyphs (NAGs, e.g., `$1`, `$2`).
    pub include_nags: bool,
    /// Include `{comments}` in output.
    pub include_comments: bool,
}

impl Default for PgnRenderingConfig {
    fn default() -> Self {
        Self::all_markings()
    }
}

impl PgnRenderingConfig {
    /// Configuration that includes all markings (annotations, NAGs, comments).
    pub const fn all_markings() -> PgnRenderingConfig {
        PgnRenderingConfig {
            include_annotations: true,
            include_nags: true,
            include_comments: true,
        }
    }

    /// Configuration for clean output without any annotations.
    pub const fn no_markings() -> PgnRenderingConfig {
        PgnRenderingConfig {
            include_annotations: false,
            include_nags: false,
            include_comments: false,
        }
    }

    /// Builder-style setter for annotations.
    pub fn annotations(&mut self, include: bool) -> &mut Self {
        self.include_annotations = include;
        self
    }

    /// Builder-style setter for NAGs.
    pub fn nags(&mut self, include: bool) -> &mut Self {
        self.include_nags = include;
        self
    }

    /// Builder-style setter for comments.
    pub fn comments(&mut self, include: bool) -> &mut Self {
        self.include_comments = include;
        self
    }
}
