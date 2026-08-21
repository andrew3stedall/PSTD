use crate::output::metadata::{ItemKind, ItemVisibility};

/// The readpst `-t[eajc]` family expressed as an immutable PSTD policy.
///
/// The CLI translation belongs to RP-M3-03. Keeping the policy here lets the
/// canonical item stream record the same decisions before an adapter filters
/// anything, so filtered and unsupported content remains auditable.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemTypeFilter {
    #[default]
    All,
    Email,
    Appointment,
    Journal,
    Contact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ItemRoutingPolicy {
    pub include_deleted: bool,
    pub include_associated: bool,
    pub item_type_filter: ItemTypeFilter,
}

impl Default for ItemRoutingPolicy {
    fn default() -> Self {
        Self {
            include_deleted: false,
            include_associated: false,
            item_type_filter: ItemTypeFilter::All,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemClassification {
    pub kind: Option<ItemKind>,
    pub confidence: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingDecision {
    pub selected: bool,
    pub status: &'static str,
}

/// Classify the message/container class used by libpst's `pst_process`.
/// Unknown and absent classes remain distinguishable; neither is coerced to a
/// successful ordinary note.
pub fn classify_message_class(message_class: Option<&str>) -> ItemClassification {
    let Some(message_class) = message_class
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return ItemClassification {
            kind: None,
            confidence: "message_class_missing",
        };
    };

    let normalized = message_class.to_ascii_lowercase();
    let kind = if normalized == "ipm.note" || normalized.starts_with("ipm.note.") {
        ItemKind::Note
    } else if normalized.starts_with("ipm.schedule.meeting.") {
        ItemKind::Schedule
    } else if normalized == "ipm.appointment" || normalized.starts_with("ipm.appointment.") {
        ItemKind::Appointment
    } else if normalized == "ipm.contact" || normalized.starts_with("ipm.contact.") {
        ItemKind::Contact
    } else if normalized == "ipm.journal"
        || normalized.starts_with("ipm.journal.")
        || normalized == "ipm.activity"
        || normalized.starts_with("ipm.activity.")
    {
        ItemKind::Journal
    } else if normalized == "ipm.stickynote" || normalized.starts_with("ipm.stickynote.") {
        ItemKind::StickyNote
    } else if normalized == "ipm.task" || normalized.starts_with("ipm.task.") {
        ItemKind::Task
    } else if normalized.starts_with("report.") || normalized.starts_with("report/") {
        ItemKind::Report
    } else {
        return ItemClassification {
            kind: Some(ItemKind::Other),
            confidence: "message_class_unknown",
        };
    };

    ItemClassification {
        kind: Some(kind),
        confidence: "message_class_normalized",
    }
}

pub fn route_item(
    visibility: ItemVisibility,
    classification: &ItemClassification,
    policy: ItemRoutingPolicy,
) -> RoutingDecision {
    if visibility == ItemVisibility::Deleted && !policy.include_deleted {
        return RoutingDecision {
            selected: false,
            status: "filtered_deleted",
        };
    }
    if visibility == ItemVisibility::Associated && !policy.include_associated {
        return RoutingDecision {
            selected: false,
            status: "filtered_associated",
        };
    }
    if visibility == ItemVisibility::Unknown {
        return RoutingDecision {
            selected: false,
            status: "unavailable_unknown_visibility",
        };
    }

    let Some(kind) = classification.kind else {
        return RoutingDecision {
            selected: false,
            status: "unavailable_missing_item_class",
        };
    };

    if matches!(kind, ItemKind::StickyNote | ItemKind::Task) {
        return RoutingDecision {
            selected: false,
            status: "skipped_unsupported_by_readpst",
        };
    }
    if kind == ItemKind::Other {
        return RoutingDecision {
            selected: false,
            status: "skipped_unknown_item_class",
        };
    }

    let filter_matches = match policy.item_type_filter {
        ItemTypeFilter::All => true,
        ItemTypeFilter::Email => {
            matches!(kind, ItemKind::Note | ItemKind::Schedule | ItemKind::Report)
        }
        ItemTypeFilter::Appointment => kind == ItemKind::Appointment,
        ItemTypeFilter::Journal => kind == ItemKind::Journal,
        ItemTypeFilter::Contact => kind == ItemKind::Contact,
    };
    if !filter_matches {
        return RoutingDecision {
            selected: false,
            status: "filtered_item_type",
        };
    }

    RoutingDecision {
        selected: true,
        status: match kind {
            ItemKind::Note => "routed_email",
            ItemKind::Schedule => "routed_schedule_email",
            ItemKind::Appointment => "routed_appointment",
            ItemKind::Contact => "routed_contact",
            ItemKind::Journal => "routed_journal",
            ItemKind::Report => "routed_report",
            ItemKind::StickyNote | ItemKind::Task | ItemKind::Other | ItemKind::Store => {
                "skipped_unsupported_by_readpst"
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{classify_message_class, route_item, ItemRoutingPolicy, ItemTypeFilter};
    use crate::output::metadata::{ItemKind, ItemVisibility};

    #[test]
    fn classifies_readpst_item_families_without_guessing_unknowns() {
        let cases = [
            (
                Some("IPM.Note"),
                Some(ItemKind::Note),
                "message_class_normalized",
            ),
            (
                Some("IPM.Schedule.Meeting.Request"),
                Some(ItemKind::Schedule),
                "message_class_normalized",
            ),
            (
                Some("IPM.Appointment"),
                Some(ItemKind::Appointment),
                "message_class_normalized",
            ),
            (
                Some("IPM.Contact"),
                Some(ItemKind::Contact),
                "message_class_normalized",
            ),
            (
                Some("IPM.Activity"),
                Some(ItemKind::Journal),
                "message_class_normalized",
            ),
            (
                Some("IPM.Task"),
                Some(ItemKind::Task),
                "message_class_normalized",
            ),
            (
                Some("REPORT.IPM.Note.NDR"),
                Some(ItemKind::Report),
                "message_class_normalized",
            ),
            (
                Some("IPM.FutureType"),
                Some(ItemKind::Other),
                "message_class_unknown",
            ),
            (None, None, "message_class_missing"),
        ];

        for (raw, expected_kind, expected_confidence) in cases {
            let classification = classify_message_class(raw);
            assert_eq!(classification.kind, expected_kind, "class={raw:?}");
            assert_eq!(
                classification.confidence, expected_confidence,
                "class={raw:?}"
            );
        }
    }

    #[test]
    fn policy_preserves_explicit_filter_and_unsupported_statuses() {
        let all = ItemRoutingPolicy::default();
        let contact = classify_message_class(Some("IPM.Contact"));
        assert_eq!(
            route_item(ItemVisibility::Visible, &contact, all).status,
            "routed_contact"
        );

        let associated = classify_message_class(Some("IPM.Note"));
        assert_eq!(
            route_item(ItemVisibility::Associated, &associated, all).status,
            "filtered_associated"
        );
        let include_associated = ItemRoutingPolicy {
            include_associated: true,
            ..all
        };
        assert_eq!(
            route_item(ItemVisibility::Associated, &associated, include_associated).status,
            "routed_email"
        );

        let task = classify_message_class(Some("IPM.Task"));
        assert_eq!(
            route_item(ItemVisibility::Visible, &task, all).status,
            "skipped_unsupported_by_readpst"
        );
        assert_eq!(
            route_item(
                ItemVisibility::Visible,
                &contact,
                ItemRoutingPolicy {
                    item_type_filter: ItemTypeFilter::Email,
                    ..all
                }
            )
            .status,
            "filtered_item_type"
        );
    }
}
