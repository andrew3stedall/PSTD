use crate::error::PstdResult;
use crate::pst::bbt::BbtIndex;
use crate::pst::bth::{BthHeader, BthMap, BthPropertyEntry, PropertyStorageStatus};
use crate::pst::property_node_resolver::PropertyNodeResolver;
use crate::pst::heap::{
    heap_candidate_offsets_with_limit, heap_signature_offsets_with_limit, HeapOnNode,
    PQ12_MAX_HEAP_SCAN_OFFSET,
};
use crate::pst::limits::ParserLimits;
use crate::pst::nbt::NbtEntry;
use crate::pst::payload::{load_payload_block, PayloadBlock};
use crate::pst::property_context::{PropertyContext, PropertyContextParseReport};
use crate::pst::reader::PstByteReader;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NodePayloadReport {
    pub node_id: u64,
    pub data_block_id: u64,
    pub payload_size_bytes: u64,
    pub property_count: usize,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct LoadedNodePayload {
    pub payload: PayloadBlock,
    pub properties: PropertyContext,
    pub property_report: PropertyContextParseReport,
    pub report: NodePayloadReport,
}

pub fn load_node_property_context(
    reader: &PstByteReader,
    bbt: &BbtIndex,
    entry: &NbtEntry,
    limits: ParserLimits,
) -> PstdResult<LoadedNodePayload> {
    load_node_property_context_with_fallback_charset(reader, bbt, entry, limits, None)
}

pub fn load_node_property_context_with_fallback_charset(
    reader: &PstByteReader,
    bbt: &BbtIndex,
    entry: &NbtEntry,
    limits: ParserLimits,
    fallback_charset: Option<&str>,
) -> PstdResult<LoadedNodePayload> {
    let payload = load_payload_block(reader, bbt, entry.data_block_id, limits)?;
    let payload_base_offset = payload.block_ref.offset.0;
    let (property_report, traversal_status) =
        match load_heap_bth_from_candidates(&payload.bytes, payload_base_offset) {
            Ok(mut parsed) => {
                resolve_node_values(&mut parsed.entries, reader, bbt, entry, limits);
                (
                PropertyContext::from_property_entries(
                    parsed.header,
                    &parsed.entries,
                    fallback_charset,
                )?,
                parsed.traversal_status,
            )},
            Err(reason) => (
                PropertyContext::from_bth_with_fallback_charset(
                    &BthMap::parse(&payload.bytes, payload_base_offset)?,
                    fallback_charset,
                )?,
                format!("legacy_flat_bth_property_context; pq11_heap_probe={reason}"),
            ),
        };
    let properties = property_report
        .context
        .clone()
        .with_pq10_traversal_status(&traversal_status);
    let report = NodePayloadReport {
        node_id: entry.node_id.0,
        data_block_id: entry.data_block_id.0,
        payload_size_bytes: payload.bytes.len() as u64,
        property_count: property_report.parsed_property_count,
        status: format!("node_property_context_loaded; traversal={traversal_status}"),
    };

    Ok(LoadedNodePayload {
        payload,
        properties,
        property_report,
        report,
    })
}

fn resolve_node_values(
    entries: &mut [BthPropertyEntry],
    reader: &PstByteReader,
    bbt: &BbtIndex,
    owner: &NbtEntry,
    limits: ParserLimits,
) {
    let needs_subnodes = entries.iter().any(|entry| {
        entry.source.as_ref().is_some_and(|source| source.status == PropertyStorageStatus::NodeUnresolved)
    });
    let resolver = needs_subnodes.then(|| PropertyNodeResolver::for_owner(reader, bbt, owner, limits));
    let mut resolved_bytes = 0u64;
    for entry in entries {
        let Some(source) = entry.source.as_mut() else { continue };
        source.owner_node_id = Some(owner.node_id.0);
        source.source_block_ids = vec![owner.data_block_id.0];
        if source.status != PropertyStorageStatus::NodeUnresolved {
            continue;
        }
        let result = match resolver.as_ref().expect("NID entry requires resolver") {
            Ok(resolver) => resolver.resolve(source.value_hnid),
            Err(reason) => Err(*reason),
        };
        match result {
            Ok(value) => {
                resolved_bytes = resolved_bytes.saturating_add(value.bytes.len() as u64);
                if resolved_bytes > limits.max_block_bytes {
                    source.resolution_detail = Some("ResourceLimit".into());
                    continue;
                }
                entry.entry.value = value.bytes;
                source.status = if value.data_tree {
                    PropertyStorageStatus::DataTree
                } else {
                    PropertyStorageStatus::Subnode
                };
                source.source_block_ids.extend(value.source_block_ids);
            }
            Err(reason) => source.resolution_detail = Some(format!("{reason:?}")),
        }
    }
}

struct HeapPropertyContext {
    header: BthHeader,
    entries: Vec<BthPropertyEntry>,
    traversal_status: String,
}

fn load_heap_bth_from_candidates(
    buf: &[u8],
    base_offset: u64,
) -> Result<HeapPropertyContext, String> {
    let candidates = heap_candidate_offsets_with_limit(buf, PQ12_MAX_HEAP_SCAN_OFFSET);
    if candidates.is_empty() {
        return Err(candidate_not_found_reason(buf));
    }

    let mut last_error = "candidate_not_parsed".to_string();
    for candidate_offset in candidates.iter().copied() {
        let candidate_buf = &buf[candidate_offset..];
        let candidate_base_offset = base_offset + candidate_offset as u64;
        match HeapOnNode::parse(candidate_buf, candidate_base_offset) {
            Ok(heap) => match BthMap::parse_property_context_with_sources(
                &heap,
                candidate_buf,
                candidate_base_offset,
            ) {
                Ok((header, entries)) => {
                    let status = if candidate_offset == 0 {
                        "heap_bth_property_context".to_string()
                    } else {
                        format!("heap_bth_property_context_at_offset_{candidate_offset}")
                    };
                    return Ok(HeapPropertyContext {
                        header,
                        entries,
                        traversal_status: status,
                    });
                }
                Err(reason) => {
                    last_error = format!(
                        "candidate_bth_failed_at_offset_{candidate_offset}:{}; pq12_boundary=candidate_bth_failed; pq12_payload_bucket={}",
                        sanitized_reason(&reason.to_string()),
                        payload_size_bucket(buf.len())
                    );
                }
            },
            Err(reason) => {
                last_error = format!(
                    "candidate_heap_failed_at_offset_{candidate_offset}:{}; pq12_boundary=candidate_heap_failed; pq12_payload_bucket={}",
                    sanitized_reason(&reason.to_string()),
                    payload_size_bucket(buf.len())
                );
            }
        }
    }

    Err(last_error)
}

fn candidate_not_found_reason(buf: &[u8]) -> String {
    let signatures = heap_signature_offsets_with_limit(buf, PQ12_MAX_HEAP_SCAN_OFFSET);
    if let Some(first_offset) = signatures.first() {
        format!(
            "candidate_not_found; pq12_boundary=signature_without_valid_page_map; pq12_first_signature_offset_{first_offset}; pq12_signature_count={}; pq12_payload_bucket={}",
            signatures.len(),
            payload_size_bucket(buf.len())
        )
    } else {
        format!(
            "candidate_not_found; pq12_boundary=no_signature_in_first_4096; pq12_payload_bucket={}",
            payload_size_bucket(buf.len())
        )
    }
}

fn payload_size_bucket(len: usize) -> &'static str {
    match len {
        0..=127 => "lt_128",
        128..=511 => "lt_512",
        512..=4095 => "lt_4096",
        4096..=65535 => "lt_65536",
        _ => "gte_65536",
    }
}

fn sanitized_reason(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' => ch,
            _ => '_',
        })
        .collect::<String>()
        .trim_matches('_')
        .chars()
        .take(80)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::NamedTempFile;

    use super::load_node_property_context;
    use crate::pst::bbt::{BbtEntry, BbtIndex};
    use crate::pst::limits::ParserLimits;
    use crate::pst::mapi::PR_SUBJECT;
    use crate::pst::nbt::NbtEntry;
    use crate::pst::primitives::{BlockId, ByteOffset, NodeId};
    use crate::pst::reader::PstByteReader;

    #[test]
    fn loads_node_property_context_from_legacy_flat_data_block() {
        let mut bytes = bth_with_subject("Hello from node");
        let offset = 16usize;
        let mut file_bytes = vec![0; offset];
        file_bytes.append(&mut bytes);

        let file = NamedTempFile::new().unwrap();
        fs::write(file.path(), file_bytes).unwrap();
        let reader = PstByteReader::open(file.path()).unwrap();
        let bbt = index_with_entry(BlockId(100), offset as u64, 44);
        let entry = NbtEntry {
            node_id: NodeId(200),
            data_block_id: BlockId(100),
            subnode_block_id: None,
        };

        let loaded =
            load_node_property_context(&reader, &bbt, &entry, ParserLimits::default()).unwrap();
        assert_eq!(loaded.report.node_id, 200);
        assert_eq!(loaded.report.data_block_id, 100);
        assert_eq!(loaded.report.property_count, 1);
        assert_eq!(
            loaded.properties.string_value(PR_SUBJECT).as_deref(),
            Some("Hello from node")
        );
        assert_eq!(
            loaded.properties.pq10_status(),
            "pq10_traversal=legacy_flat_bth_property_context; pq11_heap_probe=candidate_not_found; pq12_boundary=no_signature_in_first_4096; pq12_payload_bucket=lt_128"
        );
        assert_eq!(
            loaded.report.status,
            "node_property_context_loaded; traversal=legacy_flat_bth_property_context; pq11_heap_probe=candidate_not_found; pq12_boundary=no_signature_in_first_4096; pq12_payload_bucket=lt_128"
        );
    }

    #[test]
    fn loads_node_property_context_from_heap_bth_data_block() {
        let mut bytes = heap_bth_with_subject("Hello from heap");
        let offset = 32usize;
        let size = bytes.len() as u64;
        let mut file_bytes = vec![0; offset];
        file_bytes.append(&mut bytes);

        let file = NamedTempFile::new().unwrap();
        fs::write(file.path(), file_bytes).unwrap();
        let reader = PstByteReader::open(file.path()).unwrap();
        let bbt = index_with_entry(BlockId(100), offset as u64, size);
        let entry = NbtEntry {
            node_id: NodeId(200),
            data_block_id: BlockId(100),
            subnode_block_id: None,
        };

        let loaded =
            load_node_property_context(&reader, &bbt, &entry, ParserLimits::default()).unwrap();
        assert_eq!(loaded.report.property_count, 1);
        assert_eq!(
            loaded.properties.string_value(PR_SUBJECT).as_deref(),
            Some("Hello from heap")
        );
        assert_eq!(
            loaded.properties.pq10_status(),
            "pq10_traversal=heap_bth_property_context"
        );
        assert_eq!(
            loaded.report.status,
            "node_property_context_loaded; traversal=heap_bth_property_context"
        );
    }

    #[test]
    fn loads_node_property_context_from_offset_heap_bth_data_block() {
        let mut bytes = vec![0; 16];
        bytes.extend_from_slice(&heap_bth_with_subject("Hello from offset heap"));
        let offset = 32usize;
        let size = bytes.len() as u64;
        let mut file_bytes = vec![0; offset];
        file_bytes.append(&mut bytes);

        let file = NamedTempFile::new().unwrap();
        fs::write(file.path(), file_bytes).unwrap();
        let reader = PstByteReader::open(file.path()).unwrap();
        let bbt = index_with_entry(BlockId(100), offset as u64, size);
        let entry = NbtEntry {
            node_id: NodeId(200),
            data_block_id: BlockId(100),
            subnode_block_id: None,
        };

        let loaded =
            load_node_property_context(&reader, &bbt, &entry, ParserLimits::default()).unwrap();
        assert_eq!(
            loaded.properties.string_value(PR_SUBJECT).as_deref(),
            Some("Hello from offset heap")
        );
        assert_eq!(
            loaded.properties.pq10_status(),
            "pq10_traversal=heap_bth_property_context_at_offset_16"
        );
    }

    #[test]
    fn leaves_missing_subnode_subject_unresolved_in_node_extraction() {
        let mut bytes = heap_bth_with_subject("must not be read");
        bytes[28..32].copy_from_slice(&0x64u32.to_le_bytes());
        let file = NamedTempFile::new().unwrap();
        fs::write(file.path(), &bytes).unwrap();
        let reader = PstByteReader::open(file.path()).unwrap();
        let bbt = index_with_entry(BlockId(100), 0, bytes.len() as u64);
        let entry = NbtEntry {
            node_id: NodeId(200),
            data_block_id: BlockId(100),
            subnode_block_id: None,
        };
        let loaded =
            load_node_property_context(&reader, &bbt, &entry, ParserLimits::default()).unwrap();
        assert!(loaded.properties.string_value(PR_SUBJECT).is_none());
        assert_eq!(loaded.property_report.unresolved_reference_count, 1);
        assert_eq!(loaded.property_report.decode_error_count, 0);
        let value = loaded.properties.value(PR_SUBJECT).unwrap();
        assert_eq!(value.raw, 0x64u32.to_le_bytes());
        assert!(value.status.starts_with("HNID_UNRESOLVED"));
    }

    #[test]
    fn resolves_subnode_subject_in_node_extraction() {
        let mut heap = heap_bth_with_subject("unused heap data");
        heap[28..32].copy_from_slice(&0x64u32.to_le_bytes());
        let subject = utf16le("Subnode subject");
        let mut subnodes = vec![2, 0, 1, 0, 0, 0, 0, 0];
        subnodes.extend_from_slice(&0x64u64.to_le_bytes());
        subnodes.extend_from_slice(&8u64.to_le_bytes());
        subnodes.extend_from_slice(&0u64.to_le_bytes());
        let mut bytes = vec![0; 512];
        let mut bbt = index_with_entry(BlockId(100), 512, heap.len() as u64);
        bytes.extend_from_slice(&heap);
        for (bid, payload) in [(2, subnodes), (8, subject)] {
            bbt.entries.push(BbtEntry { block_id: BlockId(bid), offset: ByteOffset(bytes.len() as u64), size: payload.len() as u64 });
            bytes.extend_from_slice(&payload);
        }
        let file = NamedTempFile::new().unwrap();
        fs::write(file.path(), bytes).unwrap();
        let reader = PstByteReader::open(file.path()).unwrap();
        let owner = NbtEntry {
            node_id: NodeId(200), data_block_id: BlockId(100), subnode_block_id: Some(BlockId(2)),
        };
        let loaded = load_node_property_context(&reader, &bbt, &owner, ParserLimits::default()).unwrap();
        assert_eq!(loaded.properties.string_value(PR_SUBJECT).as_deref(), Some("Subnode subject"));
        assert_eq!(loaded.property_report.unresolved_reference_count, 0);
        let source = &loaded.property_report.property_sources[0];
        assert_eq!(source.value_hnid, 0x64);
        assert_eq!(source.owner_node_id, Some(200));
        assert!(source.source_block_ids.contains(&8));
        assert_eq!(source.status, crate::pst::bth::PropertyStorageStatus::Subnode);
    }

    fn bth_with_subject(value: &str) -> Vec<u8> {
        let mut body = Vec::new();
        let mut value_bytes = utf16le_fixed(value, 32);
        body.push(4);
        body.push(32);
        body.extend_from_slice(&1u16.to_le_bytes());
        body.extend_from_slice(&0u32.to_le_bytes());
        body.extend_from_slice(&PR_SUBJECT.to_le_bytes());
        body.append(&mut value_bytes);
        body
    }

    fn heap_bth_with_subject(value: &str) -> Vec<u8> {
        let subject = utf16le(value);
        let subject_end = 32u16 + subject.len() as u16;
        let page_map_offset = 144u16;
        let mut body = vec![0; page_map_offset as usize + 16];
        body[0..2].copy_from_slice(&page_map_offset.to_le_bytes());
        body[2] = 0xec;
        body[3] = 0xbc;
        body[4..8].copy_from_slice(&0x20u32.to_le_bytes());

        body[16] = 0xb5;
        body[17] = 2;
        body[18] = 6;
        body[19] = 0;
        body[20..24].copy_from_slice(&0x40u32.to_le_bytes());

        body[24..26].copy_from_slice(&0x0037u16.to_le_bytes());
        body[26..28].copy_from_slice(&0x001fu16.to_le_bytes());
        body[28..32].copy_from_slice(&0x60u32.to_le_bytes());

        body[32..subject_end as usize].copy_from_slice(&subject);

        body[144..146].copy_from_slice(&3u16.to_le_bytes());
        body[146..148].copy_from_slice(&0u16.to_le_bytes());
        body[148..150].copy_from_slice(&16u16.to_le_bytes());
        body[150..152].copy_from_slice(&24u16.to_le_bytes());
        body[152..154].copy_from_slice(&32u16.to_le_bytes());
        body[154..156].copy_from_slice(&subject_end.to_le_bytes());
        body
    }

    fn utf16le(value: &str) -> Vec<u8> {
        let mut bytes = Vec::new();
        for unit in value.encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes
    }

    fn utf16le_fixed(value: &str, len: usize) -> Vec<u8> {
        let mut bytes = utf16le(value);
        bytes.resize(len, 0);
        bytes
    }

    fn index_with_entry(block_id: BlockId, offset: u64, size: u64) -> BbtIndex {
        BbtIndex {
            root: None,
            entries: vec![BbtEntry {
                block_id,
                offset: ByteOffset(offset),
                size,
            }],
            parsed_pages: 0,
            discovered_child_pages: 0,
            traversal_error_count: 0,
            duplicate_entry_count: 0,
            truncated_entry_count: 0,
            status: "test".to_string(),
        }
    }
}
