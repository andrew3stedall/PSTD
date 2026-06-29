use crate::error::{PstdError, PstdResult};
use crate::pst::binary::{slice_at, u16_le_at, u32_le_at};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TableColumn {
    pub tag: u32,
    pub offset: u16,
    pub width: u16,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TableRow {
    pub row_id: u32,
    pub values: Vec<(u32, Vec<u8>)>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TableContext {
    pub columns: Vec<TableColumn>,
    pub rows: Vec<TableRow>,
}

impl TableContext {
    pub fn parse(buf: &[u8], base_offset: u64) -> PstdResult<Self> {
        if buf.len() < 8 {
            return Err(PstdError::pst_parse(Some(base_offset), "table context buffer too short"));
        }
        let column_count = u16_le_at(buf, 0, base_offset)? as usize;
        let row_count = u16_le_at(buf, 2, base_offset)? as usize;
        let row_width = u16_le_at(buf, 4, base_offset)? as usize;
        let mut cursor = 8usize;
        let mut columns = Vec::new();
        for _ in 0..column_count {
            if cursor + 8 > buf.len() { break; }
            columns.push(TableColumn {
                tag: u32_le_at(buf, cursor, base_offset)?,
                offset: u16_le_at(buf, cursor + 4, base_offset)?,
                width: u16_le_at(buf, cursor + 6, base_offset)?,
            });
            cursor += 8;
        }

        let mut rows = Vec::new();
        for row_index in 0..row_count {
            if cursor + row_width > buf.len() { break; }
            let row_buf = slice_at(buf, cursor, row_width, base_offset)?;
            let row_id = row_index as u32;
            let mut values = Vec::new();
            for column in &columns {
                let start = column.offset as usize;
                let width = column.width as usize;
                if start + width <= row_buf.len() {
                    values.push((column.tag, row_buf[start..start + width].to_vec()));
                }
            }
            rows.push(TableRow { row_id, values });
            cursor += row_width;
        }
        Ok(Self { columns, rows })
    }
}
