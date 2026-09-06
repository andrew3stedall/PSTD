//! Shared borrowed joins for attachment output. Never choose an arbitrary duplicate payload.
use std::collections::{BTreeMap, BTreeSet};

use crate::output::metadata::AttachmentRecord;
use crate::pst::attachments::AttachmentPayload;

pub(crate) struct AttachmentIndex<'a> {
    pub records: Vec<&'a AttachmentRecord>,
    payloads: BTreeMap<&'a str, Option<&'a AttachmentPayload>>,
}

impl<'a> AttachmentIndex<'a> {
    pub fn new(records: &'a [AttachmentRecord], payloads: &'a [AttachmentPayload]) -> Self {
        let mut records = records.iter().collect::<Vec<_>>();
        let mut keys = records
            .iter()
            .map(|record| record.attachment_key.as_str())
            .collect::<BTreeSet<_>>();
        let mut indexed_payloads = BTreeMap::new();
        for payload in payloads {
            let key = payload.record.attachment_key.as_str();
            if keys.insert(key) {
                records.push(&payload.record);
            }
            indexed_payloads
                .entry(key)
                .and_modify(|value| *value = None)
                .or_insert(Some(payload));
        }
        records.sort_by(|left, right| {
            (&left.message_key, left.ordinal, &left.attachment_key).cmp(&(
                &right.message_key,
                right.ordinal,
                &right.attachment_key,
            ))
        });
        Self {
            records,
            payloads: indexed_payloads,
        }
    }

    pub fn payload(&self, key: &str) -> Result<Option<&'a AttachmentPayload>, &'static str> {
        match self.payloads.get(key) {
            Some(Some(payload)) => Ok(Some(*payload)),
            Some(None) => Err("duplicate attachment ID in payloads"),
            None => Ok(None),
        }
    }
}
