//! Measures BBT block resolution cost against the previous linear-scan lookup.
//!
//! Run with `cargo run --release --example index_lookup_benchmark`.

use std::hint::black_box;
use std::time::Instant;

use pstd::pst::bbt::{BbtEntry, BbtIndex};
use pstd::pst::primitives::{BlockId, ByteOffset};

fn main() {
    println!("entries\tlinear_ms\tindexed_ms\tspeedup");
    for count in [1_000u64, 10_000, 100_000] {
        let entries: Vec<BbtEntry> = (0..count)
            .map(|ordinal| BbtEntry {
                block_id: BlockId(ordinal * 4),
                offset: ByteOffset(ordinal * 512),
                size: 512,
            })
            .collect();
        let index = BbtIndex {
            root: None,
            entries,
            parsed_pages: 0,
            discovered_child_pages: 0,
            traversal_error_count: 0,
            duplicate_entry_count: 0,
            truncated_entry_count: 0,
            status: "benchmark".to_string(),
            lookup_index: Default::default(),
            page_limit_reached: false,
        };

        // Resolve every block once, as a whole-file extraction does.
        let started = Instant::now();
        let mut linear_hits = 0u64;
        for ordinal in 0..count {
            let block_id = BlockId(ordinal * 4);
            let found = index
                .entries
                .iter()
                .find(|entry| entry.block_id == block_id)
                .map(BbtEntry::block_ref);
            linear_hits += u64::from(black_box(found).is_some());
        }
        let linear = started.elapsed();

        let started = Instant::now();
        let mut indexed_hits = 0u64;
        for ordinal in 0..count {
            let found = index.lookup(BlockId(ordinal * 4));
            indexed_hits += u64::from(black_box(found).is_some());
        }
        let indexed = started.elapsed();

        assert_eq!(linear_hits, count);
        assert_eq!(indexed_hits, count);
        println!(
            "{count}\t{:.3}\t{:.3}\t{:.1}x",
            linear.as_secs_f64() * 1000.0,
            indexed.as_secs_f64() * 1000.0,
            linear.as_secs_f64() / indexed.as_secs_f64().max(f64::EPSILON)
        );
    }
}
