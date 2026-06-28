use serde::Serialize;

use crate::error::PstdResult;

#[derive(Debug, Default)]
pub struct JsonlBuffer {
    bytes: Vec<u8>,
    rows: usize,
}

impl JsonlBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write_record<T: Serialize>(&mut self, record: &T) -> PstdResult<()> {
        serde_json::to_writer(&mut self.bytes, record)?;
        self.bytes.push(b'\n');
        self.rows += 1;
        Ok(())
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn is_empty(&self) -> bool {
        self.rows == 0
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}
