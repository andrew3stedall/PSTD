const LZFU_MAGIC: u32 = 0x7546_5a4c;
const MELA_MAGIC: u32 = 0x414c_454d;
const MAX_RTF_OUTPUT_BYTES: usize = 16 * 1024 * 1024;
const INITIAL_DICTIONARY: &[u8] = b"{\\rtf1\\ansi\\mac\\deff0\\deftab720{\\fonttbl;}{\\f0\\fnil \\froman \\fswiss \\fmodern \\fscript \\fdecor MS Sans SerifSymbolArialTimes New RomanCourier{\\colortbl\\red0\\green0\\blue0\r\n\\par \\pard\\plain\\f0\\fs20\\b\\i\\u\\tab\\tx";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtfValidation {
    pub status: String,
    pub decoded: Option<Vec<u8>>,
    pub recovered_html: Option<String>,
}

pub fn validate(input: &[u8]) -> RtfValidation {
    let (decoded, status) = if input.starts_with(b"{\\rtf") {
        if balanced_groups(input) {
            (Some(input.to_vec()), "rtf_valid_direct")
        } else {
            (None, "rtf_invalid_unbalanced_groups")
        }
    } else {
        match decompress_rtf(input) {
            Some(value) if value.starts_with(b"{\\rtf") && balanced_groups(&value) => {
                (Some(value), "rtf_valid_lzfu")
            }
            Some(_) => (None, "rtf_invalid_decoded_magic"),
            None => (None, "rtf_invalid_header_crc_size_or_budget"),
        }
    };

    let recovered_html = decoded.as_deref().and_then(recover_fromhtml);
    let status = match (status, recovered_html.is_some()) {
        ("rtf_valid_direct", true) => "rtf_valid_direct; html_recovered_fromhtml1",
        ("rtf_valid_lzfu", true) => "rtf_valid_lzfu; html_recovered_fromhtml1",
        (value, _) => value,
    };
    RtfValidation {
        status: status.to_string(),
        decoded,
        recovered_html,
    }
}

pub fn decompress_rtf(input: &[u8]) -> Option<Vec<u8>> {
    if input.len() < 16 {
        return None;
    }
    let compressed_size = read_u32(input, 0)? as usize;
    let raw_size = read_u32(input, 4)? as usize;
    let magic = read_u32(input, 8)?;
    let expected_crc = read_u32(input, 12)?;
    if compressed_size.checked_add(4)? != input.len() || raw_size > MAX_RTF_OUTPUT_BYTES {
        return None;
    }
    let payload = input.get(16..)?;
    match magic {
        MELA_MAGIC => {
            if expected_crc != 0
                || compressed_size != raw_size
                || payload.len().checked_add(12)? != raw_size
            {
                return None;
            }
            Some(payload.to_vec())
        }
        LZFU_MAGIC => {
            if crc32(payload) != expected_crc {
                return None;
            }
            let decoded = decompress_lzfu(payload, raw_size)?;
            (decoded.len() == raw_size).then_some(decoded)
        }
        _ => None,
    }
}

fn decompress_lzfu(input: &[u8], raw_size: usize) -> Option<Vec<u8>> {
    if raw_size > MAX_RTF_OUTPUT_BYTES || INITIAL_DICTIONARY.len() > 4096 {
        return None;
    }
    let mut dictionary = [0u8; 4096];
    dictionary[..INITIAL_DICTIONARY.len()].copy_from_slice(INITIAL_DICTIONARY);
    let mut dictionary_position = INITIAL_DICTIONARY.len();
    let mut output = Vec::with_capacity(raw_size);
    let mut input_position = 0usize;
    while output.len() < raw_size {
        let flags = *input.get(input_position)?;
        input_position += 1;
        for bit in 0..8 {
            if output.len() == raw_size {
                break;
            }
            if flags & (1 << bit) == 0 {
                let value = *input.get(input_position)?;
                input_position += 1;
                output.push(value);
                dictionary[dictionary_position & 0x0fff] = value;
                dictionary_position = (dictionary_position + 1) & 0x0fff;
            } else {
                let first = *input.get(input_position)? as usize;
                let second = *input.get(input_position + 1)? as usize;
                input_position += 2;
                let mut reference = (first << 4) | (second >> 4);
                let length = (second & 0x0f) + 2;
                for _ in 0..length {
                    if output.len() == raw_size {
                        break;
                    }
                    let value = dictionary[reference & 0x0fff];
                    reference = (reference + 1) & 0x0fff;
                    output.push(value);
                    dictionary[dictionary_position & 0x0fff] = value;
                    dictionary_position = (dictionary_position + 1) & 0x0fff;
                }
            }
        }
    }
    Some(output)
}

fn balanced_groups(input: &[u8]) -> bool {
    let mut depth = 0usize;
    for byte in input {
        match byte {
            b'{' => depth = depth.saturating_add(1),
            b'}' => {
                if depth == 0 {
                    return false;
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    depth == 0
}

#[derive(Clone, Copy)]
struct HtmlState {
    skip: bool,
    htmltag: bool,
    ignorable: bool,
}

fn recover_fromhtml(input: &[u8]) -> Option<String> {
    if !input.starts_with(b"{\\rtf")
        || !input
            .windows(b"\\fromhtml1".len())
            .any(|window| window == b"\\fromhtml1")
    {
        return None;
    }
    let mut output = String::new();
    let mut stack = vec![HtmlState {
        skip: false,
        htmltag: false,
        ignorable: false,
    }];
    let mut index = 0usize;
    while index < input.len() {
        match input[index] {
            b'{' => {
                let mut state = *stack.last()?;
                state.ignorable = false;
                stack.push(state);
                index += 1;
            }
            b'}' => {
                if stack.len() == 1 {
                    return None;
                }
                stack.pop();
                index += 1;
            }
            b'\\' => {
                let (word, number, next) = read_control(input, index)?;
                index = next;
                let state = stack.last_mut()?;
                match word.as_str() {
                    "*" => state.ignorable = true,
                    "htmltag" => {
                        state.htmltag = true;
                        state.skip = false;
                    }
                    "htmlrtf" => state.skip = true,
                    "par" | "line" if !state.skip => output.push('\n'),
                    "tab" if !state.skip => output.push('\t'),
                    "hex" if !state.skip => output.push(cp1252_char(number? as u8)?),
                    "{" | "}" | "\\" if !state.skip => output.push_str(&word),
                    destination
                        if ["fonttbl", "colortbl", "stylesheet", "info"].contains(&destination) =>
                    {
                        state.skip = true
                    }
                    _ if state.ignorable && !state.htmltag => state.skip = true,
                    _ => {}
                }
            }
            byte => {
                if !stack.last()?.skip && byte != b'\r' && byte != b'\n' {
                    output.push(byte as char);
                }
                index += 1;
            }
        }
    }
    if stack.len() != 1 {
        return None;
    }
    let html = output.trim().to_string();
    (html.contains('<') && html.contains('>') && !html.contains("{\\rtf") && !html.contains('\\'))
        .then_some(html)
}

fn read_control(input: &[u8], start: usize) -> Option<(String, Option<i32>, usize)> {
    let mut index = start.checked_add(1)?;
    let first = *input.get(index)?;
    if matches!(first, b'{' | b'}' | b'\\') {
        return Some(((first as char).to_string(), None, index + 1));
    }
    if first == b'\'' {
        let hex = std::str::from_utf8(input.get(index + 1..index + 3)?).ok()?;
        return Some((
            "hex".to_string(),
            Some(i32::from(u8::from_str_radix(hex, 16).ok()?)),
            index + 3,
        ));
    }
    if !first.is_ascii_alphabetic() {
        return Some(((first as char).to_string(), None, index + 1));
    }
    let word_start = index;
    while input.get(index).is_some_and(u8::is_ascii_alphabetic) {
        index += 1;
    }
    let word = std::str::from_utf8(input.get(word_start..index)?)
        .ok()?
        .to_string();
    let mut sign = 1i32;
    if input.get(index) == Some(&b'-') {
        sign = -1;
        index += 1;
    }
    let number_start = index;
    while input.get(index).is_some_and(u8::is_ascii_digit) {
        index += 1;
    }
    let number = if index > number_start {
        Some(
            std::str::from_utf8(input.get(number_start..index)?)
                .ok()?
                .parse::<i32>()
                .ok()?
                * sign,
        )
    } else {
        None
    };
    if input.get(index) == Some(&b' ') {
        index += 1;
    }
    Some((word, number, index))
}

fn cp1252_char(value: u8) -> Option<char> {
    match value {
        0x00..=0x7f | 0xa0..=0xff => char::from_u32(u32::from(value)),
        0x80 => Some('€'),
        0x82 => Some('‚'),
        0x83 => Some('ƒ'),
        0x84 => Some('„'),
        0x85 => Some('…'),
        0x86 => Some('†'),
        0x87 => Some('‡'),
        0x88 => Some('ˆ'),
        0x89 => Some('‰'),
        0x8a => Some('Š'),
        0x8b => Some('‹'),
        0x8c => Some('Œ'),
        0x8e => Some('Ž'),
        0x91 => Some('‘'),
        0x92 => Some('’'),
        0x93 => Some('“'),
        0x94 => Some('”'),
        0x95 => Some('•'),
        0x96 => Some('–'),
        0x97 => Some('—'),
        0x98 => Some('˜'),
        0x99 => Some('™'),
        0x9a => Some('š'),
        0x9b => Some('›'),
        0x9c => Some('œ'),
        0x9e => Some('ž'),
        0x9f => Some('Ÿ'),
        _ => None,
    }
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & (0u32.wrapping_sub(crc & 1)));
        }
    }
    !crc
}

fn read_u32(input: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        input.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

#[cfg(test)]
mod tests {
    use super::{validate, MELA_MAGIC};

    fn wrap_uncompressed(raw: &[u8]) -> Vec<u8> {
        let framed_size = raw.len() + 12;
        let mut value = Vec::new();
        value.extend_from_slice(&(framed_size as u32).to_le_bytes());
        value.extend_from_slice(&(framed_size as u32).to_le_bytes());
        value.extend_from_slice(&MELA_MAGIC.to_le_bytes());
        value.extend_from_slice(&0u32.to_le_bytes());
        value.extend_from_slice(raw);
        value
    }

    #[test]
    fn validates_direct_and_wrapped_rtf() {
        let raw = b"{\\rtf1\\ansi Hello}";
        assert_eq!(validate(raw).status, "rtf_valid_direct");
        assert_eq!(validate(&wrap_uncompressed(raw)).status, "rtf_valid_lzfu");
    }

    #[test]
    fn rejects_malformed_header_crc_and_unbalanced_input() {
        let raw = b"{\\rtf1 Hello}";
        let mut bad_crc = wrap_uncompressed(raw);
        bad_crc[12] = 1;
        assert_eq!(validate(&bad_crc).decoded, None);
        assert_eq!(
            validate(b"{\\rtf1 broken").status,
            "rtf_invalid_unbalanced_groups"
        );
    }

    #[test]
    fn recovers_only_valid_fromhtml_markup() {
        let value = b"{\\rtf1\\fromhtml1{\\*\\htmltag <b>}Bold{\\*\\htmltag </b>}}";
        let projection = validate(value);
        assert_eq!(projection.recovered_html.as_deref(), Some("<b>Bold</b>"));
        assert!(validate(b"{\\rtf1 plain}").recovered_html.is_none());
    }
}
