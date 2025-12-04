//! Error types for ESE database parsing.

use thiserror::Error;

/// Result type alias for ESE operations.
pub type Result<T> = std::result::Result<T, EseError>;

/// Errors that can occur during ESE database parsing.
#[derive(Error, Debug)]
pub enum EseError {
    #[error("Invalid database header")]
    InvalidHeader,

    #[error("Invalid signature, expected 0x89ABCDEF, got {0:#x}")]
    InvalidSignature(u32),

    #[error("Unsupported database version: {version:#x}, revision: {revision:#x}")]
    UnsupportedVersion { version: u32, revision: u32 },

    #[error("Invalid page number: {0}")]
    InvalidPageNumber(u32),

    #[error("Invalid tag number: {0}")]
    InvalidTagNumber(u16),

    #[error("Table not found: {0:?}")]
    TableNotFound(Vec<u8>),

    #[error("Column not found: {0:?}")]
    ColumnNotFound(Vec<u8>),

    #[error("Unsupported catalog type: {0}")]
    UnsupportedCatalogType(u16),

    #[error("Unsupported column type: {0}")]
    UnsupportedColumnType(u32),

    #[error("Compressed tagged data is not supported")]
    CompressedDataUnsupported,

    #[error("Long value data is not yet supported")]
    LongValueUnsupported,

    #[error("Multi-value data is not fully supported")]
    MultiValueUnsupported,

    #[error("Callback catalog type is not supported")]
    CallbackUnsupported,

    #[error("Invalid data definition header")]
    InvalidDataDefinition,

    #[error("Invalid catalog entry")]
    InvalidCatalogEntry,

    #[error("Page data too short, expected at least {expected} bytes, got {actual}")]
    PageDataTooShort { expected: usize, actual: usize },

    #[error("Tag offset out of bounds: offset={offset}, size={size}, page_size={page_size}")]
    TagOffsetOutOfBounds {
        offset: usize,
        size: usize,
        page_size: usize,
    },

    #[error("Unknown codepage: {0}")]
    UnknownCodepage(u32),

    #[error("String decode error: {0}")]
    StringDecode(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Alignment error: {0}")]
    Alignment(String),
}

impl From<encoding_rs::Encoding> for EseError {
    fn from(_: encoding_rs::Encoding) -> Self {
        EseError::StringDecode("Encoding error".to_string())
    }
}
