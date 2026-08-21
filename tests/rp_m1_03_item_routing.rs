use pstd::output::metadata::{ItemKind, ItemVisibility};
use pstd::pst::item_routing::{
    classify_message_class, route_item, ItemRoutingPolicy, ItemTypeFilter,
};

#[test]
fn mixed_folder_fixture_routes_every_observable_class_explicitly() {
    let policy = ItemRoutingPolicy::default();
    let cases = [
        ("IPM.Note", ItemKind::Note, "routed_email"),
        (
            "IPM.Schedule.Meeting.Request",
            ItemKind::Schedule,
            "routed_schedule_email",
        ),
        ("IPM.Appointment", ItemKind::Appointment, "routed_appointment"),
        ("IPM.Contact", ItemKind::Contact, "routed_contact"),
        ("IPM.Activity", ItemKind::Journal, "routed_journal"),
        ("REPORT.IPM.Note.NDR", ItemKind::Report, "routed_report"),
        (
            "IPM.Task",
            ItemKind::Task,
            "skipped_unsupported_by_readpst",
        ),
        (
            "IPM.StickyNote",
            ItemKind::StickyNote,
            "skipped_unsupported_by_readpst",
        ),
        (
            "IPM.FutureType",
            ItemKind::Other,
            "skipped_unknown_item_class",
        ),
    ];

    let statuses: Vec<_> = cases
        .iter()
        .map(|(class, expected_kind, expected_status)| {
            let classification = classify_message_class(Some(class));
            assert_eq!(classification.kind, Some(*expected_kind), "class={class}");
            let decision = route_item(ItemVisibility::Visible, &classification, policy);
            assert_eq!(decision.status, *expected_status, "class={class}");
            (class, classification.confidence, decision.selected, decision.status)
        })
        .collect();

    let first = serde_json::to_vec(&statuses).expect("serialize fixture evidence");
    let second = serde_json::to_vec(&statuses).expect("serialize repeated fixture evidence");
    assert_eq!(first, second, "routing evidence must be deterministic");
}

#[test]
fn visibility_and_type_filters_are_explicit_and_non_destructive() {
    let note = classify_message_class(Some("IPM.Note"));
    let defaults = ItemRoutingPolicy::default();
    assert_eq!(
        route_item(ItemVisibility::Associated, &note, defaults).status,
        "filtered_associated"
    );
    assert_eq!(
        route_item(ItemVisibility::Deleted, &note, defaults).status,
        "filtered_deleted"
    );

    let include_deleted = ItemRoutingPolicy {
        include_deleted: true,
        ..defaults
    };
    assert_eq!(
        route_item(ItemVisibility::Deleted, &note, include_deleted).status,
        "routed_email"
    );

    let contact = classify_message_class(Some("IPM.Contact"));
    let email_only = ItemRoutingPolicy {
        item_type_filter: ItemTypeFilter::Email,
        ..defaults
    };
    assert_eq!(
        route_item(ItemVisibility::Visible, &contact, email_only).status,
        "filtered_item_type"
    );
    assert_eq!(
        route_item(ItemVisibility::Unknown, &note, defaults).status,
        "unavailable_unknown_visibility"
    );
    let missing = classify_message_class(None);
    assert_eq!(
        route_item(ItemVisibility::Visible, &missing, defaults).status,
        "unavailable_missing_item_class"
    );
}
