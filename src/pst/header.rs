#[derive(Debug, Clone)]
pub struct PstHeaderSummary {
    pub format: String,
    pub parser_status: String,
}

impl PstHeaderSummary {
    pub fn placeholder() -> Self {
        Self {
            format: "unknown".to_string(),
            parser_status: "deferred_to_later_milestone".to_string(),
        }
    }
}
