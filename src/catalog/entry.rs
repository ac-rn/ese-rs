//! Catalog entry structures.

use crate::constants::{CatalogType, ColumnType};
use crate::error::{EseError, Result};
use zerocopy::{AsBytes, FromBytes, FromZeroes};

/// Data definition header for all entries.
#[derive(Debug, Clone, Copy, AsBytes, FromBytes, FromZeroes, Default)]
#[repr(C, packed)]
pub struct DataDefinitionHeader {
    pub last_fixed_size: u8,
    pub last_variable_data_type: u8,
    pub variable_size_offset: u16,
}

impl DataDefinitionHeader {
    pub const SIZE: usize = std::mem::size_of::<Self>();

    /// Parses a data definition header from tag data (STRICT - for catalog parsing).
    /// This version fails if the header is malformed.
    /// Use `from_bytes_lenient` for data record parsing.
    pub fn from_bytes(data: &[u8]) -> Result<&Self> {
        if data.len() < Self::SIZE {
            return Err(EseError::InvalidDataDefinition);
        }
        DataDefinitionHeader::ref_from_prefix(data).ok_or(EseError::InvalidDataDefinition)
    }

    /// Parses a data definition header leniently (for data record parsing).
    /// Like Python's Structure class, we use defaults (0) for missing fields.
    /// This allows parsing of malformed records that Python handles.
    pub fn from_bytes_lenient(data: &[u8]) -> Self {
        if data.len() >= Self::SIZE {
            // Try to parse normally
            if let Some(header) = DataDefinitionHeader::ref_from_prefix(data) {
                return *header;
            }
        }
        // Use defaults if parsing fails (like Python's =0 defaults)
        Self::default()
    }
}

/// Fixed portion of catalog data definition entry.
#[derive(Debug, Clone, Copy, AsBytes, FromBytes, FromZeroes)]
#[repr(C, packed)]
struct CatalogFixed {
    father_data_page_id: u32,
    entry_type: u16,
    identifier: u32,
}

/// Column-specific fields.
#[derive(Debug, Clone, Copy, AsBytes, FromBytes, FromZeroes)]
#[repr(C, packed)]
struct ColumnFields {
    column_type: u32,
    space_usage: u32,
    column_flags: u32,
    code_page: u32,
}

/// Table/Index/LV shared fields.
#[derive(Debug, Clone, Copy, AsBytes, FromBytes, FromZeroes)]
#[repr(C, packed)]
struct OtherFields {
    father_data_page_number: u32,
}

/// Space usage field (for non-column types).
#[derive(Debug, Clone, Copy, AsBytes, FromBytes, FromZeroes)]
#[repr(C, packed)]
struct SpaceUsageField {
    space_usage: u32,
}

/// Index-specific fields.
#[derive(Debug, Clone, Copy, AsBytes, FromBytes, FromZeroes)]
#[repr(C, packed)]
struct IndexFields {
    index_flags: u32,
    locale: u32,
}

/// Catalog data definition entry (parsed variant).
#[derive(Debug, Clone)]
pub struct CatalogDataDefinitionEntry {
    pub father_data_page_id: u32,
    pub entry_type: CatalogType,
    pub identifier: u32,
    pub details: CatalogEntryDetails,
}

#[derive(Debug, Clone)]
pub enum CatalogEntryDetails {
    Table {
        father_data_page_number: u32,
        space_usage: u32,
    },
    Column {
        column_type: ColumnType,
        space_usage: u32,
        column_flags: u32,
        code_page: u32,
    },
    Index {
        father_data_page_number: u32,
        space_usage: u32,
        index_flags: u32,
        locale: u32,
    },
    LongValue {
        father_data_page_number: u32,
        space_usage: u32,
    },
}

impl CatalogDataDefinitionEntry {
    /// Parses a catalog entry from tag data (after the data definition header).
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut offset = 0;

        // Parse fixed portion
        if data.len() < offset + std::mem::size_of::<CatalogFixed>() {
            return Err(EseError::InvalidCatalogEntry);
        }

        let fixed =
            CatalogFixed::ref_from_prefix(&data[offset..]).ok_or(EseError::InvalidCatalogEntry)?;
        offset += std::mem::size_of::<CatalogFixed>();

        let entry_type = CatalogType::from_u16(fixed.entry_type)
            .ok_or(EseError::UnsupportedCatalogType(fixed.entry_type))?;

        // Parse type-specific fields
        let details = match entry_type {
            CatalogType::Table => {
                if data.len() < offset + std::mem::size_of::<OtherFields>() {
                    return Err(EseError::InvalidCatalogEntry);
                }
                let other = OtherFields::ref_from_prefix(&data[offset..])
                    .ok_or(EseError::InvalidCatalogEntry)?;
                offset += std::mem::size_of::<OtherFields>();

                if data.len() < offset + std::mem::size_of::<SpaceUsageField>() {
                    return Err(EseError::InvalidCatalogEntry);
                }
                let space = SpaceUsageField::ref_from_prefix(&data[offset..])
                    .ok_or(EseError::InvalidCatalogEntry)?;

                CatalogEntryDetails::Table {
                    father_data_page_number: other.father_data_page_number,
                    space_usage: space.space_usage,
                }
            }

            CatalogType::Column => {
                if data.len() < offset + std::mem::size_of::<ColumnFields>() {
                    return Err(EseError::InvalidCatalogEntry);
                }
                let col_fields = ColumnFields::ref_from_prefix(&data[offset..])
                    .ok_or(EseError::InvalidCatalogEntry)?;

                let column_type = ColumnType::from_u32(col_fields.column_type)
                    .ok_or(EseError::UnsupportedColumnType(col_fields.column_type))?;

                CatalogEntryDetails::Column {
                    column_type,
                    space_usage: col_fields.space_usage,
                    column_flags: col_fields.column_flags,
                    code_page: col_fields.code_page,
                }
            }

            CatalogType::Index => {
                if data.len() < offset + std::mem::size_of::<OtherFields>() {
                    return Err(EseError::InvalidCatalogEntry);
                }
                let other = OtherFields::ref_from_prefix(&data[offset..])
                    .ok_or(EseError::InvalidCatalogEntry)?;
                offset += std::mem::size_of::<OtherFields>();

                if data.len() < offset + std::mem::size_of::<SpaceUsageField>() {
                    return Err(EseError::InvalidCatalogEntry);
                }
                let space = SpaceUsageField::ref_from_prefix(&data[offset..])
                    .ok_or(EseError::InvalidCatalogEntry)?;
                offset += std::mem::size_of::<SpaceUsageField>();

                if data.len() < offset + std::mem::size_of::<IndexFields>() {
                    return Err(EseError::InvalidCatalogEntry);
                }
                let idx_fields = IndexFields::ref_from_prefix(&data[offset..])
                    .ok_or(EseError::InvalidCatalogEntry)?;

                CatalogEntryDetails::Index {
                    father_data_page_number: other.father_data_page_number,
                    space_usage: space.space_usage,
                    index_flags: idx_fields.index_flags,
                    locale: idx_fields.locale,
                }
            }

            CatalogType::LongValue => {
                if data.len() < offset + std::mem::size_of::<OtherFields>() {
                    return Err(EseError::InvalidCatalogEntry);
                }
                let other = OtherFields::ref_from_prefix(&data[offset..])
                    .ok_or(EseError::InvalidCatalogEntry)?;
                offset += std::mem::size_of::<OtherFields>();

                if data.len() < offset + std::mem::size_of::<SpaceUsageField>() {
                    return Err(EseError::InvalidCatalogEntry);
                }
                let space = SpaceUsageField::ref_from_prefix(&data[offset..])
                    .ok_or(EseError::InvalidCatalogEntry)?;

                CatalogEntryDetails::LongValue {
                    father_data_page_number: other.father_data_page_number,
                    space_usage: space.space_usage,
                }
            }

            CatalogType::Callback => {
                return Err(EseError::CallbackUnsupported);
            }
        };

        Ok(CatalogDataDefinitionEntry {
            father_data_page_id: fixed.father_data_page_id,
            entry_type,
            identifier: fixed.identifier,
            details,
        })
    }

    /// Extracts the item name from the full entry data.
    pub fn extract_item_name(entry_data: &[u8]) -> Result<Vec<u8>> {
        let dd_header = DataDefinitionHeader::from_bytes(entry_data)?;

        let num_entries = if dd_header.last_variable_data_type > 127 {
            dd_header.last_variable_data_type - 127
        } else {
            dd_header.last_variable_data_type
        };

        let var_offset = dd_header.variable_size_offset as usize;
        if entry_data.len() < var_offset + 2 {
            return Err(EseError::InvalidCatalogEntry);
        }

        let item_len =
            u16::from_le_bytes([entry_data[var_offset], entry_data[var_offset + 1]]) as usize;

        let name_offset = var_offset + (2 * num_entries as usize);
        if entry_data.len() < name_offset + item_len {
            return Err(EseError::InvalidCatalogEntry);
        }

        Ok(entry_data[name_offset..name_offset + item_len].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_definition_header_size() {
        assert_eq!(DataDefinitionHeader::SIZE, 4);
    }
}
