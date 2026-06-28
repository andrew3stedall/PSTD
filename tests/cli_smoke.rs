#[test]
fn package_version_is_available() {
    assert_ne!(env!("CARGO_PKG_VERSION"), "");
}
