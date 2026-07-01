#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct ParserLimits {
    pub max_btree_pages: usize,
    pub max_block_bytes: u64,
    pub max_subnode_depth: usize,
}

impl Default for ParserLimits {
    fn default() -> Self {
        Self {
            max_btree_pages: 1024,
            max_block_bytes: 64 * 1024 * 1024,
            max_subnode_depth: 16,
        }
    }
}

impl ParserLimits {
    pub fn conservative() -> Self {
        Self {
            max_btree_pages: 128,
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ParserLimits;

    #[test]
    fn exposes_default_parser_limits() {
        let limits = ParserLimits::default();
        assert_eq!(limits.max_btree_pages, 1024);
        assert_eq!(limits.max_block_bytes, 64 * 1024 * 1024);
        assert_eq!(limits.max_subnode_depth, 16);
    }

    #[test]
    fn exposes_conservative_parser_limits() {
        let limits = ParserLimits::conservative();
        assert_eq!(limits.max_btree_pages, 128);
        assert_eq!(limits.max_block_bytes, 64 * 1024 * 1024);
    }
}
