//! Root page header structure.

use zerocopy::{AsBytes, FromBytes, FromZeroes};

/// Root page header.
///
/// Root pages are the top level of a B-tree structure.
#[derive(Debug, Clone, Copy, AsBytes, FromBytes, FromZeroes)]
#[repr(C, packed)]
pub struct RootHeader {
    pub initial_number_of_pages: u32,
    pub parent_father_data_page: u32,
    pub extent_space: u32,
    pub space_tree_page_number: u32,
}

impl RootHeader {
    /// Parses a root header from tag data.
    pub fn from_bytes(data: &[u8]) -> Option<&Self> {
        RootHeader::ref_from_prefix(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_header_size() {
        assert_eq!(std::mem::size_of::<RootHeader>(), 16);
    }
}
