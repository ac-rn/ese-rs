//! Index page entry structure.

/// Index page entry.
///
/// Index entries contain a record page key used for indexing.
#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub record_page_key: Vec<u8>,
}

impl IndexEntry {
    /// Parses an index entry from tag data.
    pub fn parse(data: &[u8]) -> Self {
        IndexEntry {
            record_page_key: data.to_vec(),
        }
    }
}
