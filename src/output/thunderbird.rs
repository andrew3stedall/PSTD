use sha2::{Digest, Sha256};

use crate::output::calendar::serialize_icalendar;
use crate::output::contact::serialize_vcards;
use crate::output::mailbox::{MailboxArtifact, MailboxArtifactSummary};
use crate::output::metadata::{CalendarRecord, ContactRecord, NonMailRecord};
use crate::output::non_mail::serialize_vjournals;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThunderbirdTypedStatus {
    pub profile: String,
    pub status: String,
    pub contact_record_count: usize,
    pub calendar_record_count: usize,
    pub journal_record_count: usize,
    pub non_mail_record_count: usize,
    pub unsupported_non_mail_count: usize,
    pub artifact_count: usize,
    pub artifacts: Vec<MailboxArtifactSummary>,
    pub decisions: Vec<String>,
}

pub fn render_typed_outputs(
    contacts: &[ContactRecord],
    calendars: &[CalendarRecord],
    non_mail: &[NonMailRecord],
) -> (ThunderbirdTypedStatus, Vec<MailboxArtifact>) {
    let journal_count = non_mail
        .iter()
        .filter(|record| record.item_kind == "journal")
        .count();
    let unsupported_non_mail_count = non_mail.len().saturating_sub(journal_count);
    let mut artifacts = Vec::new();

    if !contacts.is_empty() {
        artifacts.push(artifact(
            "outputs/thunderbird/typed/contacts.vcf",
            "thunderbird_typed_contact",
            serialize_vcards(contacts).into_bytes(),
            "typed_contact_projection_emitted",
        ));
    }
    if !calendars.is_empty() {
        artifacts.push(artifact(
            "outputs/thunderbird/typed/appointments.ics",
            "thunderbird_typed_calendar",
            serialize_icalendar(calendars).into_bytes(),
            "typed_calendar_projection_emitted",
        ));
    }
    if journal_count > 0 {
        artifacts.push(artifact(
            "outputs/thunderbird/typed/journals.vjournal",
            "thunderbird_typed_journal",
            serialize_vjournals(non_mail).into_bytes(),
            "typed_journal_projection_emitted",
        ));
    }
    if !non_mail.is_empty() {
        let mut bytes = Vec::new();
        for record in non_mail {
            if let Ok(mut line) = serde_json::to_vec(record) {
                line.push(b'\n');
                bytes.extend_from_slice(&line);
            }
        }
        artifacts.push(artifact(
            "outputs/thunderbird/typed/non-mail.jsonl",
            "thunderbird_typed_non_mail",
            bytes,
            "typed_non_mail_evidence_emitted",
        ));
    }

    let status = if artifacts.is_empty() {
        "typed_outputs_unavailable"
    } else if unsupported_non_mail_count > 0 {
        "typed_outputs_partial"
    } else {
        "typed_outputs_available"
    };
    let decisions = vec![
        format!(
            "contacts:{}; calendars:{}; journals:{}; non_mail:{}",
            contacts.len(),
            calendars.len(),
            journal_count,
            non_mail.len()
        ),
        format!(
            "unsupported_non_mail_preserved:{}",
            unsupported_non_mail_count
        ),
        "ordinary_email_serialization_not_used_for_typed_outputs".to_string(),
    ];
    let summaries = artifacts
        .iter()
        .map(|artifact| artifact.summary.clone())
        .collect::<Vec<_>>();
    (
        ThunderbirdTypedStatus {
            profile: "thunderbird_typed".to_string(),
            status: status.to_string(),
            contact_record_count: contacts.len(),
            calendar_record_count: calendars.len(),
            journal_record_count: journal_count,
            non_mail_record_count: non_mail.len(),
            unsupported_non_mail_count,
            artifact_count: artifacts.len(),
            artifacts: summaries,
            decisions,
        },
        artifacts,
    )
}

fn artifact(path: &str, output_kind: &str, bytes: Vec<u8>, status: &str) -> MailboxArtifact {
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    MailboxArtifact {
        summary: MailboxArtifactSummary {
            path: path.to_string(),
            message_key: None,
            folder_path: "typed".to_string(),
            output_kind: output_kind.to_string(),
            sha256: hex::encode(hasher.finalize()),
            size_bytes: bytes.len() as u64,
            status: status.to_string(),
        },
        bytes,
    }
}

#[cfg(test)]
mod tests {
    use super::render_typed_outputs;

    #[test]
    fn empty_typed_source_is_explicitly_unavailable() {
        let (status, artifacts) = render_typed_outputs(&[], &[], &[]);
        assert!(artifacts.is_empty());
        assert_eq!(status.status, "typed_outputs_unavailable");
    }
}
