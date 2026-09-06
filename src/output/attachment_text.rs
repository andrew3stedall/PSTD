//! Bounded, plain-text projections for attachment formats that are useful to
//! downstream search without pretending to understand their meaning.

use std::collections::BTreeSet;
use std::io::{Cursor, Read, Write};
use std::process::{Command, Stdio};

use sha2::{Digest, Sha256};
use zip::ZipArchive;

use crate::config::AttachmentTextMode;
use crate::output::metadata::AttachmentRecord;
use crate::pst::attachments::AttachmentPayload;

const MAX_ENTRY_BYTES: u64 = 64 * 1024 * 1024;
const MAX_TEXT_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AttachmentTextRecord {
    pub attachment_key: String,
    pub message_key: String,
    pub filename: String,
    pub content_type: Option<String>,
    pub source_archive_path: String,
    pub source_size_bytes: u64,
    pub source_sha256: String,
    pub parser: String,
    pub text: Option<String>,
    pub text_size_bytes: Option<u64>,
    pub text_sha256: Option<String>,
    pub status: String,
}

pub fn parse_attachment_text_records(
    mode: AttachmentTextMode,
    attachments: &[AttachmentRecord],
    payloads: &[AttachmentPayload],
) -> Vec<AttachmentTextRecord> {
    iter_attachment_text_records(mode, attachments, payloads).collect()
}

/// Parse and release one text projection at a time without cloning source metadata.
pub fn iter_attachment_text_records<'a>(
    mode: AttachmentTextMode,
    attachments: &'a [AttachmentRecord],
    payloads: &'a [AttachmentPayload],
) -> impl Iterator<Item = AttachmentTextRecord> + 'a {
    let index = if mode == AttachmentTextMode::None {
        crate::output::attachment_index::AttachmentIndex::new(&[], &[])
    } else {
        crate::output::attachment_index::AttachmentIndex::new(attachments, payloads)
    };
    (0..index.records.len()).map(move |position| {
        let record = index.records[position];
        match index.payload(&record.attachment_key) {
            Ok(payload) => parse_attachment_text(mode, record, payload),
            Err(_) => {
                let mut output = parse_attachment_text(mode, record, None);
                output.status = "attachment_payload_duplicate_id".to_string();
                output
            }
        }
    })
}

fn parse_attachment_text(
    mode: AttachmentTextMode,
    record: &AttachmentRecord,
    payload: Option<&AttachmentPayload>,
) -> AttachmentTextRecord {
    let format = attachment_format(record);
    let parser = match format {
        AttachmentFormat::Pdf => "pdf-pdftotext",
        AttachmentFormat::OfficeOpenXml => "office-open-xml",
        AttachmentFormat::LegacyOffice => "unsupported-legacy-office",
        AttachmentFormat::Unsupported => "unsupported",
    };
    let mut output = base_text_record(record, parser);

    if mode != AttachmentTextMode::OfficePdf {
        output.status = "attachment_text_not_requested".to_string();
        return output;
    }
    let Some(payload) = payload else {
        output.status = "attachment_payload_unavailable".to_string();
        return output;
    };
    if !payload_matches_record(payload, record) {
        output.status = "attachment_payload_integrity_failed".to_string();
        return output;
    }

    let result = match format {
        AttachmentFormat::Pdf => parse_pdf(&payload.bytes),
        AttachmentFormat::OfficeOpenXml => parse_office_open_xml(record, &payload.bytes),
        AttachmentFormat::LegacyOffice => {
            Err("legacy Office binary format is not enabled".to_string())
        }
        AttachmentFormat::Unsupported => {
            Err("attachment type is outside the Office/PDF text scope".to_string())
        }
    };
    match result {
        Ok(text) => {
            let text = normalize_text(&text);
            let text_bytes = text.as_bytes();
            output.text_size_bytes = Some(text_bytes.len() as u64);
            output.text_sha256 = Some(sha256_hex(text_bytes));
            output.text = Some(text);
            output.status = if output.text.as_deref().is_some_and(str::is_empty) {
                "attachment_text_parsed_empty".to_string()
            } else {
                "attachment_text_parsed".to_string()
            };
        }
        Err(status) => output.status = status,
    }
    output
}

fn base_text_record(record: &AttachmentRecord, parser: &str) -> AttachmentTextRecord {
    AttachmentTextRecord {
        attachment_key: record.attachment_key.clone(),
        message_key: record.message_key.clone(),
        filename: record
            .filename_original
            .clone()
            .unwrap_or_else(|| record.filename_safe.clone()),
        content_type: record.content_type.clone(),
        source_archive_path: record.archive_path.clone(),
        source_size_bytes: record.size_bytes,
        source_sha256: record.sha256.clone(),
        parser: parser.to_string(),
        text: None,
        text_size_bytes: None,
        text_sha256: None,
        status: "attachment_text_not_attempted".to_string(),
    }
}

fn payload_matches_record(payload: &AttachmentPayload, record: &AttachmentRecord) -> bool {
    payload.record.message_key == record.message_key
        && payload.record.attachment_key == record.attachment_key
        && payload.record.archive_path == record.archive_path
        && payload.bytes.len() as u64 == record.size_bytes
        && sha256_hex(&payload.bytes) == record.sha256
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AttachmentFormat {
    Pdf,
    OfficeOpenXml,
    LegacyOffice,
    Unsupported,
}

fn attachment_format(record: &AttachmentRecord) -> AttachmentFormat {
    let extension = record
        .extension
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let content_type = record
        .content_type
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if extension == "pdf" || content_type == "application/pdf" {
        return AttachmentFormat::Pdf;
    }
    if matches!(
        extension.as_str(),
        "docx"
            | "docm"
            | "dotx"
            | "dotm"
            | "xlsx"
            | "xlsm"
            | "xltx"
            | "xltm"
            | "pptx"
            | "pptm"
            | "potx"
            | "potm"
    ) || content_type.starts_with("application/vnd.openxmlformats-officedocument.")
    {
        return AttachmentFormat::OfficeOpenXml;
    }
    if matches!(
        extension.as_str(),
        "doc" | "xls" | "ppt" | "dot" | "xlt" | "pot"
    ) || content_type == "application/msword"
        || content_type == "application/vnd.ms-excel"
        || content_type == "application/vnd.ms-powerpoint"
    {
        return AttachmentFormat::LegacyOffice;
    }
    AttachmentFormat::Unsupported
}

fn parse_pdf(bytes: &[u8]) -> Result<String, String> {
    if !bytes.starts_with(b"%PDF-")
        || !bytes
            .windows(b"%%EOF".len())
            .any(|window| window == b"%%EOF")
    {
        return Err("pdf_invalid_structure".to_string());
    }

    let mut child = Command::new("pdftotext")
        .args(["-layout", "-enc", "UTF-8", "-", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                "pdf_text_parser_unavailable".to_string()
            } else {
                format!("pdf_text_parser_start_failed: {error}")
            }
        })?;
    child
        .stdin
        .take()
        .ok_or_else(|| "pdf_text_parser_stdin_unavailable".to_string())?
        .write_all(bytes)
        .map_err(|error| format!("pdf_text_parser_input_failed: {error}"))?;
    let output = child
        .wait_with_output()
        .map_err(|error| format!("pdf_text_parser_wait_failed: {error}"))?;
    if !output.status.success() {
        return Err("pdf_text_parse_failed".to_string());
    }
    String::from_utf8(output.stdout).map_err(|_| "pdf_text_invalid_utf8_output".to_string())
}

fn parse_office_open_xml(record: &AttachmentRecord, bytes: &[u8]) -> Result<String, String> {
    let mut archive =
        ZipArchive::new(Cursor::new(bytes)).map_err(|_| "office_package_invalid".to_string())?;
    let names = (0..archive.len())
        .filter_map(|index| {
            archive
                .by_index(index)
                .ok()
                .map(|file| file.name().to_string())
        })
        .collect::<BTreeSet<_>>();
    let extension = record
        .extension
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let content_type = record
        .content_type
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();

    if extension.starts_with("xls")
        || extension.starts_with("xlt")
        || content_type.contains("spreadsheetml")
    {
        parse_xlsx(&mut archive, &names)
    } else if extension.starts_with("ppt")
        || extension.starts_with("pot")
        || content_type.contains("presentationml")
    {
        parse_pptx(&mut archive, &names)
    } else {
        parse_docx(&mut archive, &names)
    }
}

fn parse_docx<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    names: &BTreeSet<String>,
) -> Result<String, String> {
    let mut selected = names
        .iter()
        .filter(|name| {
            name.ends_with(".xml")
                && ((*name).as_str() == "word/document.xml"
                    || name.starts_with("word/header")
                    || name.starts_with("word/footer")
                    || (*name).as_str() == "word/footnotes.xml"
                    || (*name).as_str() == "word/endnotes.xml"
                    || (*name).as_str() == "word/comments.xml")
        })
        .cloned()
        .collect::<Vec<_>>();
    selected.sort();
    if selected.is_empty() {
        return Err("office_package_missing_word_document".to_string());
    }
    let mut text = String::new();
    for name in selected {
        let bytes = read_zip_entry(archive, &name)?;
        text.push_str(&xml_readable_text(&bytes));
        text.push('\n');
    }
    Ok(text)
}

fn parse_pptx<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    names: &BTreeSet<String>,
) -> Result<String, String> {
    let mut selected = names
        .iter()
        .filter(|name| name.starts_with("ppt/slides/slide") && name.ends_with(".xml"))
        .cloned()
        .collect::<Vec<_>>();
    selected.sort_by_key(|name| natural_part_key(name));
    if selected.is_empty() {
        return Err("office_package_missing_presentation_slides".to_string());
    }
    let mut text = String::new();
    for name in selected {
        text.push_str(&xml_readable_text(&read_zip_entry(archive, &name)?));
        text.push('\n');
    }
    Ok(text)
}

fn parse_xlsx<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    names: &BTreeSet<String>,
) -> Result<String, String> {
    let shared_strings = if names.contains("xl/sharedStrings.xml") {
        let xml =
            String::from_utf8_lossy(&read_zip_entry(archive, "xl/sharedStrings.xml")?).into_owned();
        extract_named_blocks(&xml, "si")
            .iter()
            .map(|block| xml_readable_text(block.as_bytes()))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let mut sheets = names
        .iter()
        .filter(|name| name.starts_with("xl/worksheets/sheet") && name.ends_with(".xml"))
        .cloned()
        .collect::<Vec<_>>();
    sheets.sort_by_key(|name| natural_part_key(name));
    if sheets.is_empty() {
        return Err("office_package_missing_spreadsheets".to_string());
    }

    let mut text = String::new();
    for sheet in sheets {
        let xml = String::from_utf8_lossy(&read_zip_entry(archive, &sheet)?).into_owned();
        for row in extract_named_blocks(&xml, "row") {
            let mut cells = Vec::new();
            for cell in extract_named_blocks(&row, "c") {
                let open_tag = cell.split_once('>').map(|(tag, _)| tag).unwrap_or_default();
                let cell_type = attribute_value(open_tag, "t");
                let value = match cell_type.as_deref() {
                    Some("s") => extract_named_blocks(&cell, "v")
                        .first()
                        .and_then(|value| value.trim().parse::<usize>().ok())
                        .and_then(|index| shared_strings.get(index).cloned())
                        .unwrap_or_default(),
                    Some("inlineStr") => xml_readable_text(cell.as_bytes()),
                    _ => extract_named_blocks(&cell, "v")
                        .first()
                        .map(|value| xml_readable_text(value.as_bytes()))
                        .unwrap_or_default(),
                };
                cells.push(value);
            }
            if !cells.is_empty() {
                text.push_str(&cells.join("\t"));
                text.push('\n');
            }
        }
    }
    Ok(text)
}

fn read_zip_entry<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    name: &str,
) -> Result<Vec<u8>, String> {
    let mut entry = archive
        .by_name(name)
        .map_err(|_| "office_package_entry_unavailable".to_string())?;
    if entry.size() > MAX_ENTRY_BYTES {
        return Err("office_package_entry_exceeds_budget".to_string());
    }
    let mut bytes = Vec::with_capacity(entry.size() as usize);
    entry
        .read_to_end(&mut bytes)
        .map_err(|_| "office_package_entry_read_failed".to_string())?;
    if bytes.len() > MAX_TEXT_BYTES {
        return Err("office_text_exceeds_budget".to_string());
    }
    Ok(bytes)
}

fn extract_named_blocks(source: &str, name: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut cursor = 0usize;
    while let Some(relative) = source[cursor..].find('<') {
        let start = cursor + relative;
        let Some(end) = source[start..].find('>') else {
            break;
        };
        let end = start + end;
        let open = &source[start..=end];
        if tag_name(open) != Some(name) || open.starts_with("</") || open.ends_with("/>") {
            cursor = end + 1;
            continue;
        }
        let close_marker = format!("</{name}");
        let Some(close_relative) = source[end + 1..].find(&close_marker) else {
            break;
        };
        let close_start = end + 1 + close_relative;
        let Some(close_end_relative) = source[close_start..].find('>') else {
            break;
        };
        let close_end = close_start + close_end_relative;
        blocks.push(source[start..=close_end].to_string());
        cursor = close_end + 1;
    }
    blocks
}

fn tag_name(tag: &str) -> Option<&str> {
    let tag = tag
        .trim_start_matches('<')
        .trim_start_matches('/')
        .trim_end_matches('>');
    let tag = tag.trim_end_matches('/').trim();
    let name = tag.split_whitespace().next()?;
    (!name.starts_with('!') && !name.starts_with('?'))
        .then_some(name.rsplit(':').next().unwrap_or(name))
}

fn attribute_value(tag: &str, name: &str) -> Option<String> {
    let tag = tag.trim_start_matches('<').trim_end_matches('>');
    let needle = format!("{name}=");
    let start = tag.find(&needle)? + needle.len();
    let quote = tag.as_bytes().get(start).copied()? as char;
    if !matches!(quote, '\'' | '"') {
        return None;
    }
    let value_start = start + 1;
    let end = tag[value_start..].find(quote)? + value_start;
    Some(tag[value_start..end].to_string())
}

fn xml_readable_text(xml: &[u8]) -> String {
    let source = String::from_utf8_lossy(xml);
    let mut text = String::new();
    let mut cursor = 0usize;
    while cursor < source.len() {
        let Some(relative) = source[cursor..].find('<') else {
            text.push_str(&decode_xml_entities(&source[cursor..]));
            break;
        };
        let start = cursor + relative;
        text.push_str(&decode_xml_entities(&source[cursor..start]));
        let Some(end_relative) = source[start..].find('>') else {
            break;
        };
        let end = start + end_relative;
        let tag = &source[start..=end];
        let name = tag_name(tag).unwrap_or_default();
        if name == "br" || name == "tab" {
            text.push(if name == "tab" { '\t' } else { '\n' });
        } else if tag.starts_with("</") && matches!(name, "p" | "tr" | "row" | "li" | "txBody") {
            text.push('\n');
        }
        cursor = end + 1;
    }
    normalize_text(&text)
}

fn decode_xml_entities(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    while let Some(relative) = value[cursor..].find('&') {
        let start = cursor + relative;
        output.push_str(&value[cursor..start]);
        let Some(end_relative) = value[start..].find(';') else {
            output.push_str(&value[start..]);
            return output;
        };
        let end = start + end_relative;
        let entity = &value[start + 1..end];
        let decoded = match entity {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            _ if entity.starts_with("#x") => u32::from_str_radix(&entity[2..], 16)
                .ok()
                .and_then(char::from_u32),
            _ if entity.starts_with('#') => {
                entity[1..].parse::<u32>().ok().and_then(char::from_u32)
            }
            _ => None,
        };
        if let Some(decoded) = decoded {
            output.push(decoded);
        } else {
            output.push_str(&value[start..=end]);
        }
        cursor = end + 1;
    }
    output.push_str(&value[cursor..]);
    output
}

fn normalize_text(value: &str) -> String {
    let mut output = String::new();
    let mut blank_lines = 0usize;
    for line in value.replace("\r\n", "\n").replace('\r', "\n").lines() {
        let line = line.trim_end();
        if line.trim().is_empty() {
            blank_lines += 1;
            if blank_lines <= 2 && !output.is_empty() {
                output.push('\n');
            }
        } else {
            blank_lines = 0;
            if !output.is_empty() && !output.ends_with('\n') {
                output.push('\n');
            }
            output.push_str(line);
        }
    }
    output.trim().to_string()
}

fn natural_part_key(name: &str) -> u64 {
    let digits = name
        .chars()
        .filter(|character| character.is_ascii_digit())
        .collect::<String>();
    digits.parse().unwrap_or(u64::MAX)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write};

    use super::{
        attribute_value, decode_xml_entities, extract_named_blocks, parse_pdf, xml_readable_text,
    };

    #[test]
    fn extracts_readable_ooxml_text_and_entities() {
        let xml = br#"<w:document><w:p><w:r><w:t>Hello &amp; world</w:t></w:r></w:p><w:p>Next</w:p></w:document>"#;
        assert_eq!(xml_readable_text(xml), "Hello & world\nNext");
        assert_eq!(decode_xml_entities("&#x41;&#66;"), "AB");
    }

    #[test]
    fn extracts_named_blocks_and_attributes() {
        let blocks = extract_named_blocks(r#"<row r="1"><c t="s"><v>0</v></c></row>"#, "c");
        assert_eq!(blocks, vec![r#"<c t="s"><v>0</v></c>"#]);
        assert_eq!(attribute_value(r#"<c t="s">"#, "t").as_deref(), Some("s"));
    }

    #[test]
    fn reads_ooxml_fixture_without_semantic_interpretation() {
        let mut bytes = Cursor::new(Vec::new());
        {
            let mut archive = zip::ZipWriter::new(&mut bytes);
            let options = zip::write::SimpleFileOptions::default();
            archive.start_file("word/document.xml", options).unwrap();
            archive
                .write_all(br#"<w:document><w:p><w:r><w:t>Report &amp; notes</w:t></w:r></w:p></w:document>"#)
                .unwrap();
            archive.finish().unwrap();
        }
        let record = crate::pst::attachments::attachment_payload(
            "msg_1",
            0,
            crate::pst::attachments::AttachmentMetadata {
                filename_original: Some("report.docx".to_string()),
                ..Default::default()
            },
            bytes.into_inner(),
        );
        let records = super::parse_attachment_text_records(
            crate::config::AttachmentTextMode::OfficePdf,
            std::slice::from_ref(&record.record),
            std::slice::from_ref(&record),
        );
        assert_eq!(records[0].status, "attachment_text_parsed");
        assert_eq!(records[0].text.as_deref(), Some("Report & notes"));
    }

    #[test]
    fn reads_well_formed_pdf_as_plain_text_when_backend_is_available() {
        let stream = "BT\n/F1 12 Tf\n72 720 Td\n(Hello PDF) Tj\nET\n";
        let objects = [
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>".to_string(),
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
            format!("<< /Length {} >>\nstream\n{}endstream", stream.len(), stream),
        ];
        let mut pdf = b"%PDF-1.4\n".to_vec();
        let mut offsets = vec![0usize];
        for (index, object) in objects.iter().enumerate() {
            offsets.push(pdf.len());
            pdf.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", index + 1, object).as_bytes());
        }
        let xref_offset = pdf.len();
        pdf.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
        pdf.extend_from_slice(b"0000000000 65535 f \n");
        for offset in offsets.iter().skip(1) {
            pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        pdf.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
                objects.len() + 1,
                xref_offset
            )
            .as_bytes(),
        );

        match parse_pdf(&pdf) {
            Ok(text) => assert!(text.contains("Hello PDF"), "unexpected PDF text: {text:?}"),
            Err(status) if status == "pdf_text_parser_unavailable" => {}
            Err(status) => panic!("well-formed PDF failed to parse: {status}"),
        }
    }
}
