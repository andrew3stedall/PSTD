pub fn normalize_subject(subject: &str) -> String {
    let mut value = subject.trim();

    loop {
        let trimmed = value.trim_start();
        let Some((prefix, remainder)) = trimmed.split_once(':') else {
            value = trimmed;
            break;
        };

        if is_thread_prefix(prefix) {
            value = remainder.trim_start();
        } else {
            value = trimmed;
            break;
        }
    }

    value.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

fn is_thread_prefix(prefix: &str) -> bool {
    let normalized = prefix.trim().to_ascii_lowercase();
    matches!(normalized.as_str(), "re" | "fw" | "fwd")
}

pub fn split_internet_references(value: &str) -> Vec<String> {
    value
        .split_whitespace()
        .map(|item| item.trim().trim_matches(',').trim_matches(';'))
        .filter(|item| item.starts_with('<') && item.ends_with('>') && item.len() > 2)
        .map(ToString::to_string)
        .collect()
}

pub fn threading_status(
    internet_message_id: Option<&str>,
    in_reply_to_id: Option<&str>,
    references: &[String],
    conversation_index: Option<&str>,
) -> String {
    if internet_message_id.is_some()
        || in_reply_to_id.is_some()
        || !references.is_empty()
        || conversation_index.is_some()
    {
        "threading_metadata_available".to_string()
    } else {
        "threading_metadata_absent".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{normalize_subject, split_internet_references, threading_status};

    #[test]
    fn normalizes_reply_and_forward_prefixes() {
        assert_eq!(normalize_subject(" Re:  FW: Quarterly   Update "), "quarterly update");
        assert_eq!(normalize_subject("Fwd: Re: Test"), "test");
    }

    #[test]
    fn keeps_non_thread_prefix_subjects() {
        assert_eq!(normalize_subject("Project: Alpha"), "project: alpha");
    }

    #[test]
    fn splits_internet_references() {
        let references = split_internet_references("<a@example> <b@example>; <c@example>, ignored");
        assert_eq!(references, vec!["<a@example>", "<b@example>", "<c@example>"]);
    }

    #[test]
    fn reports_threading_status() {
        assert_eq!(
            threading_status(Some("<id@example>"), None, &[], None),
            "threading_metadata_available"
        );
        assert_eq!(
            threading_status(None, None, &[], None),
            "threading_metadata_absent"
        );
    }
}
