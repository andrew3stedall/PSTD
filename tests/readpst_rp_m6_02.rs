use pstd::pst::crypto::{
    decode_in_place, encode_permutative, NDB_CRYPT_NONE, NDB_CRYPT_PERMUTE, NDB_CRYPT_STRONG,
};

#[test]
fn pinned_crypt_modes_are_repeatable_and_semantically_reversible() {
    let plaintext = b"readpst crypto evidence";
    let encoded = encode_permutative(plaintext);
    let mut first = encoded.clone();
    let mut second = encoded;

    assert_eq!(
        decode_in_place(0, &mut first, NDB_CRYPT_PERMUTE).unwrap(),
        "payload_loaded_permute_decoded"
    );
    assert_eq!(
        decode_in_place(0, &mut second, NDB_CRYPT_PERMUTE).unwrap(),
        "payload_loaded_permute_decoded"
    );
    assert_eq!(first, plaintext);
    assert_eq!(first, second);

    let mut unencoded = plaintext.to_vec();
    assert_eq!(
        decode_in_place(0, &mut unencoded, NDB_CRYPT_NONE).unwrap(),
        "payload_loaded_unencoded"
    );
    assert_eq!(unencoded, plaintext);
}

#[test]
fn pinned_strong_crypt_vector_decodes_in_the_public_production_module() {
    let mut encrypted = vec![
        0x6f, 0xab, 0x36, 0xbf, 0xbe, 0x12, 0x8e, 0x2b, 0xa8, 0xc4, 0xa6, 0x33, 0xd9, 0x09, 0x61,
        0xbe, 0x75,
    ];
    let status = decode_in_place(0x12345678, &mut encrypted, NDB_CRYPT_STRONG).unwrap();

    assert_eq!(status, "payload_loaded_strong_crypt_decoded");
    assert_eq!(encrypted, b"strong-crypt-test");
}

#[test]
fn unknown_crypt_method_is_an_explicit_negative_result() {
    let error = decode_in_place(0, &mut [0u8; 4], 7).unwrap_err();
    assert!(error
        .to_string()
        .contains("unsupported PST data block crypt method 0x07"));
}
