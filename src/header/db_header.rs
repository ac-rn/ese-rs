//! ESE database header structure.

use crate::constants::{DbState, FileType, DB_HEADER_SIGNATURE};
use crate::error::{EseError, Result};
use crate::header::JetSignature;
use zerocopy::{AsBytes, FromBytes, FromZeroes};

/// ESE database header (first page of the database file).
///
/// This structure contains metadata about the database including version,
/// page size, state, and various timestamps.
#[derive(Debug, Clone, Copy, AsBytes, FromBytes, FromZeroes)]
#[repr(C, packed)]
pub struct DbHeader {
    checksum: u32,
    signature: [u8; 4],
    version: u32,
    file_type: u32,
    db_time: u64,
    db_signature: JetSignature,
    db_state: u32,
    consistent_position: u64,
    consistent_time: u64,
    attach_time: u64,
    attach_position: u64,
    detach_time: u64,
    detach_position: u64,
    log_signature: JetSignature,
    unknown: u32,
    previous_backup: [u8; 24],
    previous_inc_backup: [u8; 24],
    current_full_backup: [u8; 24],
    shadowing_disabled: u32,
    last_object_id: u32,
    windows_major_version: u32,
    windows_minor_version: u32,
    windows_build_number: u32,
    windows_service_pack_number: u32,
    file_format_revision: u32,
    page_size: u32,
    repair_count: u32,
    repair_time: u64,
    unknown2: [u8; 28],
    scrub_time: u64,
    required_log: u64,
    upgrade_exchange_format: u32,
    upgrade_free_pages: u32,
    upgrade_space_map_pages: u32,
    current_shadow_backup: [u8; 24],
    creation_file_format_version: u32,
    creation_file_format_revision: u32,
    unknown3: [u8; 16],
    old_repair_count: u32,
    ecc_count: u32,
    last_ecc_time: u64,
    old_ecc_fix_success_count: u32,
    ecc_fix_error_count: u32,
    last_ecc_fix_error_time: u64,
    old_ecc_fix_error_count: u32,
    bad_checksum_error_count: u32,
    last_bad_checksum_time: u64,
    old_checksum_error_count: u32,
    committed_log: u32,
    previous_shadow_copy: [u8; 24],
    previous_differential_backup: [u8; 24],
    unknown4: [u8; 40],
    nls_major_version: u32,
    nls_minor_version: u32,
    unknown5: [u8; 148],
    unknown_flags: u32,
}

impl DbHeader {
    /// Expected size of the database header structure
    pub const SIZE: usize = std::mem::size_of::<DbHeader>();

    /// Parses a database header from raw bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The data is too short
    /// - The signature is invalid
    pub fn from_bytes(data: &[u8]) -> Result<&Self> {
        if data.len() < Self::SIZE {
            return Err(EseError::InvalidHeader);
        }

        let header = DbHeader::ref_from_prefix(data).ok_or(EseError::InvalidHeader)?;

        // Validate signature
        if header.signature != DB_HEADER_SIGNATURE {
            let sig = u32::from_le_bytes(header.signature);
            return Err(EseError::InvalidSignature(sig));
        }

        Ok(header)
    }

    /// Returns the database version number.
    #[inline]
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Returns the file format revision number.
    #[inline]
    pub fn file_format_revision(&self) -> u32 {
        self.file_format_revision
    }

    /// Returns the page size in bytes.
    #[inline]
    pub fn page_size(&self) -> u32 {
        self.page_size
    }

    /// Returns the database signature.
    #[inline]
    pub fn signature(&self) -> [u8; 4] {
        self.signature
    }

    /// Returns the checksum.
    #[inline]
    pub fn checksum(&self) -> u32 {
        self.checksum
    }

    /// Returns the file type.
    pub fn file_type(&self) -> Option<FileType> {
        match self.file_type {
            0 => Some(FileType::Database),
            1 => Some(FileType::StreamingFile),
            _ => None,
        }
    }

    /// Returns the database state.
    pub fn db_state(&self) -> Option<DbState> {
        match self.db_state {
            1 => Some(DbState::JustCreated),
            2 => Some(DbState::DirtyShutdown),
            3 => Some(DbState::CleanShutdown),
            4 => Some(DbState::BeingConverted),
            5 => Some(DbState::ForceDetach),
            _ => None,
        }
    }

    /// Returns true if this is a supported ESE version.
    pub fn is_supported_version(&self) -> bool {
        // Support Windows 2003 and later
        self.version >= 0x620
    }

    /// Returns a formatted version string.
    pub fn version_string(&self) -> String {
        // Copy fields to avoid unaligned reference to packed struct
        let version = self.version;
        let revision = self.file_format_revision;
        format!("0x{:x} rev 0x{:x}", version, revision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_header_size() {
        // Verify the structure size matches expectations
        // Note: packed struct with alignment 1, actual size is 660
        assert_eq!(DbHeader::SIZE, 660);
    }

    #[test]
    fn test_invalid_signature() {
        let mut data = vec![0u8; 1024];
        data[4..8].copy_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        let result = DbHeader::from_bytes(&data);
        assert!(result.is_err());
        assert!(matches!(result, Err(EseError::InvalidSignature(_))));
    }

    #[test]
    fn test_valid_signature() {
        let mut data = vec![0u8; 1024];
        data[4..8].copy_from_slice(&DB_HEADER_SIGNATURE);
        data[8..12].copy_from_slice(&0x620u32.to_le_bytes()); // version
        data[236..240].copy_from_slice(&8192u32.to_le_bytes()); // page_size (offset 236)

        let result = DbHeader::from_bytes(&data);
        assert!(result.is_ok());
        let header = result.unwrap();
        // Copy values to avoid unaligned references to packed struct
        let version = header.version;
        let page_size = header.page_size;
        assert_eq!(version, 0x620);
        assert_eq!(page_size, 8192);
    }
}
