use std::collections::HashSet;

use crate::error::PstdResult;
use crate::pst::bbt::BbtIndex;
use crate::pst::limits::ParserLimits;
use crate::pst::message_table::MessageTableNodeType;
use crate::pst::nbt::NbtEntry;
use crate::pst::payload::load_payload_block;
use crate::pst::primitives::NodeId;
use crate::pst::reader::PstByteReader;
use crate::pst::tc_heap::resolve_tcinfo_from_heap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageTableMembership {
    pub member_node_ids: Vec<NodeId>,
    pub status: String,
}

pub fn load_message_table_membership(
    reader: &PstByteReader,
    bbt: &BbtIndex,
    entry: &NbtEntry,
    table_type: MessageTableNodeType,
    limits: ParserLimits,
) -> PstdResult<MessageTableMembership> {
    if matches!(
        table_type,
        MessageTableNodeType::SearchContentsTable | MessageTableNodeType::HierarchyTable
    ) {
        return Ok(MessageTableMembership {
            member_node_ids: Vec::new(),
            status: format!(
                "message_table_membership_non_authoritative; table_type={}",
                table_type.status_label()
            ),
        });
    }

    let payload = load_payload_block(reader, bbt, entry.data_block_id, limits)?;
    let report = resolve_tcinfo_from_heap(&payload.bytes, payload.block_ref.offset.0)?;
    let Some(row_index) = report.row_index_report else {
        return Ok(MessageTableMembership {
            member_node_ids: Vec::new(),
            status: "message_table_membership_row_index_unavailable".to_string(),
        });
    };
    if row_index.truncated {
        return Ok(MessageTableMembership {
            member_node_ids: Vec::new(),
            status: "message_table_membership_row_index_truncated".to_string(),
        });
    }

    let expected_type = match table_type {
        MessageTableNodeType::ContentsTable => 0x04,
        MessageTableNodeType::AssociatedContentsTable => 0x08,
        MessageTableNodeType::SearchContentsTable | MessageTableNodeType::HierarchyTable => {
            unreachable!("non-authoritative table types returned above")
        }
    };
    let mut seen = HashSet::new();
    let mut member_node_ids = Vec::with_capacity(row_index.entries.len());
    for row in row_index.entries {
        if row.row_key == 0 {
            return Ok(MessageTableMembership {
                member_node_ids: Vec::new(),
                status: "message_table_membership_zero_row_key".to_string(),
            });
        }
        if row.row_key & 0x1f != expected_type {
            return Ok(MessageTableMembership {
                member_node_ids: Vec::new(),
                status: format!(
                    "message_table_membership_row_type_mismatch; row_key=0x{:08x}; table_type={}",
                    row.row_key,
                    table_type.status_label()
                ),
            });
        }
        if !seen.insert(row.row_key) {
            return Ok(MessageTableMembership {
                member_node_ids: Vec::new(),
                status: format!(
                    "message_table_membership_duplicate_row_key; row_key=0x{:08x}",
                    row.row_key
                ),
            });
        }
        member_node_ids.push(NodeId(u64::from(row.row_key)));
    }

    Ok(MessageTableMembership {
        status: format!(
            "message_table_membership_rows_exact; table_type={}; rows={}",
            table_type.status_label(),
            member_node_ids.len()
        ),
        member_node_ids,
    })
}

#[cfg(test)]
mod tests {
    use super::MessageTableMembership;
    use crate::pst::primitives::NodeId;

    #[test]
    fn membership_result_preserves_exact_node_ids() {
        let result = MessageTableMembership {
            member_node_ids: vec![NodeId(0x24), NodeId(0x44)],
            status: "message_table_membership_rows_exact; table_type=contents_table; rows=2"
                .to_string(),
        };
        assert_eq!(result.member_node_ids, vec![NodeId(0x24), NodeId(0x44)]);
    }
}
