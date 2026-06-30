use crate::error::{PstdError, PstdResult};

pub fn require_len(buf: &[u8], needed: usize, offset: Option<u64>) -> PstdResult<()> {
    if buf.len() < needed {
        return Err(PstdError::pst_parse(
            offset,
            format!("buffer too short: needed {needed} bytes, got {}", buf.len()),
        ));
    }
    Ok(())
}

pub fn slice_at(buf: &[u8], start: usize, len: usize, base_offset: u64) -> PstdResult<&[u8]> {
    let end = start.checked_add(len).ok_or_else(|| {
        PstdError::pst_parse(Some(base_offset + start as u64), "slice range overflowed")
    })?;
    if end > buf.len() {
        return Err(PstdError::pst_parse(
            Some(base_offset + start as u64),
            format!(
                "slice beyond buffer: start {start}, len {len}, buffer {}",
                buf.len()
            ),
        ));
    }
    Ok(&buf[start..end])
}

pub fn u8_at(buf: &[u8], start: usize, base_offset: u64) -> PstdResult<u8> {
    Ok(slice_at(buf, start, 1, base_offset)?[0])
}

pub fn u16_le_at(buf: &[u8], start: usize, base_offset: u64) -> PstdResult<u16> {
    let bytes = slice_at(buf, start, 2, base_offset)?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

pub fn u32_le_at(buf: &[u8], start: usize, base_offset: u64) -> PstdResult<u32> {
    let bytes = slice_at(buf, start, 4, base_offset)?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

pub fn u64_le_at(buf: &[u8], start: usize, base_offset: u64) -> PstdResult<u64> {
    let bytes = slice_at(buf, start, 8, base_offset)?;
    Ok(u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]))
}

pub fn i32_le_at(buf: &[u8], start: usize, base_offset: u64) -> PstdResult<i32> {
    Ok(u32_le_at(buf, start, base_offset)? as i32)
}
