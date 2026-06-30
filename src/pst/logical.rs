use crate::error::{PstdError, PstdResult};
use crate::pst::bbt::BbtIndex;
use crate::pst::block::{load_block, LoadedBlock};
use crate::pst::nbt::{NbtEntry, NbtIndex};
use crate::pst::primitives::NodeId;
use crate::pst::reader::PstByteReader;

#[derive(Debug, Clone)]
pub struct LogicalNode {
    pub entry: NbtEntry,
    pub data: Vec<u8>,
    pub status: String,
}

#[derive(Debug)]
pub struct LogicalNodeStore<'a> {
    reader: &'a PstByteReader,
    nbt: &'a NbtIndex,
    bbt: &'a BbtIndex,
}

impl<'a> LogicalNodeStore<'a> {
    pub fn new(reader: &'a PstByteReader, nbt: &'a NbtIndex, bbt: &'a BbtIndex) -> Self {
        Self { reader, nbt, bbt }
    }

    pub fn load_node(&self, node_id: NodeId) -> PstdResult<LogicalNode> {
        let entry = self
            .nbt
            .lookup(node_id)
            .ok_or_else(|| PstdError::pst_parse(None, format!("node {:?} not found", node_id)))?
            .clone();

        let block_ref = self.bbt.lookup(entry.data_block_id).ok_or_else(|| {
            PstdError::pst_parse(
                None,
                format!(
                    "data block {:?} not found for node {:?}",
                    entry.data_block_id, node_id
                ),
            )
        })?;

        let LoadedBlock { bytes, .. } = load_block(self.reader, block_ref)?;
        Ok(LogicalNode {
            entry,
            data: bytes,
            status: "loaded".to_string(),
        })
    }
}
