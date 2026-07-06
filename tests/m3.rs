use pstd::pst::bth::BthMap;
use pstd::pst::heap::HeapOnNode;

#[test]
fn heap_smoke() {
    let mut bytes = vec![0; 16];
    bytes[0..2].copy_from_slice(&8u16.to_le_bytes());
    bytes[2] = 0xec;
    bytes[3] = 0xbc;
    bytes[4..8].copy_from_slice(&0u32.to_le_bytes());
    bytes[8..10].copy_from_slice(&0u16.to_le_bytes());
    bytes[10..12].copy_from_slice(&0u16.to_le_bytes());
    bytes[12..14].copy_from_slice(&8u16.to_le_bytes());
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
