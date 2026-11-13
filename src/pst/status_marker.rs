//! Allocation-free lookup of `key=value` / `key:value` markers in status strings.
//!
//! Status strings are `;`- and `,`-separated diagnostics. Aggregated counters are read
//! back from them many times per message, so the lookup must not allocate or build a
//! substring searcher per call.

/// Returns the text that follows the first occurrence of `key` immediately followed by
/// `separator`, up to (not including) the next `,`, `;` or repeated marker.
///
/// The match is a plain substring match: `key` may be the tail of a longer name. The
/// result is identical to `status.split(&format!("{key}{separator}")).nth(1)` followed
/// by `.split([',', ';']).next()`.
pub(crate) fn marker_value<'a>(status: &'a str, key: &str, separator: char) -> Option<&'a str> {
    debug_assert!(separator.is_ascii());
    Some(value_at(
        status,
        marker_end(status, key, separator)?,
        key,
        separator,
    ))
}

/// Value that starts at `tail_start`, the byte index just past a marker.
fn value_at<'a>(status: &'a str, tail_start: usize, key: &str, separator: char) -> &'a str {
    let tail = &status[tail_start..];
    let value = tail.split([',', ';']).next().unwrap_or(tail);
    // `split` on the marker ends the piece at the next marker, so do the same here.
    match marker_end(value, key, separator) {
        Some(next_end) => &value[..next_end - key.len() - separator.len_utf8()],
        None => value,
    }
}

/// Byte index just past the first `key` + `separator` occurrence.
fn marker_end(text: &str, key: &str, separator: char) -> Option<usize> {
    let key = key.as_bytes();
    let bytes = text.as_bytes();
    text.match_indices(separator)
        .map(|(index, _)| index)
        .find(|&index| bytes[..index].ends_with(key))
        .map(|index| index + separator.len_utf8())
}

/// How a counter value is read from one status string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MarkerStyle {
    /// First `key:` marker, otherwise first `key=` marker; the value is not trimmed.
    ColonThenEquals,
    /// First `key=` marker; the value is trimmed.
    EqualsTrimmed,
}

/// Sums numeric marker values for a fixed key set over many status strings.
///
/// Each added status is scanned once for all keys, instead of once per key. For every
/// key the contribution of one status equals `marker_value` for that key and style,
/// parsed as `usize`, or zero when the marker is absent or not a number.
pub(crate) struct MarkerTotals {
    keys: &'static [&'static str],
    style: MarkerStyle,
    /// Length of the key tail used as the bucket key: the shortest key length.
    tail_len: usize,
    by_tail: std::collections::HashMap<&'static [u8], Vec<usize>>,
    totals: Vec<usize>,
    overflowed: Vec<bool>,
    /// Per key: byte index just past the first `:` marker and the first `=` marker.
    first: Vec<[Option<usize>; 2]>,
    touched: Vec<usize>,
}

impl MarkerTotals {
    /// `keys` must be non-empty strings.
    pub(crate) fn new(keys: &'static [&'static str], style: MarkerStyle) -> Self {
        let tail_len = keys.iter().map(|key| key.len()).min().unwrap_or(0);
        assert!(tail_len > 0, "marker keys must be non-empty");
        let mut by_tail: std::collections::HashMap<&'static [u8], Vec<usize>> =
            std::collections::HashMap::with_capacity(keys.len());
        for (index, key) in keys.iter().enumerate() {
            let bytes = key.as_bytes();
            by_tail
                .entry(&bytes[bytes.len() - tail_len..])
                .or_default()
                .push(index);
        }
        Self {
            keys,
            style,
            tail_len,
            by_tail,
            totals: vec![0; keys.len()],
            overflowed: vec![false; keys.len()],
            first: vec![[None; 2]; keys.len()],
            touched: Vec::new(),
        }
    }

    pub(crate) fn add(&mut self, status: &str) {
        let bytes = status.as_bytes();
        for (index, &byte) in bytes.iter().enumerate() {
            let slot = match byte {
                b':' if self.style == MarkerStyle::ColonThenEquals => 0,
                b'=' => 1,
                _ => continue,
            };
            if index < self.tail_len {
                continue;
            }
            let Some(candidates) = self.by_tail.get(&bytes[index - self.tail_len..index]) else {
                continue;
            };
            for &key_index in candidates {
                if self.first[key_index][slot].is_none()
                    && bytes[..index].ends_with(self.keys[key_index].as_bytes())
                {
                    if self.first[key_index] == [None; 2] {
                        self.touched.push(key_index);
                    }
                    self.first[key_index][slot] = Some(index + 1);
                }
            }
        }
        for key_index in self.touched.drain(..) {
            let [colon, equals] = std::mem::take(&mut self.first[key_index]);
            let (tail_start, separator) = match colon {
                Some(start) => (start, ':'),
                None => (equals.expect("touched key has a marker"), '='),
            };
            let value = value_at(status, tail_start, self.keys[key_index], separator);
            let value = match self.style {
                MarkerStyle::ColonThenEquals => value,
                MarkerStyle::EqualsTrimmed => value.trim(),
            };
            let (total, overflowed) =
                self.totals[key_index].overflowing_add(value.parse::<usize>().unwrap_or(0));
            self.totals[key_index] = total;
            self.overflowed[key_index] |= overflowed;
        }
    }

    /// Total for `key`. Panics when `key` was not registered, which is a coding error.
    /// A total combined with another must use `+`, as the replaced code did.
    pub(crate) fn total(&self, key: &str) -> usize {
        let index = self
            .keys
            .iter()
            .position(|candidate| *candidate == key)
            .unwrap_or_else(|| panic!("status marker key {key} is not registered"));
        // Same policy as the plain `usize` sum this replaces: overflow panics in debug
        // builds and wraps in release builds, and only for a total that is read.
        debug_assert!(
            !self.overflowed[index],
            "attempt to add with overflow in status marker {key}"
        );
        self.totals[index]
    }
}

#[cfg(test)]
mod tests {
    use super::{marker_value, MarkerStyle, MarkerTotals};

    fn reference<'a>(status: &'a str, key: &str, separator: char) -> Option<&'a str> {
        let marker = format!("{key}{separator}");
        status
            .split(&marker)
            .nth(1)
            .and_then(|tail| tail.split([',', ';']).next())
    }

    #[test]
    fn matches_the_split_based_reference() {
        let statuses = [
            "",
            "rows=3",
            "rows=3; columns=4, values= 12 ;tail",
            "a_rows=1; rows=2; rows=3",
            "rows:7; rows=9",
            "rows=; columns=x",
            "plausible:4,suspicious:2;byte_swapped_selected:1",
            "rows",
            "=rows=",
            "rows==5",
            "prefix rows=5 trailing",
            "unicode é rows=5; ü",
            "rowsrows=8",
            "rows=rows=5",
            "rows=1rows=2; rows=3",
            "rows:rows=5,rows:6",
        ];
        let keys = [
            "rows",
            "columns",
            "values",
            "plausible",
            "suspicious",
            "a_rows",
            "x",
            "",
        ];
        for status in statuses {
            for key in keys {
                for separator in ['=', ':'] {
                    assert_eq!(
                        marker_value(status, key, separator),
                        reference(status, key, separator),
                        "status={status:?} key={key:?} separator={separator:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn substring_keys_match_the_first_occurrence() {
        assert_eq!(marker_value("a_rows=1; rows=2", "rows", '='), Some("1"));
        assert_eq!(marker_value("rows=2", "columns", '='), None);
    }

    const STATUSES: [&str; 18] = [
        "",
        "rows=3",
        "rows=3; columns=4, values= 12 ;tail",
        "a_rows=1; rows=2; rows=3",
        "rows:7; rows=9",
        "rows=9; rows:7",
        "rows:x; rows=9",
        "rows=; columns=x",
        "plausible:4,suspicious:2;byte_swapped_selected:1",
        "rows",
        "=rows=",
        "rows==5",
        "prefix rows=5 trailing",
        "unicode é rows=5; ü columns: 6",
        "rowsrows=8",
        "rows=5rows=6; columns=1columns:2",
        "ws=4; rows= 11 ; table_rows=2",
        "columns:3;values:4,ws:5,rows:6",
    ];
    const KEYS: [&str; 7] = [
        "rows",
        "columns",
        "values",
        "plausible",
        "a_rows",
        "ws",
        "table_rows",
    ];

    fn reference_counter(status: &str, key: &str, style: MarkerStyle) -> usize {
        match style {
            MarkerStyle::ColonThenEquals => reference(status, key, ':')
                .or_else(|| reference(status, key, '='))
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(0),
            MarkerStyle::EqualsTrimmed => reference(status, key, '=')
                .and_then(|value| value.trim().parse::<usize>().ok())
                .unwrap_or(0),
        }
    }

    #[test]
    fn totals_match_per_key_reference_counters() {
        for style in [MarkerStyle::ColonThenEquals, MarkerStyle::EqualsTrimmed] {
            let mut totals = MarkerTotals::new(&KEYS, style);
            for status in STATUSES {
                // Each status alone, so one mismatch is attributable.
                let mut single = MarkerTotals::new(&KEYS, style);
                single.add(status);
                for key in KEYS {
                    assert_eq!(
                        single.total(key),
                        reference_counter(status, key, style),
                        "status={status:?} key={key:?} style={style:?}"
                    );
                }
                totals.add(status);
            }
            for key in KEYS {
                let expected: usize = STATUSES
                    .iter()
                    .map(|status| reference_counter(status, key, style))
                    .sum();
                assert_eq!(totals.total(key), expected, "key={key:?} style={style:?}");
            }
        }
    }

    #[test]
    #[should_panic(expected = "is not registered")]
    fn unregistered_key_is_a_coding_error() {
        MarkerTotals::new(&KEYS, MarkerStyle::EqualsTrimmed).total("missing");
    }

    #[test]
    fn overflow_wraps_in_release_and_panics_in_debug_only_when_read() {
        let mut totals = MarkerTotals::new(&KEYS, MarkerStyle::EqualsTrimmed);
        totals.add(&format!("rows={}", usize::MAX));
        totals.add("rows=2; columns=1");
        // A key that did not overflow is unaffected.
        assert_eq!(totals.total("columns"), 1);
        let read = std::panic::catch_unwind(|| totals.total("rows"));
        if cfg!(debug_assertions) {
            assert!(read.is_err());
        } else {
            assert_eq!(read.unwrap(), 1);
        }
    }
}
