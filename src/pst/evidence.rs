use sha2::{Digest, Sha256};

use crate::output::ids;
use crate::output::metadata::EvidenceRecord;
use crate::pst::primitives::{BlockId, NodeId};
use crate::pst::property_context::PropertyContext;
use crate::pst::subnodes::SubnodeReference;

const MAX_RAW_EVIDENCE_BYTES: usize = 64 * 1024;

pub fn property_records(
    owner_key: &str,
    node_id: NodeId,
    data_block_id: BlockId,
    properties: &PropertyContext,
) -> Vec<EvidenceRecord> {
    let mut values = properties.values.values().collect::<Vec<_>>();
    values.sort_by_key(|value| value.tag);
    values
        .into_iter()
        .map(|value| {
            let source_ref = format!(
                "node_{:x}:data_block_0x{:x}:property_0x{:08x}",
                node_id.0, data_block_id.0, value.tag
            );
            let evidence_key = ids::stable_id(
                "evidence",
                &[owner_key, "property", &format!("{:08x}", value.tag)],
            );
            let (raw_size_bytes, raw_sha256, raw_bytes_hex) = raw_fields(&value.raw);
            EvidenceRecord {
                evidence_key,
                owner_key: owner_key.to_string(),
                evidence_kind: "property".to_string(),
                source_ref,
                property_tag: Some(value.tag),
                raw_size_bytes,
                raw_sha256,
                raw_bytes_hex,
                status: value.status.clone(),
            }
        })
        .collect()
}

pub fn property_failure_record(
    owner_key: &str,
    node_id: NodeId,
    data_block_id: BlockId,
    status: &str,
) -> EvidenceRecord {
    evidence_record(
        owner_key,
        "property_context",
        format!("node_{:x}:data_block_0x{:x}", node_id.0, data_block_id.0),
        None,
        None,
        status,
    )
}

pub fn subnode_record(reference: &SubnodeReference) -> EvidenceRecord {
    evidence_record(
        &format!("node_{:x}", reference.node_id.0),
        "subnode_reference",
        format!(
            "node_{:x}:subnode_block_0x{:x}",
            reference.node_id.0, reference.subnode_block_id.0
        ),
        None,
        None,
        &reference.status,
    )
}

pub fn payload_record(
    owner_key: &str,
    evidence_kind: &str,
    source_ref: String,
    archive_path: &str,
    bytes: Option<&[u8]>,
    status: &str,
) -> EvidenceRecord {
    let mut record = evidence_record(owner_key, evidence_kind, source_ref, None, bytes, status);
    record.source_ref = format!("{}; archive_path={archive_path}", record.source_ref);
    record
}

fn evidence_record(
    owner_key: &str,
    evidence_kind: &str,
    source_ref: String,
    property_tag: Option<u32>,
    bytes: Option<&[u8]>,
    status: &str,
) -> EvidenceRecord {
    let (raw_size_bytes, raw_sha256, raw_bytes_hex) =
        bytes.map(raw_fields).unwrap_or((0, None, None));
    let evidence_key = ids::stable_id("evidence", &[owner_key, evidence_kind, &source_ref]);
    EvidenceRecord {
        evidence_key,
        owner_key: owner_key.to_string(),
        evidence_kind: evidence_kind.to_string(),
        source_ref,
        property_tag,
        raw_size_bytes,
        raw_sha256,
        raw_bytes_hex,
        status: status.to_string(),
    }
}

fn raw_fields(bytes: &[u8]) -> (u64, Option<String>, Option<String>) {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let raw_sha256 = hex::encode(hasher.finalize());
    let raw_bytes_hex = if bytes.len() <= MAX_RAW_EVIDENCE_BYTES {
        Some(hex::encode(bytes))
    } else {
        Some(hex::encode(&bytes[..MAX_RAW_EVIDENCE_BYTES]))
    };
    (bytes.len() as u64, Some(raw_sha256), raw_bytes_hex)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::property_records;
    use crate::pst::mapi::PR_SUBJECT;
    use crate::pst::primitives::{BlockId, NodeId};
    use crate::pst::property_context::{PropertyContext, PropertyValue};

    #[test]
    fn preserves_raw_property_bytes_and_orders_evidence_by_tag() {
        let mut values = HashMap::new();
        values.insert(
            PR_SUBJECT,
            PropertyValue {
                tag: PR_SUBJECT,
                name: "subject".to_string(),
                raw: vec![0x41, 0x00],
                decoded: None,
                status: "decode_failed".to_string(),
            },
        );
        let records = property_records(
            "msg-test",
            NodeId(0x24),
            BlockId(0x80),
            &PropertyContext::from_values(values),
        );
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].raw_size_bytes, 2);
        assert_eq!(records[0].raw_bytes_hex.as_deref(), Some("4100"));
        assert!(records[0].raw_sha256.is_some());
        assert!(records[0].source_ref.contains("data_block_0x80"));
    }
}
