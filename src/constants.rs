//! Constants and enumerations from the ESE specification.

/// Database file types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum FileType {
    Database = 0,
    StreamingFile = 1,
}

/// Database state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DbState {
    JustCreated = 1,
    DirtyShutdown = 2,
    CleanShutdown = 3,
    BeingConverted = 4,
    ForceDetach = 5,
}

/// Page flags.
pub mod page_flags {
    pub const ROOT: u32 = 0x0001;
    pub const LEAF: u32 = 0x0002;
    pub const PARENT: u32 = 0x0004;
    pub const EMPTY: u32 = 0x0008;
    pub const SPACE_TREE: u32 = 0x0020;
    pub const INDEX: u32 = 0x0040;
    pub const LONG_VALUE: u32 = 0x0080;
    pub const NEW_FORMAT: u32 = 0x2000;
    pub const NEW_CHECKSUM: u32 = 0x2000;
}

/// Tag flags.
pub mod tag_flags {
    pub const UNKNOWN: u8 = 0x1;
    pub const DEFUNCT: u8 = 0x2;
    pub const COMMON: u8 = 0x4;
}

/// Fixed page numbers.
pub const DATABASE_PAGE_NUMBER: u32 = 1;
pub const CATALOG_PAGE_NUMBER: u32 = 4;
pub const CATALOG_BACKUP_PAGE_NUMBER: u32 = 24;

/// Fixed Father Data Page numbers.
pub const DATABASE_FDP: u32 = 1;
pub const CATALOG_FDP: u32 = 2;
pub const CATALOG_BACKUP_FDP: u32 = 3;

/// Catalog entry types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum CatalogType {
    Table = 1,
    Column = 2,
    Index = 3,
    LongValue = 4,
    Callback = 5,
}

impl CatalogType {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            1 => Some(CatalogType::Table),
            2 => Some(CatalogType::Column),
            3 => Some(CatalogType::Index),
            4 => Some(CatalogType::LongValue),
            5 => Some(CatalogType::Callback),
            _ => None,
        }
    }
}

/// Column data types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ColumnType {
    Nil = 0,
    Bit = 1,
    UnsignedByte = 2,
    Short = 3,
    Long = 4,
    Currency = 5,
    IEEESingle = 6,
    IEEEDouble = 7,
    DateTime = 8,
    Binary = 9,
    Text = 10,
    LongBinary = 11,
    LongText = 12,
    SLV = 13,
    UnsignedLong = 14,
    LongLong = 15,
    GUID = 16,
    UnsignedShort = 17,
    Max = 18,
}

impl ColumnType {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(ColumnType::Nil),
            1 => Some(ColumnType::Bit),
            2 => Some(ColumnType::UnsignedByte),
            3 => Some(ColumnType::Short),
            4 => Some(ColumnType::Long),
            5 => Some(ColumnType::Currency),
            6 => Some(ColumnType::IEEESingle),
            7 => Some(ColumnType::IEEEDouble),
            8 => Some(ColumnType::DateTime),
            9 => Some(ColumnType::Binary),
            10 => Some(ColumnType::Text),
            11 => Some(ColumnType::LongBinary),
            12 => Some(ColumnType::LongText),
            13 => Some(ColumnType::SLV),
            14 => Some(ColumnType::UnsignedLong),
            15 => Some(ColumnType::LongLong),
            16 => Some(ColumnType::GUID),
            17 => Some(ColumnType::UnsignedShort),
            18 => Some(ColumnType::Max),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ColumnType::Nil => "NULL",
            ColumnType::Bit => "Boolean",
            ColumnType::UnsignedByte => "Unsigned byte",
            ColumnType::Short => "Signed short",
            ColumnType::Long => "Signed long",
            ColumnType::Currency => "Currency",
            ColumnType::IEEESingle => "Single precision FP",
            ColumnType::IEEEDouble => "Double precision FP",
            ColumnType::DateTime => "DateTime",
            ColumnType::Binary => "Binary",
            ColumnType::Text => "Text",
            ColumnType::LongBinary => "Long Binary",
            ColumnType::LongText => "Long Text",
            ColumnType::SLV => "Obsolete",
            ColumnType::UnsignedLong => "Unsigned long",
            ColumnType::LongLong => "Long long",
            ColumnType::GUID => "GUID",
            ColumnType::UnsignedShort => "Unsigned short",
            ColumnType::Max => "Max",
        }
    }

    /// Returns the fixed size and format for this column type, if applicable.
    pub fn fixed_size(&self) -> Option<usize> {
        match self {
            ColumnType::Nil => None,
            ColumnType::Bit => Some(1),
            ColumnType::UnsignedByte => Some(1),
            ColumnType::Short => Some(2),
            ColumnType::Long => Some(4),
            ColumnType::Currency => Some(8),
            ColumnType::IEEESingle => Some(4),
            ColumnType::IEEEDouble => Some(8),
            ColumnType::DateTime => Some(8),
            ColumnType::Binary => None,
            ColumnType::Text => None,
            ColumnType::LongBinary => None,
            ColumnType::LongText => None,
            ColumnType::SLV => None,
            ColumnType::UnsignedLong => Some(4),
            ColumnType::LongLong => Some(8),
            ColumnType::GUID => Some(16),
            ColumnType::UnsignedShort => Some(2),
            ColumnType::Max => None,
        }
    }
}

/// Tagged data type flags.
pub mod tagged_data_flags {
    pub const VARIABLE_SIZE: u8 = 0x01;
    pub const COMPRESSED: u8 = 0x02;
    pub const STORED: u8 = 0x04;
    pub const MULTI_VALUE: u8 = 0x08;
    pub const WHO_KNOWS: u8 = 0x10;
}

/// Code pages for string encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum CodePage {
    Unicode = 1200,
    Ascii = 20127,
    Western = 1252,
}

impl CodePage {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            1200 => Some(CodePage::Unicode),
            20127 => Some(CodePage::Ascii),
            1252 => Some(CodePage::Western),
            _ => None,
        }
    }

    pub fn encoding_name(&self) -> &'static str {
        match self {
            CodePage::Unicode => "utf-16le",
            CodePage::Ascii => "ascii",
            CodePage::Western => "windows-1252",
        }
    }
}

/// Database header magic signature.
pub const DB_HEADER_SIGNATURE: [u8; 4] = [0xef, 0xcd, 0xab, 0x89];

// Page format constants
pub const PAGE_HEADER_TAG_OFFSET: usize = 34;
pub const ESE_VERSION_WIN7: u32 = 0x620;
pub const ESE_REVISION_LARGE_PAGE: u32 = 17;
pub const LARGE_PAGE_THRESHOLD: u32 = 8192;

// Size limits
pub const MAX_TAG_SIZE: usize = 10 * 1024 * 1024; // 10MB
pub const MAX_TABLE_NAME_LENGTH: usize = 255;
pub const MIN_DATABASE_SIZE: usize = 8192;

/// Helper function to determine if database uses large page format.
#[inline]
pub fn is_large_page_format(version: u32, revision: u32, page_size: u32) -> bool {
    version == ESE_VERSION_WIN7
        && revision >= ESE_REVISION_LARGE_PAGE
        && page_size > LARGE_PAGE_THRESHOLD
}
