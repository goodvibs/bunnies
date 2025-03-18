
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
    pub fn no_markings() -> PgnRenderingConfig {
        PgnRenderingConfig {
            include_annotations: false,
            include_nags: false,
            include_comments: false,
        }
    }

    pub fn annotations(&mut self, include: bool) -> &mut Self {
        self.include_annotations = include;
        self
    }

    pub fn nags(&mut self, include: bool) -> &mut Self {
        self.include_nags = include;
        self
    }

    pub fn comments(&mut self, include: bool) -> &mut Self {
        self.include_comments = include;
        self
    }
}