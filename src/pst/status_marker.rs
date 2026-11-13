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
    let tail = &status[marker_end(status, key, separator)?..];
    let value = tail.split([',', ';']).next().unwrap_or(tail);
    // `split` on the marker ends the piece at the next marker, so do the same here.
    Some(match marker_end(value, key, separator) {
        Some(next_end) => &value[..next_end - key.len() - separator.len_utf8()],
        None => value,
    })
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

#[cfg(test)]
mod tests {
    use super::marker_value;

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
}
