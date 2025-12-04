//! Page header structures.

use crate::constants::page_flags;
use crate::error::{EseError, Result};
use zerocopy::{AsBytes, FromBytes, FromZeroes};

/// Common page header fields shared across all versions.
#[derive(Debug, Clone, Copy, AsBytes, FromBytes, FromZeroes)]
#[repr(C, packed)]
pub struct PageHeaderCommon {
    pub last_modification_time: u64,
    pub previous_page_number: u32,
    pub next_page_number: u32,
    pub father_data_page: u32,
    pub available_data_size: u16,
    pub available_uncommitted_data_size: u16,
    pub first_available_data_offset: u16,
    pub first_available_page_tag: u16,
    pub page_flags: u32,
}

/// Extended page header for Windows 7+ with large pages.
#[derive(Debug, Clone, Copy, AsBytes, FromBytes, FromZeroes)]
#[repr(C, packed)]
pub struct PageHeaderExtended {
    pub extended_checksum1: u64,
    pub extended_checksum2: u64,
    pub extended_checksum3: u64,
    pub page_number: u64,
    pub unknown: u64,
}

/// Page header variants for different ESE versions.
#[derive(Debug, Clone)]
pub enum PageHeader {
    /// Windows 2003 SP0 format
    Legacy2003 {
        checksum: u32,
        page_number: u32,
        common: PageHeaderCommon,
    },
    /// Windows Vista / Exchange 2003 SP1 format
    Vista {
        checksum: u32,
        ecc_checksum: u32,
        common: PageHeaderCommon,
    },
    /// Windows 7 and later format
    Win7 {
        checksum: u64,
        common: PageHeaderCommon,
        extended: Option<PageHeaderExtended>,
    },
}

impl PageHeader {
    /// Parses a page header from raw data.
    ///
    /// The format depends on the database version and revision:
    /// - version < 0x620 or (version == 0x620 && revision < 0x0b): Legacy 2003 format
    /// - version == 0x620 && revision < 0x11: Vista format
    /// - version >= 0x620 && revision >= 0x11: Win7 format
    ///
    /// Returns the parsed header and its size in bytes.
    pub fn parse(
        data: &[u8],
        version: u32,
        revision: u32,
        page_size: u32,
    ) -> Result<(Self, usize)> {
        if (version < 0x620) || (version == 0x620 && revision < 0x0b) {
            Self::parse_legacy_2003(data)
        } else if version == 0x620 && revision < 0x11 {
            Self::parse_vista(data)
        } else {
            Self::parse_win7(data, page_size)
        }
    }

    /// Parses a Windows 2003 SP0 format page header.
    fn parse_legacy_2003(data: &[u8]) -> Result<(Self, usize)> {
        const MIN_SIZE: usize = 4 + 4 + std::mem::size_of::<PageHeaderCommon>();

        if data.len() < MIN_SIZE {
            return Err(EseError::PageDataTooShort {
                expected: MIN_SIZE,
                actual: data.len(),
            });
        }

        let checksum = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let page_number = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);

        let common =
            PageHeaderCommon::ref_from_prefix(&data[8..]).ok_or(EseError::InvalidHeader)?;

        Ok((
            PageHeader::Legacy2003 {
                checksum,
                page_number,
                common: *common,
            },
            MIN_SIZE,
        ))
    }

    /// Parses a Windows Vista format page header.
    fn parse_vista(data: &[u8]) -> Result<(Self, usize)> {
        const MIN_SIZE: usize = 4 + 4 + std::mem::size_of::<PageHeaderCommon>();

        if data.len() < MIN_SIZE {
            return Err(EseError::PageDataTooShort {
                expected: MIN_SIZE,
                actual: data.len(),
            });
        }

        let checksum = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let ecc_checksum = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);

        let common =
            PageHeaderCommon::ref_from_prefix(&data[8..]).ok_or(EseError::InvalidHeader)?;

        Ok((
            PageHeader::Vista {
                checksum,
                ecc_checksum,
                common: *common,
            },
            MIN_SIZE,
        ))
    }

    /// Parses a Windows 7+ format page header.
    fn parse_win7(data: &[u8], page_size: u32) -> Result<(Self, usize)> {
        let mut offset = 0;

        const CHECKSUM_SIZE: usize = 8;
        const COMMON_SIZE: usize = std::mem::size_of::<PageHeaderCommon>();
        let min_size = CHECKSUM_SIZE + COMMON_SIZE;

        if data.len() < min_size {
            return Err(EseError::PageDataTooShort {
                expected: min_size,
                actual: data.len(),
            });
        }

        let checksum = u64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ]);
        offset += CHECKSUM_SIZE;

        let common =
            PageHeaderCommon::ref_from_prefix(&data[offset..]).ok_or(EseError::InvalidHeader)?;
        offset += COMMON_SIZE;

        // Check if we have extended header (for pages > 8KB)
        let extended = if page_size > 8192 {
            const EXTENDED_SIZE: usize = std::mem::size_of::<PageHeaderExtended>();
            if data.len() < offset + EXTENDED_SIZE {
                return Err(EseError::PageDataTooShort {
                    expected: offset + EXTENDED_SIZE,
                    actual: data.len(),
                });
            }

            let ext = PageHeaderExtended::ref_from_prefix(&data[offset..])
                .ok_or(EseError::InvalidHeader)?;
            offset += EXTENDED_SIZE;
            Some(*ext)
        } else {
            None
        };

        Ok((
            PageHeader::Win7 {
                checksum,
                common: *common,
                extended,
            },
            offset,
        ))
    }

    /// Returns the common header fields.
    pub fn common(&self) -> &PageHeaderCommon {
        match self {
            PageHeader::Legacy2003 { common, .. } => common,
            PageHeader::Vista { common, .. } => common,
            PageHeader::Win7 { common, .. } => common,
        }
    }

    /// Returns true if the ROOT flag is set.
    pub fn is_root(&self) -> bool {
        self.common().page_flags & page_flags::ROOT != 0
    }

    /// Returns true if the LEAF flag is set.
    pub fn is_leaf(&self) -> bool {
        self.common().page_flags & page_flags::LEAF != 0
    }

    /// Returns true if the SPACE_TREE flag is set.
    pub fn is_space_tree(&self) -> bool {
        self.common().page_flags & page_flags::SPACE_TREE != 0
    }

    /// Returns true if the INDEX flag is set.
    pub fn is_index(&self) -> bool {
        self.common().page_flags & page_flags::INDEX != 0
    }

    /// Returns true if the LONG_VALUE flag is set.
    pub fn is_long_value(&self) -> bool {
        self.common().page_flags & page_flags::LONG_VALUE != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_header_common_size() {
        assert_eq!(std::mem::size_of::<PageHeaderCommon>(), 32);
    }

    #[test]
    fn test_parse_legacy_2003() {
        let mut data = vec![0u8; 256];
        // Set checksum
        data[0..4].copy_from_slice(&0x12345678u32.to_le_bytes());
        // Set page number
        data[4..8].copy_from_slice(&42u32.to_le_bytes());

        let (header, size) = PageHeader::parse(&data, 0x619, 0, 8192).unwrap();
        assert!(matches!(header, PageHeader::Legacy2003 { .. }));
        assert_eq!(size, 40);
    }

    #[test]
    fn test_parse_vista() {
        let mut data = vec![0u8; 256];
        data[0..4].copy_from_slice(&0x12345678u32.to_le_bytes());
        data[4..8].copy_from_slice(&0x87654321u32.to_le_bytes());

        // Vista format: version 0x620, revision between 0x0b and 0x10
        let (header, size) = PageHeader::parse(&data, 0x620, 0x0b, 8192).unwrap();
        assert!(matches!(header, PageHeader::Vista { .. }));
        assert_eq!(size, 40);
    }

    #[test]
    fn test_parse_win7() {
        let mut data = vec![0u8; 256];
        data[0..8].copy_from_slice(&0x123456789abcdef0u64.to_le_bytes());

        let (header, size) = PageHeader::parse(&data, 0x620, 0x11, 8192).unwrap();
        assert!(matches!(header, PageHeader::Win7 { extended: None, .. }));
        assert_eq!(size, 40);
    }

    #[test]
    fn test_parse_win7_extended() {
        let mut data = vec![0u8; 256];
        data[0..8].copy_from_slice(&0x123456789abcdef0u64.to_le_bytes());

        let (header, size) = PageHeader::parse(&data, 0x620, 0x11, 16384).unwrap();
        assert!(matches!(
            header,
            PageHeader::Win7 {
                extended: Some(_),
                ..
            }
        ));
        assert_eq!(size, 80);
    }
}
