//! Lowercase hexadecimal encoding.

const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// Encodes `bytes` as lowercase hexadecimal, two digits per byte.
pub fn encode(bytes: impl AsRef<[u8]>) -> String {
    let bytes = bytes.as_ref();
    let mut out = Vec::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        out.push(HEX_DIGITS[usize::from(byte >> 4)]);
        out.push(HEX_DIGITS[usize::from(byte & 0x0f)]);
    }
    // Every pushed byte is an ASCII hex digit.
    String::from_utf8(out).expect("hex digits are ASCII")
}

#[cfg(test)]
mod tests {
    use super::encode;

    #[test]
    fn encodes_every_byte_value_as_two_lowercase_digits() {
        assert_eq!(encode([]), "");
        assert_eq!(encode([0x00, 0x0f, 0xa5, 0xff]), "000fa5ff");
        let all: Vec<u8> = (0..=255).collect();
        let expected: String = all.iter().map(|byte| format!("{byte:02x}")).collect();
        assert_eq!(encode(&all), expected);
    }
}
