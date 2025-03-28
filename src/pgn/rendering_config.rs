/// Contains a configuration for rendering PGN (Portable Game Notation) data.
#[derive(Debug, Clone, Copy)]
pub struct PgnRenderingConfig {
    pub include_annotations: bool,
    pub include_nags: bool,
    pub include_comments: bool,
}

impl Default for PgnRenderingConfig {
    fn default() -> Self {
        PgnRenderingConfig {
            include_annotations: true,
            include_nags: true,
            include_comments: true,
        }
    }
}

impl PgnRenderingConfig {
    /// Creates a new `PgnRenderingConfig` specifying no annotations, NAGs, or comments.
    pub fn no_markings() -> PgnRenderingConfig {
        PgnRenderingConfig {
            include_annotations: false,
            include_nags: false,
            include_comments: false,
        }
    }

    /// Sets whether to include annotations.
    pub fn annotations(&mut self, include: bool) -> &mut Self {
        self.include_annotations = include;
        self
    }

    /// Sets whether to include NAGs (Numeric Annotation Glyphs).
    pub fn nags(&mut self, include: bool) -> &mut Self {
        self.include_nags = include;
        self
    }

    /// Sets whether to include comments.
    pub fn comments(&mut self, include: bool) -> &mut Self {
        self.include_comments = include;
        self
    }
}
