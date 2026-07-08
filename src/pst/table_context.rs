use crate::error::{PstdError, PstdResult};
use crate::pst::binary::{slice_at, u16_le_at, u32_le_at};
use crate::pst::mapi::{has_known_value_type, property_def};

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TableContextParseReport {
    pub context: TableContext,
    pub declared_column_count: usize,
    pub parsed_column_count: usize,
    pub declared_row_count: usize,
    pub parsed_row_count: usize,
    pub row_width: usize,
    pub truncated_column_count: usize,
    pub truncated_row_count: usize,
    pub omitted_value_count: usize,
    pub selected_column_count: usize,
    pub plausible_column_count: usize,
    pub unknown_column_count: usize,
    pub selected_value_count: usize,
    pub plausible_value_count: usize,
    pub unknown_value_count: usize,
    pub status: String,
}

impl TableContext {
    pub fn parse(buf: &[u8], base_offset: u64) -> PstdResult<Self> {
        Ok(Self::parse_with_report(buf, base_offset)?.context)
    }

    pub fn parse_with_report(buf: &[u8], base_offset: u64) -> PstdResult<TableContextParseReport> {
        if buf.len() < 8 {
            return Err(PstdError::pst_parse(
                Some(base_offset),
                "table context buffer too short",
            ));
        }
        let declared_column_count = u16_le_at(buf, 0, base_offset)? as usize;
        let declared_row_count = u16_le_at(buf, 2, base_offset)? as usize;
        let row_width = u16_le_at(buf, 4, base_offset)? as usize;
        let mut cursor = 8usize;
        let mut columns = Vec::new();
        for _ in 0..declared_column_count {
            if cursor + 8 > buf.len() {
                break;
            }
            columns.push(TableColumn {
                tag: u32_le_at(buf, cursor, base_offset)?,
                offset: u16_le_at(buf, cursor + 4, base_offset)?,
                width: u16_le_at(buf, cursor + 6, base_offset)?,
            });
            cursor += 8;
        }

        let parsed_column_count = columns.len();
        let truncated_column_count = declared_column_count.saturating_sub(parsed_column_count);
        let mut rows = Vec::new();
        let mut omitted_value_count = 0usize;
        for row_index in 0..declared_row_count {
            if cursor + row_width > buf.len() {
                break;
            }
            let row_buf = slice_at(buf, cursor, row_width, base_offset)?;
            let row_id = row_index as u32;
            let mut values = Vec::new();
            for column in &columns {
                let start = column.offset as usize;
                let width = column.width as usize;
                if start + width <= row_buf.len() {
                    values.push((column.tag, row_buf[start..start + width].to_vec()));
                } else {
                    omitted_value_count += 1;
                }
            }
            rows.push(TableRow { row_id, values });
            cursor += row_width;
        }

        let selected_column_count = columns
            .iter()
            .filter(|column| property_def(column.tag).is_some())
            .count();
        let plausible_column_count = columns
            .iter()
            .filter(|column| property_def(column.tag).is_none() && has_known_value_type(column.tag))
            .count();
        let unknown_column_count = parsed_column_count
            .saturating_sub(selected_column_count)
            .saturating_sub(plausible_column_count);
        let selected_value_count = rows
            .iter()
            .flat_map(|row| row.values.iter())
            .filter(|(tag, _)| property_def(*tag).is_some())
            .count();
        let plausible_value_count = rows
            .iter()
            .flat_map(|row| row.values.iter())
            .filter(|(tag, _)| property_def(*tag).is_none() && has_known_value_type(*tag))
            .count();
        let value_count = rows.iter().map(|row| row.values.len()).sum::<usize>();
        let unknown_value_count = value_count
            .saturating_sub(selected_value_count)
            .saturating_sub(plausible_value_count);
        let parsed_row_count = rows.len();
        let truncated_row_count = declared_row_count.saturating_sub(parsed_row_count);
        let status = if truncated_column_count == 0
            && truncated_row_count == 0
            && omitted_value_count == 0
        {
            "table_context_parsed".to_string()
        } else {
            format!(
                "table_context_parsed_with_issues; truncated_columns={truncated_column_count}; truncated_rows={truncated_row_count}; omitted_values={omitted_value_count}"
            )
        };

        Ok(TableContextParseReport {
            context: Self { columns, rows },
            declared_column_count,
            parsed_column_count,
            declared_row_count,
            parsed_row_count,
            row_width,
            truncated_column_count,
            truncated_row_count,
            omitted_value_count,
            selected_column_count,
            plausible_column_count,
            unknown_column_count,
            selected_value_count,
            plausible_value_count,
            unknown_value_count,
            status,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::TableContext;

    #[test]
    fn reports_table_parse_diagnostics() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&2u16.to_le_bytes());
        buf.extend_from_slice(&4u16.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&0x0037_001fu32.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&4u16.to_le_bytes());
        buf.extend_from_slice(&[1, 2, 3, 4]);

        let report = TableContext::parse_with_report(&buf, 0).unwrap();
        assert_eq!(report.declared_column_count, 1);
        assert_eq!(report.parsed_column_count, 1);
        assert_eq!(report.declared_row_count, 2);
        assert_eq!(report.parsed_row_count, 1);
        assert_eq!(report.truncated_row_count, 1);
        assert_eq!(report.omitted_value_count, 0);
        assert_eq!(report.selected_column_count, 1);
        assert_eq!(report.selected_value_count, 1);
        assert!(report.status.contains("truncated_rows=1"));
        assert_eq!(report.context.rows[0].values[0].1, vec![1, 2, 3, 4]);
    }

    #[test]
    fn reports_omitted_values() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&4u16.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&0x0037_001fu32.to_le_bytes());
        buf.extend_from_slice(&2u16.to_le_bytes());
        buf.extend_from_slice(&4u16.to_le_bytes());
        buf.extend_from_slice(&[1, 2, 3, 4]);

        let report = TableContext::parse_with_report(&buf, 0).unwrap();
        assert_eq!(report.parsed_row_count, 1);
        assert_eq!(report.omitted_value_count, 1);
        assert!(report.context.rows[0].values.is_empty());
    }

    #[test]
    fn separates_plausible_and_unknown_columns() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&2u16.to_le_bytes());
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&8u16.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&0x9000_001fu32.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&4u16.to_le_bytes());
        buf.extend_from_slice(&0x9000_9999u32.to_le_bytes());
        buf.extend_from_slice(&4u16.to_le_bytes());
        buf.extend_from_slice(&4u16.to_le_bytes());
        buf.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);

        let report = TableContext::parse_with_report(&buf, 0).unwrap();
        assert_eq!(report.selected_column_count, 0);
        assert_eq!(report.plausible_column_count, 1);
        assert_eq!(report.unknown_column_count, 1);
        assert_eq!(report.plausible_value_count, 1);
        assert_eq!(report.unknown_value_count, 1);
    }
}
