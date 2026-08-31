use std::collections::HashMap;

use crate::error::PstdResult;
use crate::pst::bth::BthMap;
use crate::pst::mapi::{
    byte_swapped_tag, decode_string8_with_status, decode_value_with_fallback,
    has_known_value_type, property_def, resolve_string8_charset, value_summary, MapiValue,
    MapiValueType, PR_INTERNET_CPID, PR_MESSAGE_CODEPAGE,
};

const PQ10_TRAVERSAL_STATUS_TAG: u32 = 0xffff_fffe;
const PQ10_TRAVERSAL_STATUS_PREFIX: &str = "pq10_traversal=";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PropertyValue {
    pub tag: u32,
    pub name: String,
    pub raw: Vec<u8>,
    pub decoded: Option<MapiValue>,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct PropertyContext {
    pub values: HashMap<u32, PropertyValue>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PropertyContextParseReport {
    pub context: PropertyContext,
    pub bth_entry_count: usize,
    pub parsed_property_count: usize,
    pub selected_property_count: usize,
    pub unknown_property_count: usize,
    pub unknown_property_tags: Vec<u32>,
    pub plausible_property_tag_count: usize,
    pub suspicious_property_tag_count: usize,
    pub byte_swapped_selected_property_count: usize,
    pub skipped_key_count: usize,
    pub decode_error_count: usize,
    pub charset_conversion_error_count: usize,
    pub charset_resolution: crate::pst::mapi::CharsetResolution,
    pub status: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct InterpretedTag {
    tag: u32,
    is_plausible: bool,
    was_byte_swapped: bool,
}

impl PropertyContext {
    pub fn from_values(values: HashMap<u32, PropertyValue>) -> Self {
        Self { values }
    }

    pub fn from_bth(bth: &BthMap) -> PstdResult<Self> {
        Ok(Self::from_bth_with_report(bth)?.context)
    }

    pub fn from_bth_with_report(bth: &BthMap) -> PstdResult<PropertyContextParseReport> {
        Self::from_bth_with_fallback_charset(bth, None)
    }

    pub fn from_bth_with_fallback_charset(
        bth: &BthMap,
        fallback_charset: Option<&str>,
    ) -> PstdResult<PropertyContextParseReport> {
        let mut message_codepage = None;
        let mut internet_cpid = None;
        for entry in &bth.entries {
            if entry.key.len() < 4 {
                continue;
            }
            let raw_tag =
                u32::from_le_bytes([entry.key[0], entry.key[1], entry.key[2], entry.key[3]]);
            let interpreted = interpret_property_tag(raw_tag);
            match interpreted.tag {
                PR_MESSAGE_CODEPAGE => message_codepage = Some(entry.value.as_slice()),
                PR_INTERNET_CPID => internet_cpid = Some(entry.value.as_slice()),
                _ => {}
            }
        }
        let charset_resolution =
            resolve_string8_charset(message_codepage, internet_cpid, fallback_charset);
        let mut values = HashMap::new();
        let mut selected_property_count = 0usize;
        let mut unknown_property_count = 0usize;
        let mut unknown_property_tags = Vec::new();
        let mut plausible_property_tag_count = 0usize;
        let mut suspicious_property_tag_count = 0usize;
        let mut byte_swapped_selected_property_count = 0usize;
        let mut skipped_key_count = 0usize;
        let mut decode_error_count = 0usize;
        let mut charset_conversion_error_count = 0usize;

        for entry in &bth.entries {
            if entry.key.len() < 4 {
                skipped_key_count += 1;
                continue;
            }
            let raw_tag =
                u32::from_le_bytes([entry.key[0], entry.key[1], entry.key[2], entry.key[3]]);
            let interpreted = interpret_property_tag(raw_tag);
            if interpreted.is_plausible {
                plausible_property_tag_count += 1;
            } else {
                suspicious_property_tag_count += 1;
            }

            let Some(def) = property_def(interpreted.tag) else {
                unknown_property_count += 1;
                unknown_property_tags.push(interpreted.tag);
                values.insert(
                    interpreted.tag,
                    PropertyValue {
                        tag: interpreted.tag,
                        name: format!("unknown_0x{:08x}", interpreted.tag),
                        raw: entry.value.clone(),
                        decoded: None,
                        status: unknown_property_status(raw_tag, interpreted),
                    },
                );
                continue;
            };
            if interpreted.was_byte_swapped {
                byte_swapped_selected_property_count += 1;
            }
            let decoded = match decode_value_with_fallback(
                def.value_type,
                &entry.value,
                Some(charset_resolution.charset.as_str()),
            ) {
                Ok(value) => {
                    if def.value_type == MapiValueType::String8
                        && decode_string8_with_status(
                            &entry.value,
                            Some(charset_resolution.charset.as_str()),
                        )
                        .1
                    {
                        charset_conversion_error_count += 1;
                    }
                    Some(value)
                }
                Err(_) => {
                    decode_error_count += 1;
                    None
                }
            };
            selected_property_count += 1;
            values.insert(
                interpreted.tag,
                PropertyValue {
                    tag: interpreted.tag,
                    name: def.name.to_string(),
                    raw: entry.value.clone(),
                    decoded,
                    status: selected_property_status(raw_tag, interpreted),
                },
            );
        }

        let parsed_property_count = values.len();
        unknown_property_tags.sort_unstable();
        unknown_property_tags.dedup();
        let tag_shape_status = format!(
            "tag_shape=plausible:{plausible_property_tag_count},suspicious:{suspicious_property_tag_count},byte_swapped_selected:{byte_swapped_selected_property_count}"
        );
        let status = if decode_error_count == 0 && skipped_key_count == 0 {
            if unknown_property_count == 0 {
                format!("property_context_parsed; {tag_shape_status}")
            } else {
                format!(
                    "property_context_parsed_with_unknown_properties; unknown_properties={unknown_property_count}; unknown_tag_sample={}; {tag_shape_status}",
                    unknown_tag_sample(&unknown_property_tags)
                )
            }
        } else {
            format!(
                "property_context_parsed_with_issues; decode_errors={decode_error_count}; skipped_keys={skipped_key_count}; unknown_properties={unknown_property_count}; unknown_tag_sample={}; {tag_shape_status}",
                unknown_tag_sample(&unknown_property_tags)
            )
        };
        let status = format!(
            "{status}; {}; charset_conversion_errors={charset_conversion_error_count}",
            charset_resolution.status
        );

        Ok(PropertyContextParseReport {
            context: Self { values },
            bth_entry_count: bth.entries.len(),
            parsed_property_count,
            selected_property_count,
            unknown_property_count,
            unknown_property_tags,
            plausible_property_tag_count,
            suspicious_property_tag_count,
            byte_swapped_selected_property_count,
            skipped_key_count,
            decode_error_count,
            charset_conversion_error_count,
            charset_resolution,
            status,
        })
    }

    pub fn with_pq10_traversal_status(mut self, traversal_status: &str) -> Self {
        self.values.insert(
            PQ10_TRAVERSAL_STATUS_TAG,
            PropertyValue {
                tag: PQ10_TRAVERSAL_STATUS_TAG,
                name: "pq10_traversal_status".to_string(),
                raw: Vec::new(),
                decoded: None,
                status: format!("{PQ10_TRAVERSAL_STATUS_PREFIX}{traversal_status}"),
            },
        );
        self
    }

    pub fn value(&self, tag: u32) -> Option<&PropertyValue> {
        self.values.get(&tag)
    }

    pub fn string_value(&self, tag: u32) -> Option<String> {
        self.value(tag)
            .and_then(|value| value.decoded.as_ref())
            .map(value_summary)
    }

    pub fn first_string_value(&self, tags: &[u32]) -> Option<String> {
        tags.iter().find_map(|tag| self.string_value(*tag))
    }

    pub fn pq9_status(&self) -> String {
        let plausible = self.plausible_property_tag_count();
        let suspicious = self.suspicious_property_tag_count();
        let byte_swapped_selected = self.byte_swapped_selected_property_count();
        format!(
            "pq9_tag_shape=plausible:{plausible},suspicious:{suspicious},byte_swapped_selected:{byte_swapped_selected}; pq9_next_blocker={}",
            pq9_next_blocker(plausible, suspicious)
        )
    }

    pub fn pq10_status(&self) -> String {
        let traversal = self
            .values
            .values()
            .find_map(|value| value.status.strip_prefix(PQ10_TRAVERSAL_STATUS_PREFIX))
            .unwrap_or("property_context_traversal_unknown");
        format!("pq10_traversal={traversal}")
    }

    fn plausible_property_tag_count(&self) -> usize {
        self.values
            .values()
            .filter(|value| {
                value.status == "selected"
                    || value.status.starts_with("selected_byte_swapped_tag")
                    || value.status == "not_selected_plausible_mapi_tag"
            })
            .count()
    }

    fn suspicious_property_tag_count(&self) -> usize {
        self.values
            .values()
            .filter(|value| value.status.starts_with("not_selected_suspicious_key"))
            .count()
    }

    fn byte_swapped_selected_property_count(&self) -> usize {
        self.values
            .values()
            .filter(|value| value.status.starts_with("selected_byte_swapped_tag"))
            .count()
    }
}

fn pq9_next_blocker(plausible: usize, suspicious: usize) -> &'static str {
    if suspicious > plausible {
        "heap_bth_layout_traversal"
    } else if plausible > 0 {
        "selected_mapi_dictionary_expansion"
    } else {
        "property_context_signal_absent"
    }
}

fn interpret_property_tag(raw_tag: u32) -> InterpretedTag {
    if has_known_value_type(raw_tag) {
        return InterpretedTag {
            tag: raw_tag,
            is_plausible: true,
            was_byte_swapped: false,
        };
    }

    let swapped_tag = byte_swapped_tag(raw_tag);
    if property_def(swapped_tag).is_some() {
        return InterpretedTag {
            tag: swapped_tag,
            is_plausible: true,
            was_byte_swapped: true,
        };
    }

    InterpretedTag {
        tag: raw_tag,
        is_plausible: false,
        was_byte_swapped: false,
    }
}

fn selected_property_status(raw_tag: u32, interpreted: InterpretedTag) -> String {
    if interpreted.was_byte_swapped {
        format!(
            "selected_byte_swapped_tag; raw_tag=0x{raw_tag:08x}; interpreted_tag=0x{:08x}",
            interpreted.tag
        )
    } else {
        "selected".to_string()
    }
}

fn unknown_property_status(raw_tag: u32, interpreted: InterpretedTag) -> String {
    if interpreted.is_plausible {
        "not_selected_plausible_mapi_tag".to_string()
    } else {
        format!("not_selected_suspicious_key; raw_tag=0x{raw_tag:08x}")
    }
}

fn unknown_tag_sample(tags: &[u32]) -> String {
    const MAX_TAGS: usize = 16;
    if tags.is_empty() {
        return "none".to_string();
    }

    let mut sample = tags
        .iter()
        .take(MAX_TAGS)
        .map(|tag| format!("0x{tag:08x}"))
        .collect::<Vec<_>>()
        .join(",");
    if tags.len() > MAX_TAGS {
        sample.push_str(&format!(",+{}more", tags.len() - MAX_TAGS));
    }
    sample
}

#[cfg(test)]
mod tests {
    use super::PropertyContext;
    use crate::pst::bth::{BthEntry, BthHeader, BthMap};
    use crate::pst::mapi::{PR_INTERNET_CPID, PR_MESSAGE_CODEPAGE, PR_SUBJECT, PR_SUBJECT_A};

    #[test]
    fn reports_selected_unknown_and_skipped_properties() {
        let bth = BthMap {
            header: BthHeader {
                key_size: 4,
                value_size: 4,
                entry_count: 3,
                root_allocation: 0,
            },
            entries: vec![
                BthEntry {
                    key: PR_SUBJECT.to_le_bytes().to_vec(),
                    value: utf16le("Hello"),
                },
                BthEntry {
                    key: 0x9999_001fu32.to_le_bytes().to_vec(),
                    value: utf16le("Unknown"),
                },
                BthEntry {
                    key: vec![1, 2],
                    value: vec![3, 4],
                },
            ],
        };

        let report = PropertyContext::from_bth_with_report(&bth).unwrap();
        assert_eq!(report.bth_entry_count, 3);
        assert_eq!(report.parsed_property_count, 2);
        assert_eq!(report.selected_property_count, 1);
        assert_eq!(report.unknown_property_count, 1);
        assert_eq!(report.unknown_property_tags, vec![0x9999_001f]);
        assert_eq!(report.plausible_property_tag_count, 2);
        assert_eq!(report.suspicious_property_tag_count, 0);
        assert_eq!(report.byte_swapped_selected_property_count, 0);
        assert!(report
            .context
            .pq9_status()
            .contains("pq9_next_blocker=selected_mapi_dictionary_expansion"));
        assert_eq!(report.skipped_key_count, 1);
        assert_eq!(report.decode_error_count, 0);
        assert!(report.status.contains("unknown_tag_sample=0x9999001f"));
        assert!(report.status.contains("tag_shape=plausible:2,suspicious:0"));
        assert!(report.status.contains("skipped_keys=1"));
        assert_eq!(
            report.context.string_value(PR_SUBJECT).as_deref(),
            Some("Hello")
        );
    }

    #[test]
    fn selects_string8_alias_properties() {
        let bth = BthMap {
            header: BthHeader {
                key_size: 4,
                value_size: 4,
                entry_count: 1,
                root_allocation: 0,
            },
            entries: vec![BthEntry {
                key: PR_SUBJECT_A.to_le_bytes().to_vec(),
                value: b"Hi\0".to_vec(),
            }],
        };

        let report = PropertyContext::from_bth_with_report(&bth).unwrap();
        assert_eq!(report.selected_property_count, 1);
        assert_eq!(report.unknown_property_count, 0);
        assert_eq!(report.plausible_property_tag_count, 1);
        assert_eq!(
            report.context.string_value(PR_SUBJECT_A).as_deref(),
            Some("Hi")
        );
    }

    #[test]
    fn applies_explicit_fallback_charset_to_property_contexts() {
        let bth = BthMap {
            header: BthHeader {
                key_size: 4,
                value_size: 4,
                entry_count: 1,
                root_allocation: 0,
            },
            entries: vec![BthEntry {
                key: PR_SUBJECT_A.to_le_bytes().to_vec(),
                value: vec![0x80, 0],
            }],
        };

        let report =
            PropertyContext::from_bth_with_fallback_charset(&bth, Some("windows-1252")).unwrap();
        assert_eq!(
            report.context.string_value(PR_SUBJECT_A).as_deref(),
            Some("€")
        );
    }

    #[test]
    fn selects_message_codepage_before_decoding_string8_values() {
        let bth = BthMap {
            header: BthHeader {
                key_size: 4,
                value_size: 4,
                entry_count: 2,
                root_allocation: 0,
            },
            entries: vec![
                BthEntry {
                    key: PR_MESSAGE_CODEPAGE.to_le_bytes().to_vec(),
                    value: 1252i32.to_le_bytes().to_vec(),
                },
                BthEntry {
                    key: PR_SUBJECT_A.to_le_bytes().to_vec(),
                    value: vec![0x80, 0],
                },
            ],
        };

        let report = PropertyContext::from_bth_with_report(&bth).unwrap();
        assert_eq!(
            report.context.string_value(PR_SUBJECT_A).as_deref(),
            Some("€")
        );
        assert_eq!(report.charset_resolution.charset, "windows-1252");
        assert_eq!(report.charset_resolution.source, "message_codepage");
        assert!(report.status.contains("charset_metadata_selected"));
        assert_eq!(
            report.context.string_value(PR_MESSAGE_CODEPAGE).as_deref(),
            Some("1252")
        );
    }

    #[test]
    fn selects_internet_cpid_and_rejects_conflicting_evidence() {
        let bth = BthMap {
            header: BthHeader {
                key_size: 4,
                value_size: 4,
                entry_count: 2,
                root_allocation: 0,
            },
            entries: vec![
                BthEntry {
                    key: PR_INTERNET_CPID.to_le_bytes().to_vec(),
                    value: 65001i32.to_le_bytes().to_vec(),
                },
                BthEntry {
                    key: PR_SUBJECT_A.to_le_bytes().to_vec(),
                    value: vec![0xc3, 0xa9, 0],
                },
            ],
        };
        let report = PropertyContext::from_bth_with_report(&bth).unwrap();
        assert_eq!(
            report.context.string_value(PR_SUBJECT_A).as_deref(),
            Some("é")
        );
        assert_eq!(report.charset_resolution.source, "internet_cpid");

        let conflicting = BthMap {
            header: BthHeader {
                key_size: 4,
                value_size: 4,
                entry_count: 3,
                root_allocation: 0,
            },
            entries: vec![
                BthEntry {
                    key: PR_MESSAGE_CODEPAGE.to_le_bytes().to_vec(),
                    value: 1252i32.to_le_bytes().to_vec(),
                },
                BthEntry {
                    key: PR_INTERNET_CPID.to_le_bytes().to_vec(),
                    value: 65001i32.to_le_bytes().to_vec(),
                },
                BthEntry {
                    key: PR_SUBJECT_A.to_le_bytes().to_vec(),
                    value: vec![0x80, 0],
                },
            ],
        };
        let report = PropertyContext::from_bth_with_report(&conflicting).unwrap();
        assert_eq!(report.charset_resolution.charset, "iso-8859-1");
        assert_eq!(report.charset_resolution.source, "fallback");
        assert!(report.status.contains("charset_metadata_conflict"));
        assert_eq!(
            report.context.string_value(PR_SUBJECT_A).as_deref(),
            Some("\u{80}")
        );
    }

    #[test]
    fn rejects_invalid_code_page_evidence_and_keeps_explicit_override_authoritative() {
        let invalid = BthMap {
            header: BthHeader {
                key_size: 4,
                value_size: 4,
                entry_count: 2,
                root_allocation: 0,
            },
            entries: vec![
                BthEntry {
                    key: PR_MESSAGE_CODEPAGE.to_le_bytes().to_vec(),
                    value: 9500i32.to_le_bytes().to_vec(),
                },
                BthEntry {
                    key: PR_SUBJECT_A.to_le_bytes().to_vec(),
                    value: vec![0x80, 0],
                },
            ],
        };
        let report =
            PropertyContext::from_bth_with_fallback_charset(&invalid, Some("windows-1252"))
                .unwrap();
        assert_eq!(report.charset_resolution.source, "explicit_override");
        assert_eq!(
            report.context.string_value(PR_SUBJECT_A).as_deref(),
            Some("€")
        );
        assert!(report.status.contains("charset_override_authoritative"));
    }

    #[test]
    fn decodes_common_non_western_code_pages_and_reports_bad_sequences() {
        let cases = [
            (932i32, &[0x93, 0xfa, 0x96, 0x7b, 0x8c, 0xea][..], "日本語"),
            (936i32, &[0xd6, 0xd0, 0xce, 0xc4][..], "中文"),
            (949i32, &[0xc7, 0xd1, 0xb1, 0xdb][..], "한글"),
            (950i32, &[0xa4, 0xa4, 0xa4, 0xe5][..], "中文"),
        ];

        for (code_page, raw, expected) in cases {
            let bth = BthMap {
                header: BthHeader {
                    key_size: 4,
                    value_size: 4,
                    entry_count: 2,
                    root_allocation: 0,
                },
                entries: vec![
                    BthEntry {
                        key: PR_MESSAGE_CODEPAGE.to_le_bytes().to_vec(),
                        value: code_page.to_le_bytes().to_vec(),
                    },
                    BthEntry {
                        key: PR_SUBJECT_A.to_le_bytes().to_vec(),
                        value: raw.to_vec(),
                    },
                ],
            };
            let report = PropertyContext::from_bth_with_report(&bth).unwrap();
            assert_eq!(
                report.context.string_value(PR_SUBJECT_A).as_deref(),
                Some(expected),
                "code_page={code_page}"
            );
            assert_eq!(report.charset_resolution.source, "message_codepage");
            assert_eq!(report.charset_conversion_error_count, 0);
        }

        let malformed = BthMap {
            header: BthHeader {
                key_size: 4,
                value_size: 4,
                entry_count: 2,
                root_allocation: 0,
            },
            entries: vec![
                BthEntry {
                    key: PR_MESSAGE_CODEPAGE.to_le_bytes().to_vec(),
                    value: 932i32.to_le_bytes().to_vec(),
                },
                BthEntry {
                    key: PR_SUBJECT_A.to_le_bytes().to_vec(),
                    value: vec![0x82, 0x20],
                },
            ],
        };
        let report = PropertyContext::from_bth_with_report(&malformed).unwrap();
        assert_eq!(report.charset_conversion_error_count, 1);
        assert!(report.status.contains("charset_conversion_errors=1"));
        assert_eq!(report.context.values[&PR_SUBJECT_A].raw, vec![0x82, 0x20]);
        assert!(report
            .context
            .string_value(PR_SUBJECT_A)
            .unwrap()
            .contains('\u{fffd}'));
    }

    #[test]
    fn diagnoses_suspicious_property_keys() {
        let bth = BthMap {
            header: BthHeader {
                key_size: 4,
                value_size: 4,
                entry_count: 1,
                root_allocation: 0,
            },
            entries: vec![BthEntry {
                key: 0x001f_0037u32.to_le_bytes().to_vec(),
                value: utf16le("Wrong shape"),
            }],
        };

        let report = PropertyContext::from_bth_with_report(&bth).unwrap();
        assert_eq!(report.selected_property_count, 0);
        assert_eq!(report.unknown_property_count, 1);
        assert_eq!(report.plausible_property_tag_count, 0);
        assert_eq!(report.suspicious_property_tag_count, 1);
        assert!(report.status.contains("suspicious:1"));
        assert!(report
            .context
            .pq9_status()
            .contains("pq9_next_blocker=heap_bth_layout_traversal"));
        let value = report.context.values.values().next().unwrap();
        assert!(value.status.contains("not_selected_suspicious_key"));
    }

    #[test]
    fn interprets_byte_swapped_selected_tags_when_direct_shape_is_invalid() {
        let bth = BthMap {
            header: BthHeader {
                key_size: 4,
                value_size: 4,
                entry_count: 1,
                root_allocation: 0,
            },
            entries: vec![BthEntry {
                key: PR_SUBJECT.swap_bytes().to_le_bytes().to_vec(),
                value: utf16le("Swapped subject"),
            }],
        };

        let report = PropertyContext::from_bth_with_report(&bth).unwrap();
        assert_eq!(report.selected_property_count, 1);
        assert_eq!(report.unknown_property_count, 0);
        assert_eq!(report.plausible_property_tag_count, 1);
        assert_eq!(report.suspicious_property_tag_count, 0);
        assert_eq!(report.byte_swapped_selected_property_count, 1);
        assert!(report
            .context
            .pq9_status()
            .contains("byte_swapped_selected:1"));
        assert_eq!(
            report.context.string_value(PR_SUBJECT).as_deref(),
            Some("Swapped subject")
        );
        assert!(report.status.contains("byte_swapped_selected:1"));
    }

    #[test]
    fn carries_pq10_traversal_status_without_affecting_pq9_counts() {
        let context =
            PropertyContext::default().with_pq10_traversal_status("heap_bth_property_context");

        assert_eq!(
            context.pq10_status(),
            "pq10_traversal=heap_bth_property_context"
        );
        assert!(context.pq9_status().contains("plausible:0,suspicious:0"));
    }

    fn utf16le(value: &str) -> Vec<u8> {
        let mut bytes = Vec::new();
        for unit in value.encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes
    }
}
