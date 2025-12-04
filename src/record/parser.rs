//! Record parser for extracting column values from entry data.

use crate::catalog::entry::DataDefinitionHeader;
use crate::catalog::table_info::{ColumnInfo, TableInfo};
use crate::constants::{is_large_page_format, tagged_data_flags, ColumnType};
use crate::error::{EseError, Result};
use crate::header::DbHeader;
use crate::types::ColumnValue;
use crate::utils::decode_string;
use indexmap::IndexMap;

/// Parser for extracting record values from tag entry data.
pub struct RecordParser<'a> {
    tag_data: &'a [u8],
    table_info: &'a TableInfo,
    db_header: &'a DbHeader,
}

impl<'a> RecordParser<'a> {
    /// Creates a new record parser.
    pub fn new(tag_data: &'a [u8], table_info: &'a TableInfo, db_header: &'a DbHeader) -> Self {
        RecordParser {
            tag_data,
            table_info,
            db_header,
        }
    }

    /// Parses the record and returns column values.
    ///
    /// The record structure has three tiers:
    /// 1. Fixed-size data (identifier <= 127)
    /// 2. Variable-size data (127 < identifier <= 255)
    /// 3. Tagged data (identifier > 255)
    ///
    /// Like Python's lenient parsing, we try to extract what we can
    /// and use NULL for columns we can't parse.
    pub fn parse_record(&self) -> Result<IndexMap<Vec<u8>, ColumnValue>> {
        // Parse the data definition header LENIENTLY for data records
        // Use defaults like Python's Structure class if header is malformed
        let dd_header = DataDefinitionHeader::from_bytes_lenient(self.tag_data);

        let mut record = IndexMap::new();
        let mut fixed_offset = DataDefinitionHeader::SIZE;

        // Parse tagged items if needed (lazy parsing)
        let mut tagged_items: Option<IndexMap<u32, TaggedItem>> = None;

        // CRITICAL: Columns must be parsed in IDENTIFIER ORDER because fixed columns
        // are laid out sequentially in the record data (id=1, then id=2, etc.)
        let mut sorted_columns: Vec<_> = self.table_info.columns.iter().collect();
        sorted_columns.sort_by_key(|(_, col_info)| col_info.identifier);

        for (col_name, col_info) in sorted_columns {
            // Be lenient like Python - if parsing any column fails, just use NULL
            let value = if col_info.identifier <= dd_header.last_fixed_size as u32 {
                // Fixed-size column
                self.parse_fixed_column(&mut fixed_offset, col_info)
                    .unwrap_or(ColumnValue::Null)
            } else if col_info.identifier > 127
                && col_info.identifier <= dd_header.last_variable_data_type as u32
            {
                // Variable-size column
                self.parse_variable_column(&dd_header, col_info)
                    .unwrap_or(ColumnValue::Null)
            } else if col_info.identifier > 255 {
                // Tagged column (lazy parse tagged items)
                if tagged_items.is_none() {
                    // Even if parsing tagged items fails, just use empty map
                    tagged_items = Some(
                        self.parse_tagged_items(&dd_header)
                            .unwrap_or_else(|_| IndexMap::new()),
                    );
                }
                self.parse_tagged_column(col_info, tagged_items.as_ref().unwrap())
                    .unwrap_or(ColumnValue::Null)
            } else {
                ColumnValue::Null
            };

            record.insert(col_name.clone(), value);
        }

        Ok(record)
    }

    /// Parses a fixed-size column value.
    fn parse_fixed_column(
        &self,
        fixed_offset: &mut usize,
        col_info: &ColumnInfo,
    ) -> Result<ColumnValue> {
        let size = col_info.space_usage as usize;

        // Check if we have enough data
        if *fixed_offset + size > self.tag_data.len() {
            // Not enough data - column is NULL or not present
            *fixed_offset += size; // Still advance offset to stay aligned
            return Ok(ColumnValue::Null);
        }

        let data = &self.tag_data[*fixed_offset..*fixed_offset + size];
        *fixed_offset += size;

        self.parse_column_value(data, col_info)
    }

    /// Parses a variable-size column value.
    fn parse_variable_column(
        &self,
        dd_header: &DataDefinitionHeader,
        col_info: &ColumnInfo,
    ) -> Result<ColumnValue> {
        // Handle case where last_variable_data_type < 127
        if dd_header.last_variable_data_type < 127 {
            return Ok(ColumnValue::Null);
        }
        let num_var_entries = dd_header.last_variable_data_type - 127;
        let var_offset = dd_header.variable_size_offset as usize;
        let index = (col_info.identifier - 128) as usize;

        if index >= num_var_entries as usize {
            return Ok(ColumnValue::Null);
        }

        // Read variable size offset array
        let offset_pos = var_offset + index * 2;
        if self.tag_data.len() < offset_pos + 2 {
            return Ok(ColumnValue::Null);
        }

        let item_len_raw = self
            .tag_data
            .get(offset_pos..offset_pos + 2)
            .and_then(|bytes| <[u8; 2]>::try_from(bytes).ok())
            .map(u16::from_le_bytes)
            .ok_or(EseError::PageDataTooShort {
                expected: offset_pos + 2,
                actual: self.tag_data.len(),
            })?;

        // Check for NULL marker (0x8000 bit set)
        if item_len_raw & 0x8000 != 0 {
            return Ok(ColumnValue::Null);
        }

        let item_len = item_len_raw as usize;

        // Calculate previous item length for offset calculation
        let prev_item_len = if index > 0 {
            let prev_offset_pos = var_offset + (index - 1) * 2;
            let prev_len = self
                .tag_data
                .get(prev_offset_pos..prev_offset_pos + 2)
                .and_then(|bytes| <[u8; 2]>::try_from(bytes).ok())
                .map(u16::from_le_bytes)
                .unwrap_or(0);
            (prev_len & 0x7fff) as usize
        } else {
            0
        };

        let data_offset = var_offset + num_var_entries as usize * 2 + prev_item_len;
        let data_size = item_len - prev_item_len;

        if self.tag_data.len() < data_offset + data_size {
            return Ok(ColumnValue::Null);
        }

        let data = &self.tag_data[data_offset..data_offset + data_size];

        self.parse_column_value(data, col_info)
    }

    /// Parses a tagged column value.
    fn parse_tagged_column(
        &self,
        col_info: &ColumnInfo,
        tagged_items: &IndexMap<u32, TaggedItem>,
    ) -> Result<ColumnValue> {
        if let Some(item) = tagged_items.get(&col_info.identifier) {
            if item.flags & tagged_data_flags::COMPRESSED != 0 {
                return Err(EseError::CompressedDataUnsupported);
            }

            if item.flags & tagged_data_flags::MULTI_VALUE != 0 {
                // Multi-value data - return as raw binary for now
                return Ok(ColumnValue::MultiValue(item.data.clone()));
            }

            self.parse_column_value(&item.data, col_info)
        } else {
            Ok(ColumnValue::Null)
        }
    }

    /// Parses tagged items array from the record.
    fn parse_tagged_items(
        &self,
        dd_header: &DataDefinitionHeader,
    ) -> Result<IndexMap<u32, TaggedItem>> {
        // Calculate number of variable columns
        // If last_variable_data_type <= 127, there are no variable columns
        // But there might still be tagged columns!
        let num_var_entries = if dd_header.last_variable_data_type > 127 {
            dd_header.last_variable_data_type - 127
        } else {
            0
        };
        let var_offset = dd_header.variable_size_offset as usize;

        let variable_data_bytes_processed = {
            // Calculate total variable data size
            if num_var_entries == 0 {
                0
            } else {
                let last_offset_pos = var_offset + (num_var_entries as usize - 1) * 2;
                if self.tag_data.len() < last_offset_pos + 2 {
                    0
                } else {
                    let last_len = self
                        .tag_data
                        .get(last_offset_pos..last_offset_pos + 2)
                        .and_then(|bytes| <[u8; 2]>::try_from(bytes).ok())
                        .map(u16::from_le_bytes)
                        .unwrap_or(0);
                    (last_len & 0x7fff) as usize
                }
            }
        };

        let tagged_start =
            var_offset + num_var_entries as usize * 2 + variable_data_bytes_processed;

        if tagged_start >= self.tag_data.len() {
            return Ok(IndexMap::new());
        }

        let mut tagged_items = IndexMap::new();
        let mut index = tagged_start;

        // Check for flags presence based on version
        let flags_present = is_large_page_format(
            self.db_header.version(),
            self.db_header.file_format_revision(),
            self.db_header.page_size(),
        );

        // Read tagged item headers
        let mut tagged_headers = Vec::new();

        // Calculate first data offset to know when to stop reading headers
        let first_data_offset = if index + 4 <= self.tag_data.len() {
            let first_offset_raw = self
                .tag_data
                .get(index + 2..index + 4)
                .and_then(|bytes| <[u8; 2]>::try_from(bytes).ok())
                .map(u16::from_le_bytes)
                .unwrap_or(0);
            let first_offset = (first_offset_raw & 0x3fff) as usize;
            tagged_start + first_offset
        } else {
            self.tag_data.len()
        };

        loop {
            if index + 4 > self.tag_data.len() {
                break;
            }

            if index >= first_data_offset {
                break;
            }

            let identifier = self
                .tag_data
                .get(index..index + 2)
                .and_then(|bytes| <[u8; 2]>::try_from(bytes).ok())
                .map(u16::from_le_bytes)
                .unwrap_or(0) as u32;
            index += 2;

            let offset_raw = self
                .tag_data
                .get(index..index + 2)
                .and_then(|bytes| <[u8; 2]>::try_from(bytes).ok())
                .map(u16::from_le_bytes)
                .unwrap_or(0);
            let offset = (offset_raw & 0x3fff) as usize;
            let has_flags = if flags_present {
                true
            } else {
                offset_raw & 0x4000 != 0
            };
            index += 2;

            tagged_headers.push((identifier, offset, has_flags));
        }

        // Calculate sizes and extract data
        for i in 0..tagged_headers.len() {
            let (identifier, offset, has_flags) = tagged_headers[i];
            let next_offset = if i + 1 < tagged_headers.len() {
                tagged_headers[i + 1].1
            } else {
                self.tag_data.len() - tagged_start
            };

            // Handle case where offset > next_offset (shouldn't happen but be defensive)
            if offset > next_offset {
                continue;
            }

            let size = next_offset - offset;
            let data_start = tagged_start + offset;

            if data_start + size > self.tag_data.len() {
                continue;
            }

            let mut data = self.tag_data[data_start..data_start + size].to_vec();
            let flags = if has_flags && !data.is_empty() {
                let f = data[0];
                data.remove(0);
                f
            } else {
                0
            };

            tagged_items.insert(identifier, TaggedItem { data, flags });
        }

        Ok(tagged_items)
    }

    /// Parses column data into a ColumnValue based on the column type.
    fn parse_column_value(&self, data: &[u8], col_info: &ColumnInfo) -> Result<ColumnValue> {
        if data.is_empty() {
            return Ok(ColumnValue::Null);
        }

        match col_info.column_type {
            ColumnType::Nil => Ok(ColumnValue::Null),
            ColumnType::Bit => Ok(ColumnValue::Boolean(data[0] != 0)),
            ColumnType::UnsignedByte => {
                // Handle mismatched type/size - some databases mark 8-byte columns as UnsignedByte
                match data.len() {
                    1 => Ok(ColumnValue::U8(data[0])),
                    2 => {
                        let val = u16::from_le_bytes([data[0], data[1]]);
                        Ok(ColumnValue::U16(val))
                    }
                    4 => {
                        let val = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                        Ok(ColumnValue::U32(val))
                    }
                    8 => {
                        // Parse as unsigned 64-bit to match Python
                        let val = u64::from_le_bytes([
                            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
                        ]);
                        Ok(ColumnValue::U64(val))
                    }
                    _ => Ok(ColumnValue::Binary(data.to_vec())),
                }
            }
            ColumnType::Short => {
                let val = i16::from_le_bytes([data[0], data[1]]);
                Ok(ColumnValue::I16(val))
            }
            ColumnType::Long => {
                let val = i32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                Ok(ColumnValue::I32(val))
            }
            ColumnType::Currency => {
                let val = u64::from_le_bytes([
                    data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
                ]);
                Ok(ColumnValue::Currency(val))
            }
            ColumnType::IEEESingle => {
                let val = f32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                Ok(ColumnValue::F32(val))
            }
            ColumnType::IEEEDouble => {
                let val = f64::from_le_bytes([
                    data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
                ]);
                Ok(ColumnValue::F64(val))
            }
            ColumnType::DateTime => {
                let val = u64::from_le_bytes([
                    data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
                ]);
                Ok(ColumnValue::DateTime(val))
            }
            ColumnType::Binary => Ok(ColumnValue::Binary(data.to_vec())),
            ColumnType::Text | ColumnType::LongText => {
                let codepage = col_info.code_page.unwrap_or(1252);
                let text = decode_string(data, codepage)?;
                Ok(ColumnValue::Text(text))
            }
            ColumnType::LongBinary => Ok(ColumnValue::LongValue(data.to_vec())),
            ColumnType::SLV => Ok(ColumnValue::Null), // Obsolete
            ColumnType::UnsignedLong => {
                let val = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                Ok(ColumnValue::U32(val))
            }
            ColumnType::LongLong => {
                // Python treats LongLong as unsigned (<Q), not signed
                let val = u64::from_le_bytes([
                    data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
                ]);
                Ok(ColumnValue::U64(val))
            }
            ColumnType::GUID => {
                let mut guid = [0u8; 16];
                guid.copy_from_slice(&data[..16]);
                Ok(ColumnValue::Guid(guid))
            }
            ColumnType::UnsignedShort => {
                let val = u16::from_le_bytes([data[0], data[1]]);
                Ok(ColumnValue::U16(val))
            }
            ColumnType::Max => Ok(ColumnValue::Null),
        }
    }
}

/// Represents a tagged data item.
struct TaggedItem {
    data: Vec<u8>,
    flags: u8,
}
