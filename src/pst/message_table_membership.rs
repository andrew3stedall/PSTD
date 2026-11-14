use std::collections::HashSet;

use crate::error::PstdResult;
use crate::pst::bbt::BbtIndex;
use crate::pst::limits::ParserLimits;
use crate::pst::message_table::MessageTableNodeType;
use crate::pst::nbt::NbtEntry;
use crate::pst::primitives::NodeId;
use crate::pst::reader::PstByteReader;
use crate::pst::tc_reader::{read_row_ids, TcSource};

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

    let row_ids = read_row_ids(
        TcSource {
            reader,
            bbt,
            limits,
        },
        entry.data_block_id,
    )?;

    let expected_type = match table_type {
        MessageTableNodeType::ContentsTable => 0x04,
        MessageTableNodeType::AssociatedContentsTable => 0x08,
        MessageTableNodeType::SearchContentsTable | MessageTableNodeType::HierarchyTable => {
            unreachable!("non-authoritative table types returned above")
        }
    };
    let mut seen = HashSet::new();
    let mut member_node_ids = Vec::with_capacity(row_ids.len());
    for row_key in row_ids {
        if row_key == 0 {
            return Ok(MessageTableMembership {
                member_node_ids: Vec::new(),
                status: "message_table_membership_zero_row_key".to_string(),
            });
        }
        if row_key & 0x1f != expected_type {
            return Ok(MessageTableMembership {
                member_node_ids: Vec::new(),
                status: format!(
                    "message_table_membership_row_type_mismatch; row_key=0x{:08x}; table_type={}",
                    row_key,
                    table_type.status_label()
                ),
            });
        }
        if !seen.insert(row_key) {
            return Ok(MessageTableMembership {
                member_node_ids: Vec::new(),
                status: format!(
                    "message_table_membership_duplicate_row_key; row_key=0x{:08x}",
                    row_key
                ),
            });
        }
        member_node_ids.push(NodeId(u64::from(row_key)));
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
