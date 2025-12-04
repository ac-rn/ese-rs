//! Column value types representing parsed database values.

use std::fmt;

/// Represents a parsed column value from the database.
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnValue {
    /// NULL value
    Null,
    /// Boolean (1 byte)
    Boolean(bool),
    /// Signed 8-bit integer
    I8(i8),
    /// Unsigned 8-bit integer
    U8(u8),
    /// Signed 16-bit integer
    I16(i16),
    /// Unsigned 16-bit integer
    U16(u16),
    /// Signed 32-bit integer
    I32(i32),
    /// Unsigned 32-bit integer
    U32(u32),
    /// Signed 64-bit integer
    I64(i64),
    /// Unsigned 64-bit integer
    U64(u64),
    /// 32-bit floating point
    F32(f32),
    /// 64-bit floating point
    F64(f64),
    /// DateTime (64-bit timestamp)
    DateTime(u64),
    /// Currency (64-bit)
    Currency(u64),
    /// GUID (16 bytes)
    Guid([u8; 16]),
    /// Binary data
    Binary(Vec<u8>),
    /// Text string
    Text(String),
    /// Multi-value data (not fully parsed)
    MultiValue(Vec<u8>),
    /// Long value reference (not fully implemented)
    LongValue(Vec<u8>),
}

impl ColumnValue {
    /// Returns true if the value is NULL.
    pub fn is_null(&self) -> bool {
        matches!(self, ColumnValue::Null)
    }

    /// Converts the value to a string representation.
    pub fn to_string_lossy(&self) -> String {
        match self {
            ColumnValue::Null => "NULL".to_string(),
            ColumnValue::Boolean(b) => b.to_string(),
            ColumnValue::I8(v) => v.to_string(),
            ColumnValue::U8(v) => v.to_string(),
            ColumnValue::I16(v) => v.to_string(),
            ColumnValue::U16(v) => v.to_string(),
            ColumnValue::I32(v) => v.to_string(),
            ColumnValue::U32(v) => v.to_string(),
            ColumnValue::I64(v) => v.to_string(),
            ColumnValue::U64(v) => v.to_string(),
            ColumnValue::F32(v) => v.to_string(),
            ColumnValue::F64(v) => v.to_string(),
            ColumnValue::DateTime(v) => format!("DateTime({})", v),
            ColumnValue::Currency(v) => format!("Currency({})", v),
            ColumnValue::Guid(bytes) => {
                format!(
                    "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                    bytes[0], bytes[1], bytes[2], bytes[3],
                    bytes[4], bytes[5],
                    bytes[6], bytes[7],
                    bytes[8], bytes[9],
                    bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
                )
            }
            ColumnValue::Binary(data) => hex::encode(data),
            ColumnValue::Text(s) => s.clone(),
            ColumnValue::MultiValue(data) => format!("MultiValue({})", hex::encode(data)),
            ColumnValue::LongValue(data) => format!("LongValue({})", hex::encode(data)),
        }
    }
}

impl fmt::Display for ColumnValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_lossy())
    }
}
