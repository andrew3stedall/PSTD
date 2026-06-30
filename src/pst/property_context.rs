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

impl PropertyContext {
    pub fn from_bth(bth: &BthMap) -> PstdResult<Self> {
        let mut values = HashMap::new();
        for entry in &bth.entries {
            if entry.key.len() < 4 {
                continue;
            }
            let tag = u32::from_le_bytes([entry.key[0], entry.key[1], entry.key[2], entry.key[3]]);
            let Some(def) = property_def(tag) else {
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
            let decoded = decode_value(def.value_type, &entry.value).ok();
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
        Ok(Self { values })
    }

    pub fn value(&self, tag: u32) -> Option<&PropertyValue> {
        self.values.get(&tag)
    }

    pub fn string_value(&self, tag: u32) -> Option<String> {
        self.value(tag)
            .and_then(|value| value.decoded.as_ref())
            .map(value_summary)
    }
}
