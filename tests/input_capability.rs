use std::fs;

use pstd::pst::capability::{InputCapability, InputCapabilityStatus, InputFamily};
use pstd::pst::header::{PstHeader, PST_MAGIC};
use pstd::pst::limits::InputLimits;
use pstd::pst::reader::PstByteReader;

fn synthetic_header(index_type: u8, crypt_method: Option<u8>, roots: bool) -> Vec<u8> {
    let mut header = vec![0u8; 514];
    header[0..4].copy_from_slice(&PST_MAGIC);
    header[8..10].copy_from_slice(b"SM");
    header[10] = index_type;
    if roots {
        header[48..56].copy_from_slice(&1024u64.to_le_bytes());
        header[56..64].copy_from_slice(&2048u64.to_le_bytes());
    }
    if let Some(method) = crypt_method {
        header[513] = method;
    }
    header
}

#[test]
fn unicode_capability_is_ready_only_with_safe_roots_and_no_crypt() {
    let bytes = synthetic_header(0x17, Some(0), true);
    let header = PstHeader::parse_bytes(&bytes, 4096).expect("header");
    let capability = InputCapability::from_header("unicode.pst", &header, InputLimits::default());

    assert_eq!(capability.family, InputFamily::UnicodePst);
    assert_eq!(capability.status, InputCapabilityStatus::Ready);
    assert!(capability.allows_extraction);
    assert_eq!(capability.default_charset, "iso-8859-1");
    assert_eq!(capability.index_type, Some(0x17));
}

#[test]
fn unsupported_families_and_crypt_are_not_empty_success() {
    for (index_type, expected_family) in [
        (0x0e, InputFamily::AnsiPst),
        (0x24, InputFamily::Ost2013),
        (0x7e, InputFamily::Unknown),
    ] {
        let bytes = synthetic_header(index_type, Some(0), true);
        let header = PstHeader::parse_bytes(&bytes, 4096).expect("header");
        let capability =
            InputCapability::from_header("fixture.pst", &header, InputLimits::default());
        assert_eq!(capability.family, expected_family);
        assert_eq!(capability.status, InputCapabilityStatus::Unsupported);
        assert!(!capability.allows_extraction);
    }

    let supported_crypt = synthetic_header(0x17, Some(1), true);
    let supported_header = PstHeader::parse_bytes(&supported_crypt, 4096).expect("header");
    let supported_capability =
        InputCapability::from_header("permute-encrypted.pst", &supported_header, InputLimits::default());
    assert_eq!(supported_capability.status, InputCapabilityStatus::Ready);
    assert!(supported_capability.allows_extraction);

    for crypt_method in [2, 7] {
        let bytes = synthetic_header(0x17, Some(crypt_method), true);
        let header = PstHeader::parse_bytes(&bytes, 4096).expect("header");
        let capability =
            InputCapability::from_header("encrypted.pst", &header, InputLimits::default());
        assert_eq!(capability.status, InputCapabilityStatus::Unsupported);
        let expected_index_status = if crypt_method == 2 {
            "unsupported_strong_crypt_method".to_string()
        } else {
            format!("unsupported_crypt_method:{crypt_method}")
        };
        assert_eq!(capability.index_status, expected_index_status);
        assert!(!capability.allows_extraction);
    }
}

#[test]
fn missing_roots_and_short_headers_are_explicit() {
    let bytes = synthetic_header(0x17, Some(0), false);
    let header = PstHeader::parse_bytes(&bytes, 514).expect("header");
    let capability = InputCapability::from_header("partial.pst", &header, InputLimits::default());
    assert_eq!(capability.status, InputCapabilityStatus::Partial);
    assert!(!capability.allows_extraction);

    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("short.pst");
    fs::write(&path, [0u8; 8]).expect("short fixture");
    let probed = InputCapability::probe(&path, InputLimits::default());
    assert_eq!(probed.status, InputCapabilityStatus::Malformed);
    assert!(!probed.allows_extraction);
    assert_eq!(probed.root_condition, "not_decoded");
}

#[test]
fn reader_enforces_file_and_single_read_budgets() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("input.bin");
    fs::write(&path, [0u8; 8]).expect("input");

    let file_limited = InputLimits {
        max_file_bytes: 4,
        ..InputLimits::default()
    };
    let error = PstByteReader::open_with_limits(&path, &file_limited)
        .expect_err("file budget must reject input");
    assert!(error.to_string().contains("max_file_bytes"));

    let read_limited = InputLimits {
        max_single_read_bytes: 2,
        ..InputLimits::default()
    };
    let reader = PstByteReader::open_with_limits(&path, &read_limited).expect("reader");
    assert!(reader.read_at(0, 3).is_err());
    assert_eq!(reader.read_prefix(8).expect("bounded prefix").len(), 2);
}

#[test]
fn capability_projection_is_repeatable() {
    let bytes = synthetic_header(0x17, Some(0), true);
    let header_a = PstHeader::parse_bytes(&bytes, 4096).expect("header");
    let header_b = PstHeader::parse_bytes(&bytes, 4096).expect("header");
    let limits = InputLimits::default();
    let a = InputCapability::from_header("same.pst", &header_a, limits);
    let b = InputCapability::from_header("same.pst", &header_b, limits);
    assert_eq!(a, b);
}
