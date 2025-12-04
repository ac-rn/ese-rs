//! Table metadata structures.

use crate::constants::ColumnType;
use indexmap::IndexMap;

/// Information about a table in the database.
#[derive(Debug, Clone)]
pub struct TableInfo {
    pub name: Vec<u8>,
    pub father_data_page_number: u32,
    pub space_usage: u32,
    pub columns: IndexMap<Vec<u8>, ColumnInfo>,
    pub indexes: IndexMap<Vec<u8>, IndexInfo>,
    pub long_values: IndexMap<Vec<u8>, LongValueInfo>,
}

impl TableInfo {
    /// Creates a new empty table info.
    pub fn new(name: Vec<u8>, father_data_page_number: u32, space_usage: u32) -> Self {
        TableInfo {
            name,
            father_data_page_number,
            space_usage,
            columns: IndexMap::new(),
            indexes: IndexMap::new(),
            long_values: IndexMap::new(),
        }
    }

    /// Returns the table name as a string (lossy conversion).
    pub fn name_str(&self) -> String {
        String::from_utf8_lossy(&self.name).to_string()
    }
}

/// Information about a column in a table.
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: Vec<u8>,
    pub identifier: u32,
    pub column_type: ColumnType,
    pub space_usage: u32,
    pub flags: u32,
    pub code_page: Option<u32>,
}

impl ColumnInfo {
    /// Returns the column name as a string (lossy conversion).
    pub fn name_str(&self) -> String {
        String::from_utf8_lossy(&self.name).to_string()
    }
}

/// Information about an index in a table.
#[derive(Debug, Clone)]
pub struct IndexInfo {
    pub name: Vec<u8>,
    pub father_data_page_number: u32,
    pub space_usage: u32,
    pub flags: u32,
    pub locale: u32,
}

impl IndexInfo {
    /// Returns the index name as a string (lossy conversion).
    pub fn name_str(&self) -> String {
        String::from_utf8_lossy(&self.name).to_string()
    }
}

/// Information about a long value in a table.
#[derive(Debug, Clone)]
pub struct LongValueInfo {
    pub name: Vec<u8>,
    pub father_data_page_number: u32,
    pub space_usage: u32,
}

impl LongValueInfo {
    /// Returns the long value name as a string (lossy conversion).
    pub fn name_str(&self) -> String {
        String::from_utf8_lossy(&self.name).to_string()
    }
}
