//! Tag extraction from pages.
//!
//! Tags are stored at the end of each page and point to data within the page.
//! This module provides zero-copy extraction of tag data.

use crate::constants::{is_large_page_format, PAGE_HEADER_TAG_OFFSET};
use crate::error::{EseError, Result};

/// Extracts tags from page data without copying.
pub struct TagExtractor<'a> {
    page_data: &'a [u8],
    header_len: usize,
    version: u32,
    revision: u32,
    page_size: u32,
    num_tags: u16,
}

impl<'a> TagExtractor<'a> {
    /// Creates a new tag extractor.
    pub fn new(
        page_data: &'a [u8],
        header_len: usize,
        version: u32,
        revision: u32,
        page_size: u32,
    ) -> Self {
        // Extract number of tags from the page header common fields
        // The first_available_page_tag field is at a FIXED offset in PageHeaderCommon
        // For Win7+ format: 8 bytes checksum + 34 bytes into common = offset 34 from common start
        // For Vista format: 4+4 bytes checksums + 34 bytes into common = offset 34 from common start
        // For Legacy format: 4+4 bytes + 34 bytes into common = offset 34 from common start
        // The common header always starts after the initial checksum(s), and first_available_page_tag
        // is at offset 34 within the common header structure
        let num_tags = if page_data.len() >= 40 {
            // For all formats, the tag count is at offset 34-35 in the page
            // (after 8-byte or 4+4 byte checksums, this lands in the common header)
            u16::from_le_bytes([
                page_data[PAGE_HEADER_TAG_OFFSET],
                page_data[PAGE_HEADER_TAG_OFFSET + 1],
            ])
        } else {
            0
        };

        TagExtractor {
            page_data,
            header_len,
            version,
            revision,
            page_size,
            num_tags,
        }
    }

    /// Returns the number of tags in this page.
    pub fn num_tags(&self) -> u16 {
        self.num_tags
    }

    /// Extracts a tag by its index.
    ///
    /// Returns a tuple of (flags, data_slice).
    /// The data slice is a zero-copy reference into the page data.
    ///
    /// # Errors
    ///
    /// Returns an error if the tag number is invalid or if the tag
    /// points to invalid offsets.
    pub fn extract_tag(&self, tag_num: u16) -> Result<(u8, &'a [u8])> {
        if tag_num >= self.num_tags {
            return Err(EseError::InvalidTagNumber(tag_num));
        }

        // Tags are stored at the end of the page, working backwards
        // Each tag is 4 bytes: 2 bytes for value size, 2 bytes for offset/flags
        // CRITICAL: Python reads tags BACKWARDS from the end!
        // Tag 0 is at page_end - 4, Tag 1 is at page_end - 8, etc.
        let tag_offset = self.page_data.len() - ((tag_num as usize + 1) * 4);

        if tag_offset + 4 > self.page_data.len() {
            return Err(EseError::PageDataTooShort {
                expected: tag_offset + 4,
                actual: self.page_data.len(),
            });
        }

        let tag_bytes = &self.page_data[tag_offset..tag_offset + 4];

        // Parse tag descriptor based on version
        let (value_size, page_flags, value_offset) =
            if is_large_page_format(self.version, self.revision, self.page_size) {
                // New format for large pages (Win7+, revision >= 17, page > 8KB)
                let size_raw = u16::from_le_bytes([tag_bytes[0], tag_bytes[1]]);
                let offset_raw = u16::from_le_bytes([tag_bytes[2], tag_bytes[3]]);

                let value_size = (size_raw & 0x7fff) as usize;
                let value_offset = (offset_raw & 0x7fff) as usize;

                // For new format, flags are embedded in the data itself
                // We need to extract and clear them
                let data_start = self.header_len + value_offset;
                if data_start + value_size > self.page_data.len() {
                    return Err(EseError::TagOffsetOutOfBounds {
                        offset: value_offset,
                        size: value_size,
                        page_size: self.page_data.len(),
                    });
                }

                // Flags are in bits 5-7 of the second byte
                let page_flags = if value_size > 1 {
                    (self.page_data[data_start + 1] >> 5) & 0x07
                } else {
                    0
                };

                // Extract data and clear the flag bits
                let mut data = self.page_data[data_start..data_start + value_size].to_vec();
                if data.len() > 1 {
                    data[1] &= 0x1f; // Clear bits 5-7
                }

                // SAFETY: We're returning a reference to modified data, but we need to
                // return &'a [u8]. For now, we'll handle this by returning the original
                // data and document that flags need to be cleared by the caller.
                // TODO: Consider a better approach that doesn't require allocation.

                (value_size, page_flags, value_offset)
            } else {
                // Legacy format
                let size_raw = u16::from_le_bytes([tag_bytes[0], tag_bytes[1]]);
                let offset_flags_raw = u16::from_le_bytes([tag_bytes[2], tag_bytes[3]]);

                let value_size = (size_raw & 0x1fff) as usize;
                let page_flags = ((offset_flags_raw & 0xe000) >> 13) as u8;
                let value_offset = (offset_flags_raw & 0x1fff) as usize;

                (value_size, page_flags, value_offset)
            };

        // Extract the data slice
        let data_start = self.header_len + value_offset;
        let data_end = data_start + value_size;

        if data_end > self.page_data.len() {
            return Err(EseError::TagOffsetOutOfBounds {
                offset: value_offset,
                size: value_size,
                page_size: self.page_data.len(),
            });
        }

        let tag_data = &self.page_data[data_start..data_end];

        Ok((page_flags, tag_data))
    }

    /// Extracts a tag and returns owned data with flags cleared (for new format).
    ///
    /// This is necessary for the new format where flags are embedded in the data.
    pub fn extract_tag_owned(&self, tag_num: u16) -> Result<(u8, Vec<u8>)> {
        if tag_num >= self.num_tags {
            return Err(EseError::InvalidTagNumber(tag_num));
        }

        // CRITICAL: Tags are stored BACKWARDS from the end of the page!
        // Tag 0 is at page_end - 4, Tag 1 is at page_end - 8, etc.
        // This must match the logic in extract_tag()
        let tag_offset = self.page_data.len() - ((tag_num as usize + 1) * 4);

        if tag_offset + 4 > self.page_data.len() {
            return Err(EseError::PageDataTooShort {
                expected: tag_offset + 4,
                actual: self.page_data.len(),
            });
        }

        let tag_bytes = &self.page_data[tag_offset..tag_offset + 4];

        if is_large_page_format(self.version, self.revision, self.page_size) {
            // New format - need to extract and clear flags
            let size_raw = u16::from_le_bytes([tag_bytes[0], tag_bytes[1]]);
            let offset_raw = u16::from_le_bytes([tag_bytes[2], tag_bytes[3]]);

            let value_size = (size_raw & 0x7fff) as usize;
            let value_offset = (offset_raw & 0x7fff) as usize;

            let data_start = self.header_len + value_offset;
            if data_start + value_size > self.page_data.len() {
                return Err(EseError::TagOffsetOutOfBounds {
                    offset: value_offset,
                    size: value_size,
                    page_size: self.page_data.len(),
                });
            }

            let mut data = self.page_data[data_start..data_start + value_size].to_vec();
            let page_flags = if data.len() > 1 {
                let flags = (data[1] >> 5) & 0x07;
                data[1] &= 0x1f; // Clear flag bits
                flags
            } else {
                0
            };

            Ok((page_flags, data))
        } else {
            // Legacy format - can use zero-copy
            let (flags, data) = self.extract_tag(tag_num)?;
            Ok((flags, data.to_vec()))
        }
    }

    /// Iterates over all tags in the page.
    pub fn iter_tags(&'a self) -> TagIterator<'a> {
        TagIterator {
            extractor: self,
            current: 0,
        }
    }
}

/// Iterator over tags in a page.
pub struct TagIterator<'a> {
    extractor: &'a TagExtractor<'a>,
    current: u16,
}

impl<'a> Iterator for TagIterator<'a> {
    type Item = Result<(u8, &'a [u8])>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.extractor.num_tags {
            return None;
        }

        let result = self.extractor.extract_tag(self.current);
        self.current += 1;
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_page(num_tags: u16) -> Vec<u8> {
        let mut page = vec![0u8; 8192];

        // Mock header (40 bytes for Win7 format)
        let header_len = 40;

        // Set first_available_page_tag in header (at offset 34)
        page[34..36].copy_from_slice(&num_tags.to_le_bytes());

        // Add some tag data in the middle of the page
        let data_start = header_len + 100;
        page[data_start..data_start + 10].copy_from_slice(b"TESTDATA12");

        // Add tag descriptors at the end
        let tag_array_start = page.len() - (num_tags as usize * 4);
        if num_tags > 0 {
            // First tag: size=10, offset=100, flags=0
            page[tag_array_start..tag_array_start + 2].copy_from_slice(&10u16.to_le_bytes());
            page[tag_array_start + 2..tag_array_start + 4].copy_from_slice(&100u16.to_le_bytes());
        }

        page
    }

    #[test]
    fn test_tag_extractor_num_tags() {
        let page = create_test_page(5);
        let extractor = TagExtractor::new(&page, 40, 0x620, 0x11, 8192);
        assert_eq!(extractor.num_tags(), 5);
    }

    #[test]
    fn test_extract_tag() {
        let page = create_test_page(1);
        let extractor = TagExtractor::new(&page, 40, 0x620, 0x11, 8192);

        let (flags, data) = extractor.extract_tag(0).unwrap();
        assert_eq!(flags, 0);
        assert_eq!(data, b"TESTDATA12");
    }

    #[test]
    fn test_invalid_tag_number() {
        let page = create_test_page(1);
        let extractor = TagExtractor::new(&page, 40, 0x620, 0x11, 8192);

        let result = extractor.extract_tag(5);
        assert!(matches!(result, Err(EseError::InvalidTagNumber(5))));
    }
}
