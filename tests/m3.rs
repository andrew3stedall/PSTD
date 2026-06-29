use pstd::pst::heap::HeapOnNode;
use pstd::pst::bth::BthMap;

#[test]
fn heap_smoke() {
    let mut bytes = vec![];
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    let heap = HeapOnNode::parse(&bytes, 0).unwrap();
    assert_eq!(heap.allocations.len(), 0);
}

#[test]
fn bth_smoke() {
    let mut bytes = vec![4, 4];
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    let bth = BthMap::parse(&bytes, 0).unwrap();
    assert_eq!(bth.entries.len(), 0);
}
