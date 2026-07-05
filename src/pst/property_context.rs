use std::collections::HashMap;

use crate::error::PstdResult;
use crate::pst::bth::BthMap;
use crate::pst::mapi::{decode_value, property_def, value_summary, MapiValue};

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
    pub skipped_key_count: usize,
    pub decode_error_count: usize,
    pub status: String,
}

impl PropertyContext {
    pub fn from_bth(bth: &BthMap) -> PstdResult<Self> {
        Ok(Self::from_bth_with_report(bth)?.context)
    }

    pub fn from_bth_with_report(bth: &BthMap) -> PstdResult<PropertyContextParseReport> {
        let mut values = HashMap::new();
        let mut selected_property_count = 0usize;
        let mut unknown_property_count = 0usize;
        let mut unknown_property_tags = Vec::new();
        let mut skipped_key_count = 0usize;
        let mut decode_error_count = 0usize;

        for entry in &bth.entries {
            if entry.key.len() < 4 {
                skipped_key_count += 1;
                continue;
            }
            let tag = u32::from_le_bytes([entry.key[0], entry.key[1], entry.key[2], entry.key[3]]);
            let Some(def) = property_def(tag) else {
                unknown_property_count += 1;
                unknown_property_tags.push(tag);
                values.insert(
                    tag,
                    PropertyValue {
                        tag,
                        name: format!("unknown_0x{tag:08x}"),
                        raw: entry.value.clone(),
                        decoded: None,
                        status: "not_selected".to_string(),
                    },
                );
                continue;
            };
            let decoded = match decode_value(def.value_type, &entry.value) {
                Ok(value) => Some(value),
                Err(_) => {
                    decode_error_count += 1;
                    None
                }
            };
            selected_property_count += 1;
            values.insert(
                tag,
                PropertyValue {
                    tag,
                    name: def.name.to_string(),
                    raw: entry.value.clone(),
                    decoded,
                    status: "selected".to_string(),
                },
            );
        }

        let parsed_property_count = values.len();
        unknown_property_tags.sort_unstable();
        unknown_property_tags.dedup();
        let status = if decode_error_count == 0 && skipped_key_count == 0 {
            if unknown_property_count == 0 {
                "property_context_parsed".to_string()
            } else {
                format!(
                    "property_context_parsed_with_unknown_properties; unknown_properties={unknown_property_count}; unknown_tag_sample={}",
                    unknown_tag_sample(&unknown_property_tags)
                )
            }
        } else {
            format!(
                "property_context_parsed_with_issues; decode_errors={decode_error_count}; skipped_keys={skipped_key_count}; unknown_properties={unknown_property_count}; unknown_tag_sample={}",
                unknown_tag_sample(&unknown_property_tags)
            )
        };

        Ok(PropertyContextParseReport {
            context: Self { values },
            bth_entry_count: bth.entries.len(),
            parsed_property_count,
            selected_property_count,
            unknown_property_count,
            unknown_property_tags,
            skipped_key_count,
            decode_error_count,
            status,
        })
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
    use crate::pst::mapi::{PR_SUBJECT, PR_SUBJECT_A};

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
        assert_eq!(report.skipped_key_count, 1);
        assert_eq!(report.decode_error_count, 0);
        assert!(report.status.contains("unknown_tag_sample=0x9999001f"));
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
        assert_eq!(
            report.context.string_value(PR_SUBJECT_A).as_deref(),
            Some("Hi")
        );
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
