#[derive(Debug, Clone)]
pub struct PlaceholderRecordSet {
    pub source_name: String,
}

impl PlaceholderRecordSet {
    pub fn new(source_name: impl Into<String>) -> Self {
        Self {
            source_name: source_name.into(),
        }
    }
}
