use crate::pst::heap::HeapOnNode;
use crate::pst::tc_recipient_identity_reference::RecipientIdentityReferenceEvidence;
use crate::pst::tcinfo::HnidKind;

const PT_STRING8: u16 = 0x001e;
const PT_UNICODE: u16 = 0x001f;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RecipientIdentityStringEvidence {
    pub property_tag: u32,
    pub property_name: String,
    pub row_values: Vec<String>,
}

pub fn resolve_recipient_identity_heap_strings(
    reference_evidence: &RecipientIdentityReferenceEvidence,
    heap: &HeapOnNode,
    heap_bytes: &[u8],
    base_offset: u64,
) -> Result<RecipientIdentityStringEvidence, String> {
    if reference_evidence.row_references.is_empty()
        || reference_evidence.row_references.len() != reference_evidence.reference_kinds.len()
    {
        return Err("recipient identity reference evidence is incomplete".to_string());
    }

    let property_type = reference_evidence.property_tag as u16;
    if !matches!(property_type, PT_STRING8 | PT_UNICODE) {
        return Err("recipient identity property is not a supported string type".to_string());
    }

    let mut row_values = Vec::with_capacity(reference_evidence.row_references.len());
    for (reference, kind) in reference_evidence
        .row_references
        .iter()
        .zip(&reference_evidence.reference_kinds)
    {
        if *kind != HnidKind::HeapId {
            return Err(format!(
                "recipient identity HNID 0x{reference:08x} is not heap-resident"
            ));
        }
        let bytes = heap
            .allocation_by_hid(heap_bytes, *reference, base_offset)
            .map_err(|error| error.to_string())?;
        row_values.push(decode_string(bytes, property_type)?);
    }

    Ok(RecipientIdentityStringEvidence {
        property_tag: reference_evidence.property_tag,
        property_name: reference_evidence.property_name.clone(),
        row_values,
    })
}

fn decode_string(bytes: &[u8], property_type: u16) -> Result<String, String> {
    match property_type {
        PT_STRING8 => {
            let end = bytes.iter().position(|byte| *byte == 0).unwrap_or(bytes.len());
            String::from_utf8(bytes[..end].to_vec())
                .map_err(|_| "recipient identity STRING8 value is not valid UTF-8".to_string())
        }
        PT_UNICODE => {
            if !bytes.len().is_multiple_of(2) {
                return Err("recipient identity Unicode value has odd byte length".to_string());
            }
            let units = bytes
                .chunks_exact(2)
                .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
                .take_while(|unit| *unit != 0)
                .collect::<Vec<_>>();
            String::from_utf16(&units)
                .map_err(|_| "recipient identity Unicode value is invalid UTF-16".to_string())
        }
        _ => Err("recipient identity property is not a supported string type".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_recipient_identity_heap_strings;
    use crate::pst::heap::HeapOnNode;
    use crate::pst::tc_recipient_identity_reference::RecipientIdentityReferenceEvidence;
    use crate::pst::tcinfo::HnidKind;

    #[test]
    fn resolves_unicode_heap_strings_for_each_recipient_row() {
        let heap_bytes = sample_heap(&[
            "Alice\0".encode_utf16().flat_map(u16::to_le_bytes).collect(),
            "Bob\0".encode_utf16().flat_map(u16::to_le_bytes).collect(),
        ]);
        let heap = HeapOnNode::parse(&heap_bytes, 0).unwrap();
        let evidence = RecipientIdentityReferenceEvidence {
            property_tag: 0x3001_001f,
            property_name: "PidTagDisplayName".to_string(),
            data_offset: 0,
            row_references: vec![0x20, 0x40],
            reference_kinds: vec![HnidKind::HeapId, HnidKind::HeapId],
        };

        let resolved = resolve_recipient_identity_heap_strings(&evidence, &heap, &heap_bytes, 0)
            .expect("heap strings should resolve");

        assert_eq!(resolved.row_values, vec!["Alice", "Bob"]);
    }

    #[test]
    fn fails_closed_for_node_references() {
        let heap_bytes = sample_heap(&[b"Alice\0".to_vec()]);
        let heap = HeapOnNode::parse(&heap_bytes, 0).unwrap();
        let evidence = RecipientIdentityReferenceEvidence {
            property_tag: 0x3001_001e,
            property_name: "PidTagDisplayName".to_string(),
            data_offset: 0,
            row_references: vec![0x24],
            reference_kinds: vec![HnidKind::NodeId],
        };

        let error = resolve_recipient_identity_heap_strings(&evidence, &heap, &heap_bytes, 0)
            .expect_err("node references must not be guessed");
        assert!(error.contains("not heap-resident"));
    }

    fn sample_heap(values: &[Vec<u8>]) -> Vec<u8> {
        let data_start = 16usize;
        let data_len: usize = values.iter().map(Vec::len).sum();
        let page_map_offset = data_start + data_len;
        let mut buf = vec![0u8; page_map_offset + 4 + (values.len() + 1) * 2];
        buf[0..2].copy_from_slice(&(page_map_offset as u16).to_le_bytes());
        buf[2] = 0xec;
        buf[3] = 0xbc;
        buf[4..8].copy_from_slice(&0x20u32.to_le_bytes());

        let mut cursor = data_start;
        for value in values {
            buf[cursor..cursor + value.len()].copy_from_slice(value);
            cursor += value.len();
        }

        buf[page_map_offset..page_map_offset + 2]
            .copy_from_slice(&(values.len() as u16).to_le_bytes());
        let offsets_start = page_map_offset + 4;
        let mut offset = data_start as u16;
        buf[offsets_start..offsets_start + 2].copy_from_slice(&offset.to_le_bytes());
        for (index, value) in values.iter().enumerate() {
            offset += value.len() as u16;
            let start = offsets_start + (index + 1) * 2;
            buf[start..start + 2].copy_from_slice(&offset.to_le_bytes());
        }
        buf
    }
}
