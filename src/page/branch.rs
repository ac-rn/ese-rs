//! Branch page structures.

use crate::constants::tag_flags;
use crate::error::{EseError, Result};

/// Branch page header (minimal, just contains common page key).
#[derive(Debug, Clone)]
pub struct BranchHeader {
    pub common_page_key: Vec<u8>,
}

impl BranchHeader {
    /// Parses a branch header from tag data.
    pub fn parse(data: &[u8]) -> Result<Self> {
        // The entire tag data is the common page key
        Ok(BranchHeader {
            common_page_key: data.to_vec(),
        })
    }
}

/// Branch page entry.
///
/// Branch entries contain a local page key and a child page number.
#[derive(Debug, Clone)]
pub struct BranchEntry {
    pub common_page_key_size: Option<u16>,
    pub local_page_key_size: u16,
    pub local_page_key: Vec<u8>,
    pub child_page_number: u32,
}

impl BranchEntry {
    /// Parses a branch entry from tag data.
    ///
    /// # Arguments
    ///
    /// * `flags` - Tag flags (determines if common key size is present)
    /// * `data` - Raw tag data
    pub fn parse(flags: u8, data: &[u8]) -> Result<Self> {
        let mut offset = 0;

        // Check minimum data length for the header
        let min_size = if flags & tag_flags::COMMON != 0 { 8 } else { 6 }; // 2 bytes key size + 4 bytes page number
        if data.len() < min_size {
            return Err(EseError::Parse(format!(
                "Branch entry too short: got {} bytes, need at least {} (flags: 0x{:x})",
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
                "Branch entry too short for local key size: got {} bytes, offset {}, need {} more",
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
        let local_page_key = if local_page_key_size > 0 {
            let available_for_key = data.len().saturating_sub(offset);
            let actual_key_size = std::cmp::min(local_page_key_size as usize, available_for_key);

            if actual_key_size > 0 {
                let key = data[offset..offset + actual_key_size].to_vec();
                offset += actual_key_size;
                key
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // Read child page number - if not enough data, this is not a valid branch entry
        if data.len() < offset + 4 {
            // Not enough data for child page number - likely metadata, not a real branch entry
            // Return error so caller can skip this tag
            return Err(EseError::Parse(format!(
                "Branch entry too short for child page number: got {} bytes, offset {}, need {} more (likely metadata tag)",
                data.len(),
                offset,
                4
            )));
        }
        let child_page_number = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);

        Ok(BranchEntry {
            common_page_key_size,
            local_page_key_size,
            local_page_key,
            child_page_number,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_entry_without_common() {
        let mut data = vec![];
        data.extend_from_slice(&5u16.to_le_bytes()); // local key size
        data.extend_from_slice(b"ABCDE"); // local key
        data.extend_from_slice(&42u32.to_le_bytes()); // child page number

        let entry = BranchEntry::parse(0, &data).unwrap();
        assert_eq!(entry.common_page_key_size, None);
        assert_eq!(entry.local_page_key_size, 5);
        assert_eq!(entry.local_page_key, b"ABCDE");
        assert_eq!(entry.child_page_number, 42);
    }

    #[test]
    fn test_branch_entry_with_common() {
        let mut data = vec![];
        data.extend_from_slice(&10u16.to_le_bytes()); // common key size
        data.extend_from_slice(&5u16.to_le_bytes()); // local key size
        data.extend_from_slice(b"ABCDE"); // local key
        data.extend_from_slice(&42u32.to_le_bytes()); // child page number

        let entry = BranchEntry::parse(tag_flags::COMMON, &data).unwrap();
        assert_eq!(entry.common_page_key_size, Some(10));
        assert_eq!(entry.local_page_key_size, 5);
        assert_eq!(entry.local_page_key, b"ABCDE");
        assert_eq!(entry.child_page_number, 42);
    }
}
