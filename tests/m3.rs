use pstd::pst::bth::BthMap;
use pstd::pst::heap::HeapOnNode;

#[test]
fn heap_smoke() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    let heap = HeapOnNode::parse(&bytes, 0).unwrap();
    assert!(heap.allocations.is_empty());
}

#[test]
fn bth_smoke() {
    let mut bytes = vec![4, 4];
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    let bth = BthMap::parse(&bytes, 0).unwrap();
    assert!(bth.entries.is_empty());
}
