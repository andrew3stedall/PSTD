use std::env;

use pstd::pst::bbt::BbtIndex;
use pstd::pst::header::PstHeader;
use pstd::pst::limits::ParserLimits;
use pstd::pst::nbt::NbtIndex;
use pstd::pst::node_payload::load_node_property_context;
use pstd::pst::primitives::NodeId;
use pstd::pst::reader::PstByteReader;
use serde_json::json;

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
    let mut values = loaded.properties.values.values().collect::<Vec<_>>();
    values.sort_by_key(|value| value.tag);
    let rows = values
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
        .collect::<Vec<_>>();
    println!("{}", serde_json::to_string_pretty(&rows)?);
    Ok(())
}
