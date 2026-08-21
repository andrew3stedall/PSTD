use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::output::ids;
use crate::output::metadata::{AttachmentRecord, EmbeddedGraphRecord, MessageRecord};
use crate::pst::attachments::AttachmentPayload;
use crate::pst::messages::BodyPayload;

pub const MAX_EMBEDDED_DEPTH: u64 = 8;
pub const MAX_EMBEDDED_NODES: usize = 4096;
pub const MAX_EMBEDDED_BYTES: u64 = 64 * 1024 * 1024;

pub fn build_embedded_graph(
    messages: &[MessageRecord],
    attachments: &[AttachmentRecord],
    body_payloads: &[BodyPayload],
    attachment_payloads: &[AttachmentPayload],
) -> Vec<EmbeddedGraphRecord> {
    let message_keys = messages
        .iter()
        .map(|message| message.message_key.clone())
        .collect::<BTreeSet<_>>();
    let message_counts = messages
        .iter()
        .fold(BTreeMap::new(), |mut counts, message| {
            *counts.entry(message.message_key.clone()).or_insert(0usize) += 1;
            counts
        });
    let message_by_key = messages
        .iter()
        .map(|message| (message.message_key.as_str(), message))
        .collect::<BTreeMap<_, _>>();
    let mut ordered_attachments = attachments
        .iter()
        .filter(|attachment| attachment.attachment_method == Some(5))
        .collect::<Vec<_>>();
    ordered_attachments.sort_by_key(|attachment| {
        (
            attachment.message_key.clone(),
            attachment.rendering_position.unwrap_or(attachment.ordinal),
            attachment.attachment_key.clone(),
        )
    });

    let mut adjacency: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for attachment in &ordered_attachments {
        if let Some(child) = attachment.embedded_message_key.as_deref() {
            adjacency
                .entry(attachment.message_key.clone())
                .or_default()
                .push(child.to_string());
        }
    }
    for children in adjacency.values_mut() {
        children.sort();
        children.dedup();
    }
    let depths = message_depths(&message_keys, &adjacency);
    let body_bytes = body_payloads
        .iter()
        .fold(BTreeMap::new(), |mut bytes, payload| {
            *bytes.entry(payload.record.message_key.clone()).or_insert(0) +=
                payload.bytes.len() as u64;
            bytes
        });
    let attachment_bytes =
        attachment_payloads
            .iter()
            .fold(BTreeMap::new(), |mut bytes, payload| {
                *bytes.entry(payload.record.message_key.clone()).or_insert(0) +=
                    payload.bytes.len() as u64;
                bytes
            });
    let duplicate_keys = duplicate_attachment_keys(&ordered_attachments);

    ordered_attachments
        .into_iter()
        .map(|attachment| {
            let child_key = attachment.embedded_message_key.clone();
            let child_exists = child_key
                .as_deref()
                .is_some_and(|key| message_keys.contains(key));
            let child_is_email = child_key
                .as_deref()
                .and_then(|key| message_by_key.get(key))
                .is_some_and(|message| {
                    matches!(
                        message.item_type.as_str(),
                        "message_metadata" | "embedded_message_metadata"
                    )
                });
            let cycle = child_key.as_deref().is_some_and(|child| {
                reaches(
                    child,
                    &attachment.message_key,
                    &adjacency,
                    &mut BTreeSet::new(),
                )
            });
            let depth = child_key
                .as_deref()
                .and_then(|key| depths.get(key).copied())
                .unwrap_or(0);
            let raw_bytes_observed = child_key
                .as_deref()
                .map(|key| {
                    body_bytes.get(key).copied().unwrap_or(0)
                        + attachment_bytes.get(key).copied().unwrap_or(0)
                })
                .unwrap_or(0);
            let budget_status = budget_status(message_keys.len(), raw_bytes_observed, depth);
            let cycle_status = if cycle { "cycle_detected" } else { "acyclic" };
            let resolution_status = resolution_status(
                child_key.is_some(),
                child_exists,
                child_is_email,
                child_key
                    .as_deref()
                    .and_then(|key| message_counts.get(key))
                    .is_some_and(|count| *count > 1),
                duplicate_keys.contains(&attachment.attachment_key),
            );
            let authoritative = child_exists
                && !cycle
                && budget_status == "within_limits"
                && resolution_status == "child_message_resolved";
            let status = if authoritative {
                "embedded_child_resolved"
            } else if cycle {
                "embedded_child_cycle_rejected"
            } else if budget_status != "within_limits" {
                "embedded_child_budget_rejected"
            } else {
                "embedded_child_unavailable"
            };
            EmbeddedGraphRecord {
                edge_key: ids::stable_id(
                    "edge",
                    &[&attachment.message_key, &attachment.attachment_key],
                ),
                parent_message_key: attachment.message_key.clone(),
                attachment_key: attachment.attachment_key.clone(),
                child_message_key: child_key.clone(),
                relation: "embedded_message".to_string(),
                ordinal: attachment.rendering_position.unwrap_or(attachment.ordinal),
                depth,
                source_ref: attachment.source_ref.clone(),
                child_evidence_key: child_key.map(|key| format!("message:{key}")),
                raw_bytes_observed,
                budget_status: budget_status.to_string(),
                cycle_status: cycle_status.to_string(),
                resolution_status: resolution_status.to_string(),
                status: status.to_string(),
                authoritative,
            }
        })
        .collect()
}

fn budget_status(message_count: usize, raw_bytes: u64, depth: u64) -> &'static str {
    if message_count > MAX_EMBEDDED_NODES {
        "node_budget_exceeded"
    } else if raw_bytes > MAX_EMBEDDED_BYTES {
        "byte_budget_exceeded"
    } else if depth > MAX_EMBEDDED_DEPTH {
        "depth_budget_exceeded"
    } else {
        "within_limits"
    }
}

fn resolution_status(
    has_child_reference: bool,
    child_exists: bool,
    child_is_email: bool,
    ambiguous_child_owner: bool,
    duplicate_attachment_owner: bool,
) -> &'static str {
    if !has_child_reference {
        "missing_child_reference"
    } else if !child_exists {
        "child_message_unavailable"
    } else if !child_is_email {
        "child_non_email_item"
    } else if ambiguous_child_owner {
        "ambiguous_child_owner"
    } else if duplicate_attachment_owner {
        "duplicate_attachment_owner"
    } else {
        "child_message_resolved"
    }
}

fn message_depths(
    message_keys: &BTreeSet<String>,
    adjacency: &BTreeMap<String, Vec<String>>,
) -> BTreeMap<String, u64> {
    let children = adjacency
        .values()
        .flat_map(|values| values.iter().cloned())
        .collect::<BTreeSet<_>>();
    let roots = message_keys
        .difference(&children)
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut depths = BTreeMap::new();
    let mut queue = VecDeque::new();
    for root in roots {
        queue.push_back((root, 0));
    }
    while let Some((message, depth)) = queue.pop_front() {
        if depths.get(&message).is_some_and(|known| *known <= depth) {
            continue;
        }
        depths.insert(message.clone(), depth);
        if depth > MAX_EMBEDDED_DEPTH {
            continue;
        }
        if let Some(children) = adjacency.get(&message) {
            for child in children {
                queue.push_back((child.clone(), depth + 1));
            }
        }
    }
    depths
}

fn reaches(
    start: &str,
    target: &str,
    adjacency: &BTreeMap<String, Vec<String>>,
    visited: &mut BTreeSet<String>,
) -> bool {
    if start == target {
        return true;
    }
    if !visited.insert(start.to_string()) {
        return false;
    }
    adjacency.get(start).is_some_and(|children| {
        children
            .iter()
            .any(|child| reaches(child, target, adjacency, visited))
    })
}

fn duplicate_attachment_keys(attachments: &[&AttachmentRecord]) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    for attachment in attachments {
        if !seen.insert(attachment.attachment_key.clone()) {
            duplicates.insert(attachment.attachment_key.clone());
        }
    }
    duplicates
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{
        budget_status, message_depths, reaches, resolution_status, MAX_EMBEDDED_BYTES,
        MAX_EMBEDDED_DEPTH, MAX_EMBEDDED_NODES,
    };

    #[test]
    fn detects_cycle_before_child_materialization() {
        let adjacency = BTreeMap::from([
            ("parent".to_string(), vec!["child".to_string()]),
            ("child".to_string(), vec!["parent".to_string()]),
        ]);
        assert!(reaches("child", "parent", &adjacency, &mut BTreeSet::new()));
    }

    #[test]
    fn computes_bounded_nested_depth_deterministically() {
        let mut adjacency = BTreeMap::new();
        for index in 0..=MAX_EMBEDDED_DEPTH + 1 {
            adjacency.insert(
                format!("m{index}"),
                (index < MAX_EMBEDDED_DEPTH + 1)
                    .then(|| vec![format!("m{}", index + 1)])
                    .unwrap_or_default(),
            );
        }
        let keys = (0..=MAX_EMBEDDED_DEPTH + 1)
            .map(|index| format!("m{index}"))
            .collect::<BTreeSet<_>>();
        let depths = message_depths(&keys, &adjacency);
        assert_eq!(depths["m0"], 0);
        assert_eq!(depths["m8"], MAX_EMBEDDED_DEPTH);
        assert_eq!(depths["m9"], MAX_EMBEDDED_DEPTH + 1);
    }

    #[test]
    fn classifies_missing_ambiguous_and_non_email_edges() {
        assert_eq!(
            resolution_status(false, false, false, false, false),
            "missing_child_reference"
        );
        assert_eq!(
            resolution_status(true, false, false, false, false),
            "child_message_unavailable"
        );
        assert_eq!(
            resolution_status(true, true, false, false, false),
            "child_non_email_item"
        );
        assert_eq!(
            resolution_status(true, true, true, true, false),
            "ambiguous_child_owner"
        );
        assert_eq!(
            resolution_status(true, true, true, false, true),
            "duplicate_attachment_owner"
        );
    }

    #[test]
    fn classifies_node_byte_and_depth_budgets() {
        assert_eq!(
            budget_status(MAX_EMBEDDED_NODES + 1, 0, 0),
            "node_budget_exceeded"
        );
        assert_eq!(
            budget_status(0, MAX_EMBEDDED_BYTES + 1, 0),
            "byte_budget_exceeded"
        );
        assert_eq!(
            budget_status(0, 0, MAX_EMBEDDED_DEPTH + 1),
            "depth_budget_exceeded"
        );
        assert_eq!(budget_status(0, 0, MAX_EMBEDDED_DEPTH), "within_limits");
    }
}
