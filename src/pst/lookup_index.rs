use std::collections::HashMap;
use std::sync::OnceLock;

/// Below this many entries a linear scan is cheaper than building a hash map.
const LINEAR_SCAN_MAX_ENTRIES: usize = 16;

/// Lazily built first-occurrence position index over a slice of B-tree leaf entries.
///
/// The NBT and BBT indexes keep their entries in a public `Vec` in traversal order and
/// resolve a key to the *first* matching entry. A linear scan per lookup makes every
/// block or node resolution O(entries), which is quadratic over a whole PST. This cache
/// preserves the first-match result while answering in O(1) after one O(entries) build.
///
/// The cache is derived state: it is never serialized, compares nothing, and a clone
/// starts empty. It is built from the entries seen by the first lookup. If the entry
/// count later changes, or a cached position no longer holds the requested key, lookups
/// fall back to the original linear scan, so a stale cache cannot return a wrong entry.
/// Replacing entries in place with different keys while keeping the same length after
/// the first lookup is not supported.
#[derive(Default)]
pub struct FirstKeyIndex {
    cell: OnceLock<(usize, HashMap<u64, usize>)>,
}

impl FirstKeyIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the position of the first entry whose key equals `key`.
    pub fn first_position<T>(
        &self,
        entries: &[T],
        key: u64,
        key_of: impl Fn(&T) -> u64,
    ) -> Option<usize> {
        if entries.len() <= LINEAR_SCAN_MAX_ENTRIES {
            return linear_position(entries, key, &key_of);
        }
        let (indexed_len, positions) = self.cell.get_or_init(|| {
            let mut positions = HashMap::with_capacity(entries.len());
            for (position, entry) in entries.iter().enumerate() {
                positions.entry(key_of(entry)).or_insert(position);
            }
            (entries.len(), positions)
        });
        if *indexed_len != entries.len() {
            return linear_position(entries, key, &key_of);
        }
        match positions.get(&key) {
            Some(&position) if key_of(&entries[position]) == key => Some(position),
            Some(_) => linear_position(entries, key, &key_of),
            None => None,
        }
    }
}

fn linear_position<T>(entries: &[T], key: u64, key_of: &impl Fn(&T) -> u64) -> Option<usize> {
    entries.iter().position(|entry| key_of(entry) == key)
}

impl Clone for FirstKeyIndex {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl std::fmt::Debug for FirstKeyIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(if self.cell.get().is_some() {
            "FirstKeyIndex(built)"
        } else {
            "FirstKeyIndex(unbuilt)"
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(value: &u64) -> u64 {
        *value
    }

    #[test]
    fn matches_linear_first_occurrence_for_small_and_large_slices() {
        for len in [0usize, 1, 16, 17, 500] {
            // Keys repeat so that first-occurrence semantics are exercised.
            let entries: Vec<u64> = (0..len as u64).map(|value| value % 97).collect();
            let index = FirstKeyIndex::new();
            for key in 0..120u64 {
                assert_eq!(
                    index.first_position(&entries, key, identity),
                    entries.iter().position(|value| *value == key),
                    "len={len} key={key}"
                );
            }
        }
    }

    #[test]
    fn stale_cache_falls_back_to_linear_scan() {
        let mut entries: Vec<u64> = (0..100).collect();
        let index = FirstKeyIndex::new();
        assert_eq!(index.first_position(&entries, 40, identity), Some(40));

        // Length change invalidates the cache entirely.
        entries.insert(0, 1000);
        assert_eq!(index.first_position(&entries, 40, identity), Some(41));
        assert_eq!(index.first_position(&entries, 1000, identity), Some(0));

        // Same length, moved key: the cached position is verified before use.
        let mut entries: Vec<u64> = (0..100).collect();
        let index = FirstKeyIndex::new();
        assert_eq!(index.first_position(&entries, 40, identity), Some(40));
        entries.swap(40, 60);
        assert_eq!(index.first_position(&entries, 40, identity), Some(60));
    }

    #[test]
    fn clone_starts_unbuilt() {
        let entries: Vec<u64> = (0..100).collect();
        let index = FirstKeyIndex::new();
        index.first_position(&entries, 5, identity);
        assert_eq!(format!("{index:?}"), "FirstKeyIndex(built)");
        assert_eq!(format!("{:?}", index.clone()), "FirstKeyIndex(unbuilt)");
    }
}
