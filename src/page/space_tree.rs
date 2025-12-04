//! Space tree structures.

use zerocopy::{AsBytes, FromBytes, FromZeroes};

/// Space tree page header.
#[derive(Debug, Clone, Copy, AsBytes, FromBytes, FromZeroes)]
#[repr(C, packed)]
pub struct SpaceTreeHeader {
    pub unknown: u64,
}

impl SpaceTreeHeader {
    /// Parses a space tree header from tag data.
    pub fn from_bytes(data: &[u8]) -> Option<&Self> {
        SpaceTreeHeader::ref_from_prefix(data)
    }
}

/// Space tree page entry.
#[derive(Debug, Clone, Copy, AsBytes, FromBytes, FromZeroes)]
#[repr(C, packed)]
pub struct SpaceTreeEntry {
    pub page_key_size: u16,
    pub last_page_number: u32,
    pub number_of_pages: u32,
}

impl SpaceTreeEntry {
    /// Parses a space tree entry from tag data.
    pub fn from_bytes(data: &[u8]) -> Option<&Self> {
        SpaceTreeEntry::ref_from_prefix(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space_tree_header_size() {
        assert_eq!(std::mem::size_of::<SpaceTreeHeader>(), 8);
    }

    #[test]
    fn test_space_tree_entry_size() {
        assert_eq!(std::mem::size_of::<SpaceTreeEntry>(), 10);
    }
}
