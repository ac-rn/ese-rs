//! Page-level structures and parsing.

pub mod branch;
pub mod header;
pub mod index;
pub mod leaf;
pub mod root;
pub mod space_tree;
pub mod tag;

pub use branch::{BranchEntry, BranchHeader};
pub use header::{PageHeader, PageHeaderCommon};
pub use index::IndexEntry;
pub use leaf::{LeafEntry, LeafHeader};
pub use root::RootHeader;
pub use space_tree::{SpaceTreeEntry, SpaceTreeHeader};
pub use tag::TagExtractor;

use crate::error::Result;

/// Represents a parsed page from the database.
pub struct Page<'a> {
    /// Raw page data
    pub data: &'a [u8],
    /// Parsed page header
    pub header: PageHeader,
    /// Length of the header in bytes
    pub header_len: usize,
}

impl<'a> Page<'a> {
    /// Parses a page from raw data.
    pub fn parse(data: &'a [u8], version: u32, revision: u32, page_size: u32) -> Result<Self> {
        let (header, header_len) = PageHeader::parse(data, version, revision, page_size)?;

        Ok(Page {
            data,
            header,
            header_len,
        })
    }

    /// Creates a tag extractor for this page.
    pub fn tag_extractor(&self, version: u32, revision: u32, page_size: u32) -> TagExtractor<'a> {
        TagExtractor::new(self.data, self.header_len, version, revision, page_size)
    }

    /// Returns the common header fields.
    pub fn common(&self) -> &PageHeaderCommon {
        self.header.common()
    }

    /// Returns true if this is a root page.
    pub fn is_root(&self) -> bool {
        self.header.is_root()
    }

    /// Returns true if this is a leaf page.
    pub fn is_leaf(&self) -> bool {
        self.header.is_leaf()
    }

    /// Returns true if this is a branch page.
    pub fn is_branch(&self) -> bool {
        !self.header.is_leaf()
    }

    /// Returns true if this is a space tree page.
    pub fn is_space_tree(&self) -> bool {
        self.header.is_space_tree()
    }

    /// Returns true if this is an index page.
    pub fn is_index(&self) -> bool {
        self.header.is_index()
    }

    /// Returns true if this is a long value page.
    pub fn is_long_value(&self) -> bool {
        self.header.is_long_value()
    }
}
