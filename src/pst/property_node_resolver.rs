//! Owner-scoped property NID resolution. Never searches another node's subnode tree.
use std::collections::{BTreeMap, HashSet};

use crate::pst::bbt::BbtIndex;
use crate::pst::data_tree::load_attachment_data_payload;
use crate::pst::limits::ParserLimits;
use crate::pst::nbt::NbtEntry;
use crate::pst::payload::{load_payload_block, PayloadBlock};
use crate::pst::primitives::BlockId;
use crate::pst::reader::PstByteReader;
use crate::pst::subnodes::{unicode_subnode_entries, UnicodeSubnodeEntry};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ReferenceFailure {
    Missing,
    Malformed,
    Duplicate,
    Cycle,
    ResourceLimit,
    PayloadInvalid,
}

#[derive(Debug, Clone)]
pub struct ResolvedNodeValue {
    pub bytes: Vec<u8>,
    pub owner_node_id: u64,
    pub node_id: u32,
    pub data_block_id: u64,
    pub source_block_ids: Vec<u64>,
    pub data_tree: bool,
}

/// A validated map for exactly one owner's immediate subnodes. Nested owners'
/// subnode BIDs are deliberately not traversed. Build once per Property Context.
pub struct PropertyNodeResolver<'a> {
    reader: &'a PstByteReader,
    bbt: &'a BbtIndex,
    limits: ParserLimits,
    owner_node_id: u64,
    entries: BTreeMap<u32, BlockId>,
    source_block_ids: Vec<u64>,
    index_bytes: u64,
}

impl<'a> PropertyNodeResolver<'a> {
    pub fn for_owner(
        reader: &'a PstByteReader,
        bbt: &'a BbtIndex,
        owner: &NbtEntry,
        limits: ParserLimits,
    ) -> Result<Self, ReferenceFailure> {
        let root = owner.subnode_block_id.ok_or(ReferenceFailure::Missing)?;
        let mut resolver = Self {
            reader,
            bbt,
            limits,
            owner_node_id: owner.node_id.0,
            entries: BTreeMap::new(),
            source_block_ids: Vec::new(),
            index_bytes: 0,
        };
        let mut seen = HashSet::new();
        let root_payload = resolver.load_index(root, &mut seen)?;
        let bytes = &root_payload.bytes;
        if bytes.len() < 8 || bytes[0] != 2 || bytes[4..8] != [0; 4] {
            return Err(ReferenceFailure::Malformed);
        }
        match bytes[1] {
            0 => resolver.add_leaf(&root_payload, None)?,
            1 => {
                // MS-PST 2.2.2.8.3.3: Unicode SIBLOCK has 16-byte SIENTRYs;
                // cLevel=1 points to SLBLOCKs, never another SIBLOCK.
                if limits.max_subnode_depth < 1 {
                    return Err(ReferenceFailure::ResourceLimit);
                }
                let count = u16::from_le_bytes([bytes[2], bytes[3]]) as usize;
                if count == 0 || 8 + count * 16 > bytes.len() {
                    return Err(ReferenceFailure::Malformed);
                }
                let mut previous_key = None;
                for item in bytes[8..8 + count * 16].as_chunks::<16>().0 {
                    let key = u64::from_le_bytes(item[..8].try_into().unwrap());
                    let key = u32::try_from(key).map_err(|_| ReferenceFailure::Malformed)?;
                    if previous_key.is_some_and(|previous| key <= previous) {
                        return Err(ReferenceFailure::Duplicate);
                    }
                    previous_key = Some(key);
                    let bid = BlockId(u64::from_le_bytes(item[8..].try_into().unwrap()));
                    let leaf = resolver.load_index(bid, &mut seen)?;
                    resolver.add_leaf(&leaf, Some(key))?;
                }
            }
            _ => return Err(ReferenceFailure::Malformed),
        }
        Ok(resolver)
    }

    fn load_index(
        &mut self,
        bid: BlockId,
        seen: &mut HashSet<BlockId>,
    ) -> Result<PayloadBlock, ReferenceFailure> {
        if bid.0 & 2 == 0 {
            return Err(ReferenceFailure::Malformed);
        }
        if !seen.insert(bid) {
            return Err(ReferenceFailure::Cycle);
        }
        if seen.len() > self.limits.max_btree_pages {
            return Err(ReferenceFailure::ResourceLimit);
        }
        self.check_block(bid)?;
        let block = load_payload_block(self.reader, self.bbt, bid, self.limits)
            .map_err(|_| ReferenceFailure::PayloadInvalid)?;
        self.index_bytes = self
            .index_bytes
            .checked_add(block.bytes.len() as u64)
            .ok_or(ReferenceFailure::ResourceLimit)?;
        if self.index_bytes > self.limits.max_block_bytes {
            return Err(ReferenceFailure::ResourceLimit);
        }
        self.source_block_ids.push(bid.0);
        Ok(block)
    }

    fn check_block(&self, bid: BlockId) -> Result<(), ReferenceFailure> {
        let mut matches = self
            .bbt
            .entries
            .iter()
            .filter(|entry| entry.block_id == bid);
        let block = matches.next().ok_or(ReferenceFailure::Missing)?;
        if matches.next().is_some() {
            return Err(ReferenceFailure::Duplicate);
        }
        if block.size > self.limits.max_block_bytes {
            return Err(ReferenceFailure::ResourceLimit);
        }
        Ok(())
    }

    fn add_leaf(
        &mut self,
        payload: &PayloadBlock,
        expected_first: Option<u32>,
    ) -> Result<(), ReferenceFailure> {
        let entries = unicode_subnode_entries(payload).ok_or(ReferenceFailure::Malformed)?;
        if entries.is_empty() || expected_first.is_some_and(|key| entries[0].node_id != key) {
            return Err(ReferenceFailure::Malformed);
        }
        // SIBLOCK child ranges must also be ordered across leaf boundaries.
        let mut previous = self.entries.last_key_value().map(|(key, _)| *key);
        for UnicodeSubnodeEntry {
            node_id,
            data_block_id,
            ..
        } in entries
        {
            if previous.is_some_and(|key| node_id <= key) || self.entries.contains_key(&node_id) {
                return Err(ReferenceFailure::Duplicate);
            }
            previous = Some(node_id);
            self.entries.insert(node_id, data_block_id);
        }
        Ok(())
    }

    pub fn resolve(&self, node_id: u32) -> Result<ResolvedNodeValue, ReferenceFailure> {
        if node_id == 0 || node_id & 0x1f == 0 {
            return Err(ReferenceFailure::Malformed);
        }
        let bid = *self
            .entries
            .get(&node_id)
            .ok_or(ReferenceFailure::Missing)?;
        self.check_block(bid)?;
        let tree = load_attachment_data_payload(self.reader, self.bbt, bid, None, self.limits)
            .map_err(|_| ReferenceFailure::PayloadInvalid)?;
        // The data loader is bounded and rejects repeated child BIDs. Also ensure
        // no child BID has ambiguous BBT ownership before accepting its bytes.
        for child in &tree.child_bids {
            self.check_block(*child)?;
        }
        let mut source_block_ids = self.source_block_ids.clone();
        source_block_ids.push(bid.0);
        source_block_ids.extend(tree.child_bids.iter().map(|child| child.0));
        source_block_ids.sort_unstable();
        source_block_ids.dedup();
        Ok(ResolvedNodeValue {
            bytes: tree.bytes,
            owner_node_id: self.owner_node_id,
            node_id,
            data_block_id: bid.0,
            source_block_ids,
            data_tree: bid.0 & 2 != 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pst::bbt::BbtEntry;
    use crate::pst::primitives::{ByteOffset, NodeId};
    use tempfile::NamedTempFile;

    fn leaf(nid: u32, bid: u64) -> Vec<u8> {
        let mut bytes = vec![2, 0, 1, 0, 0, 0, 0, 0];
        bytes.extend_from_slice(&(nid as u64).to_le_bytes());
        bytes.extend_from_slice(&bid.to_le_bytes());
        bytes.extend_from_slice(&0u64.to_le_bytes());
        bytes
    }

    fn fixture(blocks: Vec<(u64, Vec<u8>)>) -> (NamedTempFile, BbtIndex, NbtEntry) {
        let file = NamedTempFile::new().unwrap();
        let mut bytes = vec![0; 1024];
        let mut entries = Vec::new();
        for (bid, payload) in blocks {
            entries.push(BbtEntry {
                block_id: BlockId(bid),
                offset: ByteOffset(bytes.len() as u64),
                size: payload.len() as u64,
            });
            bytes.extend_from_slice(&payload);
        }
        std::fs::write(file.path(), bytes).unwrap();
        (
            file,
            BbtIndex {
                root: None,
                entries,
                parsed_pages: 0,
                discovered_child_pages: 0,
                traversal_error_count: 0,
                duplicate_entry_count: 0,
                truncated_entry_count: 0,
                status: "test".into(),
            },
            NbtEntry {
                node_id: NodeId(0x204),
                data_block_id: BlockId(100),
                subnode_block_id: Some(BlockId(2)),
            },
        )
    }

    #[test]
    fn resolves_owner_scoped_nid_to_exact_bytes() {
        let (file, bbt, owner) = fixture(vec![(2, leaf(0x64, 8)), (8, b"body".to_vec())]);
        let reader = PstByteReader::open(file.path()).unwrap();
        let resolver =
            PropertyNodeResolver::for_owner(&reader, &bbt, &owner, ParserLimits::default())
                .unwrap();
        let value = resolver.resolve(0x64).unwrap();
        assert_eq!(value.bytes, b"body");
        assert_eq!(value.owner_node_id, 0x204);
        assert_eq!(value.node_id, 0x64);
        assert_eq!(value.source_block_ids, vec![2, 8]);
        assert!(!value.data_tree);
        assert_eq!(
            resolver.resolve(0x84).unwrap_err(),
            ReferenceFailure::Missing
        );
    }

    #[test]
    fn resolves_data_tree_without_interpreting_its_content() {
        let mut tree = vec![1, 1, 2, 0];
        tree.extend_from_slice(&4u32.to_le_bytes());
        tree.extend_from_slice(&8u64.to_le_bytes());
        tree.extend_from_slice(&12u64.to_le_bytes());
        let (file, bbt, owner) = fixture(vec![
            (2, leaf(0x64, 6)),
            (6, tree),
            (8, b"ab".to_vec()),
            (12, b"cd".to_vec()),
        ]);
        let reader = PstByteReader::open(file.path()).unwrap();
        let resolver =
            PropertyNodeResolver::for_owner(&reader, &bbt, &owner, ParserLimits::default())
                .unwrap();
        let value = resolver.resolve(0x64).unwrap();
        assert_eq!(value.bytes, b"abcd");
        assert!(value.data_tree);
        assert_eq!(value.data_block_id, 6);
    }

    #[test]
    fn rejects_duplicate_nids_and_bbt_ownership() {
        let mut duplicate = leaf(0x64, 8);
        duplicate[2..4].copy_from_slice(&2u16.to_le_bytes());
        duplicate.extend_from_slice(&leaf(0x64, 12)[8..]);
        let (file, bbt, owner) = fixture(vec![(2, duplicate), (8, vec![1]), (12, vec![2])]);
        let reader = PstByteReader::open(file.path()).unwrap();
        assert!(matches!(
            PropertyNodeResolver::for_owner(&reader, &bbt, &owner, ParserLimits::default()),
            Err(ReferenceFailure::Duplicate)
        ));
        let (file, bbt, owner) = fixture(vec![(2, leaf(0x64, 8)), (8, vec![1]), (8, vec![2])]);
        let reader = PstByteReader::open(file.path()).unwrap();
        let resolver =
            PropertyNodeResolver::for_owner(&reader, &bbt, &owner, ParserLimits::default())
                .unwrap();
        assert_eq!(
            resolver.resolve(0x64).unwrap_err(),
            ReferenceFailure::Duplicate
        );
    }

    #[test]
    fn rejects_overlapping_index_leaf_ranges_and_external_index_blocks() {
        let mut index = vec![2, 1, 2, 0, 0, 0, 0, 0];
        for (key, bid) in [(0x64u64, 6u64), (0x84, 10)] {
            index.extend_from_slice(&key.to_le_bytes());
            index.extend_from_slice(&bid.to_le_bytes());
        }
        let mut first = leaf(0x64, 8);
        first[2..4].copy_from_slice(&2u16.to_le_bytes());
        first.extend_from_slice(&leaf(0xa4, 12)[8..]);
        let (file, bbt, owner) = fixture(vec![(2, index), (6, first), (10, leaf(0x84, 16))]);
        let reader = PstByteReader::open(file.path()).unwrap();
        assert!(matches!(
            PropertyNodeResolver::for_owner(&reader, &bbt, &owner, ParserLimits::default()),
            Err(ReferenceFailure::Duplicate)
        ));
        let (file, bbt, mut owner) = fixture(vec![(8, leaf(0x64, 12))]);
        owner.subnode_block_id = Some(BlockId(8));
        let reader = PstByteReader::open(file.path()).unwrap();
        assert!(matches!(
            PropertyNodeResolver::for_owner(&reader, &bbt, &owner, ParserLimits::default()),
            Err(ReferenceFailure::Malformed)
        ));
    }

    #[test]
    fn rejects_cycles_and_resource_limits() {
        let mut index = vec![2, 1, 1, 0, 0, 0, 0, 0];
        index.extend_from_slice(&0x64u64.to_le_bytes());
        index.extend_from_slice(&2u64.to_le_bytes());
        let (file, bbt, owner) = fixture(vec![(2, index)]);
        let reader = PstByteReader::open(file.path()).unwrap();
        assert!(matches!(
            PropertyNodeResolver::for_owner(&reader, &bbt, &owner, ParserLimits::default()),
            Err(ReferenceFailure::Cycle)
        ));
        let limits = ParserLimits {
            max_btree_pages: 0,
            ..ParserLimits::default()
        };
        assert!(matches!(
            PropertyNodeResolver::for_owner(&reader, &bbt, &owner, limits),
            Err(ReferenceFailure::ResourceLimit)
        ));
    }

    #[test]
    fn resolves_indexed_leaf_but_never_descends_into_another_owner() {
        let mut index = vec![2, 1, 1, 0, 0, 0, 0, 0];
        index.extend_from_slice(&0x64u64.to_le_bytes());
        index.extend_from_slice(&6u64.to_le_bytes());
        let mut entry = leaf(0x64, 8);
        entry[24..32].copy_from_slice(&10u64.to_le_bytes());
        let (file, bbt, owner) = fixture(vec![
            (2, index),
            (6, entry),
            (8, b"owned".to_vec()),
            (10, leaf(0x84, 12)),
            (12, b"other owner".to_vec()),
        ]);
        let reader = PstByteReader::open(file.path()).unwrap();
        let resolver =
            PropertyNodeResolver::for_owner(&reader, &bbt, &owner, ParserLimits::default())
                .unwrap();
        assert_eq!(resolver.resolve(0x64).unwrap().bytes, b"owned");
        assert_eq!(
            resolver.resolve(0x84).unwrap_err(),
            ReferenceFailure::Missing
        );
    }
}
