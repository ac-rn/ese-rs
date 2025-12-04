//! Leaf page structures.

use crate::constants::tag_flags;
use crate::error::{EseError, Result};

/// Leaf page header (minimal, just contains common page key).
#[derive(Debug, Clone)]
pub struct LeafHeader {
    pub common_page_key: Vec<u8>,
}

impl LeafHeader {
    /// Parses a leaf header from tag data.
    pub fn parse(data: &[u8]) -> Result<Self> {
        // The entire tag data is the common page key
        Ok(LeafHeader {
            common_page_key: data.to_vec(),
        })
    }
}

/// Leaf page entry.
///
/// Leaf entries contain a local page key and entry data (the actual record).
#[derive(Debug, Clone)]
pub struct LeafEntry {
    pub common_page_key_size: Option<u16>,
    pub local_page_key_size: u16,
    pub local_page_key: Vec<u8>,
    pub entry_data: Vec<u8>,
}

impl LeafEntry {
    /// Parses a leaf entry from tag data.
    ///
    /// # Arguments
    ///
    /// * `flags` - Tag flags (determines if common key size is present)
    /// * `data` - Raw tag data
    pub fn parse(flags: u8, data: &[u8]) -> Result<Self> {
        let mut offset = 0;

        // Check minimum data length for the header
        let min_size = if flags & tag_flags::COMMON != 0 { 4 } else { 2 };
        if data.len() < min_size {
            return Err(EseError::Parse(format!(
                "Leaf entry too short: got {} bytes, need at least {} (flags: 0x{:x})",
                data.len(),
                min_size,
                flags
            )));
        }

        // Check if TAG_COMMON flag is set
        let common_page_key_size = if flags & tag_flags::COMMON != 0 {
            let size = u16::from_le_bytes([data[0], data[1]]);
            offset += 2;
            Some(size)
        } else {
            None
        };

        // Read local page key size
        if data.len() < offset + 2 {
            return Err(EseError::Parse(format!(
                "Leaf entry too short for local key size: got {} bytes, offset {}, need {} more",
                data.len(),
                offset,
                2
            )));
        }
        // Read size and mask out high bit (0x8000) which may be a flag in some formats
        let local_page_key_size_raw = u16::from_le_bytes([data[offset], data[offset + 1]]);
        let local_page_key_size = local_page_key_size_raw & 0x7FFF;
        offset += 2;

        // Read local page key (handle zero-length keys and short data)
        if local_page_key_size > 0 {
            // Check if we have enough data for the declared key size
            let available_for_key = data.len().saturating_sub(offset);
            let actual_key_size = std::cmp::min(local_page_key_size as usize, available_for_key);

            if actual_key_size > 0 {
                let local_page_key = data[offset..offset + actual_key_size].to_vec();
                offset += actual_key_size;

                // The rest is entry data (can be empty)
                let entry_data = if offset < data.len() {
                    data[offset..].to_vec()
                } else {
                    Vec::new()
                };

                Ok(LeafEntry {
                    common_page_key_size,
                    local_page_key_size: actual_key_size as u16,
                    local_page_key,
                    entry_data,
                })
            } else {
                // No space for key - treat as zero-length
                Ok(LeafEntry {
                    common_page_key_size,
                    local_page_key_size: 0,
                    local_page_key: Vec::new(),
                    entry_data: Vec::new(),
                })
            }
        } else {
            // Zero-length key is valid - rest is entry data
            let entry_data = if offset < data.len() {
                data[offset..].to_vec()
            } else {
                Vec::new()
            };

            Ok(LeafEntry {
                common_page_key_size,
                local_page_key_size: 0,
                local_page_key: Vec::new(),
                entry_data,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leaf_entry_without_common() {
        let mut data = vec![];
        data.extend_from_slice(&5u16.to_le_bytes()); // local key size
        data.extend_from_slice(b"ABCDE"); // local key
        data.extend_from_slice(b"ENTRYDATA"); // entry data

        let entry = LeafEntry::parse(0, &data).unwrap();
        assert_eq!(entry.common_page_key_size, None);
        assert_eq!(entry.local_page_key_size, 5);
        assert_eq!(entry.local_page_key, b"ABCDE");
        assert_eq!(entry.entry_data, b"ENTRYDATA");
    }

    #[test]
    fn test_leaf_entry_with_common() {
        let mut data = vec![];
        data.extend_from_slice(&10u16.to_le_bytes()); // common key size
        data.extend_from_slice(&5u16.to_le_bytes()); // local key size
        data.extend_from_slice(b"ABCDE"); // local key
        data.extend_from_slice(b"ENTRYDATA"); // entry data

        let entry = LeafEntry::parse(tag_flags::COMMON, &data).unwrap();
        assert_eq!(entry.common_page_key_size, Some(10));
        assert_eq!(entry.local_page_key_size, 5);
        assert_eq!(entry.local_page_key, b"ABCDE");
        assert_eq!(entry.entry_data, b"ENTRYDATA");
    }

    #[test]
    fn test_leaf_entry_zero_length_key() {
        let mut data = vec![];
        data.extend_from_slice(&0u16.to_le_bytes()); // local key size = 0
        data.extend_from_slice(b"ENTRYDATA"); // entry data

        let entry = LeafEntry::parse(0, &data).unwrap();
        assert_eq!(entry.common_page_key_size, None);
        assert_eq!(entry.local_page_key_size, 0);
        assert_eq!(entry.local_page_key, b"");
        assert_eq!(entry.entry_data, b"ENTRYDATA");
    }

    #[test]
    fn test_leaf_entry_minimal() {
        let mut data = vec![];
        data.extend_from_slice(&0u16.to_le_bytes()); // local key size = 0

        let entry = LeafEntry::parse(0, &data).unwrap();
        assert_eq!(entry.common_page_key_size, None);
        assert_eq!(entry.local_page_key_size, 0);
        assert_eq!(entry.local_page_key, b"");
        assert_eq!(entry.entry_data, b"");
    }

    #[test]
    fn test_leaf_entry_empty_entry_data() {
        let mut data = vec![];
        data.extend_from_slice(&3u16.to_le_bytes()); // local key size
        data.extend_from_slice(b"KEY"); // local key (no entry data)

        let entry = LeafEntry::parse(0, &data).unwrap();
        assert_eq!(entry.common_page_key_size, None);
        assert_eq!(entry.local_page_key_size, 3);
        assert_eq!(entry.local_page_key, b"KEY");
        assert_eq!(entry.entry_data, b"");
    }
}
