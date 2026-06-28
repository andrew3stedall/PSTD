use sha2::{Digest, Sha256};

pub fn stable_id(prefix: &str, parts: &[impl AsRef<str>]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.as_ref().as_bytes());
        hasher.update([0x1f]);
    }
    let digest = hasher.finalize();
    let short = &hex::encode(digest)[..16];
    format!("{prefix}_{short}")
}

pub fn run_id(input: &str) -> String {
    stable_id("run", &[input, &chrono::Utc::now().timestamp_millis().to_string()])
}

pub fn pst_id(source: &str) -> String {
    stable_id("pst", &[source])
}

pub fn folder_key(pst_id: &str, folder_identity: &str) -> String {
    stable_id("folder", &[pst_id, folder_identity])
}

pub fn message_key(pst_id: &str, message_identity: &str) -> String {
    stable_id("msg", &[pst_id, message_identity])
}

pub fn body_key(message_key: &str, body_type: &str) -> String {
    stable_id("body", &[message_key, body_type])
}

pub fn attachment_key(message_key: &str, ordinal: usize) -> String {
    stable_id("att", &[message_key, &ordinal.to_string()])
}

pub fn recipient_key(message_key: &str, recipient_type: &str, ordinal: usize) -> String {
    stable_id("rcpt", &[message_key, recipient_type, &ordinal.to_string()])
}
