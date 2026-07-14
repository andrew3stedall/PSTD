use std::env;
use std::fs;
use std::path::PathBuf;

use pstd::engine::metadata::extract_metadata;
use pstd::pst::messages::BodyPayload;

const LZFU_MAGIC: u32 = 0x7546_5a4c;
const MELA_MAGIC: u32 = 0x414c_454d;
const INITIAL_DICTIONARY: &[u8] = b"{\\rtf1\\ansi\\mac\\deff0\\deftab720{\\fonttbl;}{\\f0\\fnil \\froman \\fswiss \\fmodern \\fscript \\fdecor MS Sans SerifSymbolArialTimes New RomanCourier{\\colortbl\\red0\\green0\\blue0\r\n\\par \\pard\\plain\\f0\\fs20\\b\\i\\u\\tab\\tx";

fn main() {
    if let Err(error) = run() {
        eprintln!("pstd-rtf: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args_os().skip(1);
    let input = args
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "usage: pstd-rtf <input.pst> <output-dir>".to_string())?;
    let output = args
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "usage: pstd-rtf <input.pst> <output-dir>".to_string())?;
    if args.next().is_some() {
        return Err("usage: pstd-rtf <input.pst> <output-dir>".to_string());
    }

    fs::create_dir_all(&output).map_err(|error| error.to_string())?;
    let input_display = input.display().to_string();
    let run_id = pstd::output::ids::run_id(&input_display);
    let pst_id = pstd::output::ids::pst_id(&input_display);
    let metadata = extract_metadata(&input_display, &run_id, &pst_id)
        .map_err(|error| format!("metadata extraction failed: {error}"))?;

    let mut emitted = 0usize;
    for payload in metadata
        .body_payloads
        .iter()
        .filter(|payload| payload.record.body_type == "rtf")
    {
        let Some(rtf) = validated_rtf(payload) else {
            continue;
        };
        let path = output.join(format!("{}.rtf", safe_filename(&payload.record.message_key)));
        fs::write(path, rtf).map_err(|error| error.to_string())?;
        emitted += 1;
    }

    println!("rtf_files_emitted={emitted}");
    if emitted == 0 {
        return Err("no validated compressed RTF body could be decoded".to_string());
    }
    Ok(())
}

fn validated_rtf(payload: &BodyPayload) -> Option<Vec<u8>> {
    let decoded = decompress_rtf(&payload.bytes)?;
    decoded.starts_with(b"{\\rtf").then_some(decoded)
}

fn decompress_rtf(input: &[u8]) -> Option<Vec<u8>> {
    if input.len() < 16 {
        return None;
    }
    let compressed_size = read_u32(input, 0)? as usize;
    let raw_size = read_u32(input, 4)? as usize;
    let magic = read_u32(input, 8)?;
    let expected_crc = read_u32(input, 12)?;
    if compressed_size.checked_add(4)? != input.len() {
        return None;
    }
    let payload = &input[16..];
    if crc32(payload) != expected_crc {
        return None;
    }

    let decoded = match magic {
        MELA_MAGIC => {
            if payload.len() != raw_size {
                return None;
            }
            payload.to_vec()
        }
        LZFU_MAGIC => decompress_lzfu(payload, raw_size)?,
        _ => return None,
    };
    (decoded.len() == raw_size).then_some(decoded)
}

fn decompress_lzfu(input: &[u8], raw_size: usize) -> Option<Vec<u8>> {
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

fn read_u32(input: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(input.get(offset..offset + 4)?.try_into().ok()?))
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

fn safe_filename(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pstd::pst::messages::body_payload;

    fn wrap_uncompressed(raw: &[u8]) -> Vec<u8> {
        let mut value = Vec::new();
        value.extend_from_slice(&((raw.len() + 12) as u32).to_le_bytes());
        value.extend_from_slice(&(raw.len() as u32).to_le_bytes());
        value.extend_from_slice(&MELA_MAGIC.to_le_bytes());
        value.extend_from_slice(&crc32(raw).to_le_bytes());
        value.extend_from_slice(raw);
        value
    }

    #[test]
    fn validates_uncompressed_rtf_container() {
        let raw = b"{\\rtf1\\ansi Hello}";
        let compressed = wrap_uncompressed(raw);
        assert_eq!(decompress_rtf(&compressed).as_deref(), Some(raw.as_slice()));
        let payload = body_payload("message", "rtf", compressed, None);
        assert_eq!(validated_rtf(&payload).as_deref(), Some(raw.as_slice()));
    }

    #[test]
    fn rejects_bad_crc_size_magic_and_non_rtf_output() {
        let raw = b"{\\rtf1 Hello}";
        let mut bad_crc = wrap_uncompressed(raw);
        bad_crc[12] ^= 1;
        assert!(decompress_rtf(&bad_crc).is_none());

        let mut bad_size = wrap_uncompressed(raw);
        bad_size[0] ^= 1;
        assert!(decompress_rtf(&bad_size).is_none());

        let mut bad_magic = wrap_uncompressed(raw);
        bad_magic[8] ^= 1;
        assert!(decompress_rtf(&bad_magic).is_none());

        let payload = body_payload("message", "rtf", wrap_uncompressed(b"plain"), None);
        assert!(validated_rtf(&payload).is_none());
    }
}