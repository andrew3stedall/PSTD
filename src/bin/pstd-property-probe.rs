use std::env;

use pstd::pst::bbt::BbtIndex;
use pstd::pst::bth::BthMap;
use pstd::pst::header::PstHeader;
use pstd::pst::heap::HeapOnNode;
use pstd::pst::limits::ParserLimits;
use pstd::pst::nbt::NbtIndex;
use pstd::pst::node_payload::load_node_property_context;
use pstd::pst::primitives::NodeId;
use pstd::pst::property_context::{PropertyContext, PropertyValue};
use pstd::pst::reader::PstByteReader;
use pstd::pst::subnodes::{load_recursive_subnode_blocks, SubnodeReference};
use serde_json::{json, Value};

fn property_rows<'a>(values: impl Iterator<Item = &'a PropertyValue>) -> Vec<Value> {
    let mut values = values.collect::<Vec<_>>();
    values.sort_by_key(|value| value.tag);
    values
        .into_iter()
        .map(|value| {
            json!({
                "tag": format!("0x{:08x}", value.tag),
                "name": value.name,
                "raw_hex": value.raw.iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
                "decoded": value.decoded,
                "status": value.status,
            })
        })
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let path = args.next().ok_or("missing PST path")?;
    let node = args.next().ok_or("missing node id")?;
    if args.next().is_some() {
        return Err("usage: pstd-property-probe <pst> <node-id>".into());
    }
    let node_value = u64::from_str_radix(node.trim_start_matches("0x"), 16)?;
    let reader = PstByteReader::open(&path)?;
    let header = PstHeader::parse(&reader)?;
    let limits = ParserLimits::default();
    let bbt = BbtIndex::load_root_with_limits(&reader, header.roots.bbt_root, limits)?;
    let nbt = NbtIndex::load_root_with_limits(&reader, header.roots.nbt_root, limits)?;
    let entry = nbt.lookup(NodeId(node_value)).ok_or("node not found")?;
    let loaded = load_node_property_context(&reader, &bbt, entry, limits)?;

    let mut subnode_contexts = Vec::new();
    if let Some(subnode_block_id) = entry.subnode_block_id {
        let reference = SubnodeReference {
            node_id: entry.node_id,
            subnode_block_id,
            status: "probe".to_string(),
        };
        let loaded_subnodes = load_recursive_subnode_blocks(&reader, &bbt, &reference, 1, limits);
        for payload in loaded_subnodes.payloads {
            let Ok(heap) = HeapOnNode::parse(&payload.bytes, payload.block_ref.offset.0) else {
                continue;
            };
            if heap.header.client_signature != 0xbc {
                continue;
            }
            let Ok(bth) = BthMap::parse_property_context_from_heap(
                &heap,
                &payload.bytes,
                payload.block_ref.offset.0,
            ) else {
                continue;
            };
            let report = PropertyContext::from_bth_with_report(&bth)?;
            subnode_contexts.push(json!({
                "block_id": format!("0x{:x}", payload.block_id.0),
                "offset": payload.block_ref.offset.0,
                "properties": property_rows(report.context.values.values()),
            }));
        }
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "message_properties": property_rows(loaded.properties.values.values()),
            "subnode_property_contexts": subnode_contexts,
        }))?
    );
    Ok(())
}
