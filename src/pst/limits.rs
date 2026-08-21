#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InputLimits {
    pub max_file_bytes: u64,
    pub max_single_read_bytes: usize,
    pub max_diagnostics: usize,
    pub max_candidate_items: usize,
    pub max_property_bytes: u64,
    pub max_recursion_depth: usize,
}

impl Default for InputLimits {
    fn default() -> Self {
        Self {
            max_file_bytes: 16 * 1024 * 1024 * 1024,
            max_single_read_bytes: 64 * 1024 * 1024,
            max_diagnostics: 4096,
            max_candidate_items: 100_000,
            max_property_bytes: 64 * 1024 * 1024,
            max_recursion_depth: 16,
        }
    }
}

impl InputLimits {
    pub fn conservative() -> Self {
        Self {
            max_file_bytes: 2 * 1024 * 1024 * 1024,
            max_single_read_bytes: 8 * 1024 * 1024,
            max_diagnostics: 1024,
            max_candidate_items: 10_000,
            max_property_bytes: 8 * 1024 * 1024,
            max_recursion_depth: 8,
        }
    }
}

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
    use super::{InputLimits, ParserLimits};

    #[test]
    fn exposes_default_input_limits() {
        let limits = InputLimits::default();
        assert_eq!(limits.max_single_read_bytes, 64 * 1024 * 1024);
        assert_eq!(limits.max_diagnostics, 4096);
        assert_eq!(limits.max_recursion_depth, 16);
    }

    #[test]
    fn exposes_conservative_input_limits() {
        let limits = InputLimits::conservative();
        assert_eq!(limits.max_file_bytes, 2 * 1024 * 1024 * 1024);
        assert_eq!(limits.max_single_read_bytes, 8 * 1024 * 1024);
        assert_eq!(limits.max_recursion_depth, 8);
    }

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
