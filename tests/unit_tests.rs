//! Unit tests for ESE parser.

use ese_parser::constants::ColumnType;
use ese_parser::header::{DbHeader, JetSignature};
use ese_parser::page::PageHeader;

#[test]
fn test_db_header_parsing() {
    let mut data = vec![0u8; 1024];

    // Set signature
    data[4..8].copy_from_slice(&[0xef, 0xcd, 0xab, 0x89]);

    // Set version and page size
    data[8..12].copy_from_slice(&0x620u32.to_le_bytes());
    data[236..240].copy_from_slice(&8192u32.to_le_bytes()); // page_size at offset 236

    let header = DbHeader::from_bytes(&data).unwrap();
    // Use accessor methods to avoid unaligned references to packed struct
    assert_eq!(header.version(), 0x620);
    assert_eq!(header.page_size(), 8192);
}

#[test]
fn test_invalid_signature() {
    let mut data = vec![0u8; 1024];
    data[4..8].copy_from_slice(&[0x00, 0x00, 0x00, 0x00]);

    let result = DbHeader::from_bytes(&data);
    assert!(result.is_err());
}

#[test]
fn test_column_type_mapping() {
    assert_eq!(ColumnType::from_u32(1), Some(ColumnType::Bit));
    assert_eq!(ColumnType::from_u32(10), Some(ColumnType::Text));
    assert_eq!(ColumnType::from_u32(999), None);
}

#[test]
fn test_column_type_names() {
    assert_eq!(ColumnType::Text.name(), "Text");
    assert_eq!(ColumnType::LongLong.name(), "Long long");
    assert_eq!(ColumnType::GUID.name(), "GUID");
}

#[test]
fn test_column_type_sizes() {
    assert_eq!(ColumnType::Bit.fixed_size(), Some(1));
    assert_eq!(ColumnType::Long.fixed_size(), Some(4));
    assert_eq!(ColumnType::GUID.fixed_size(), Some(16));
    assert_eq!(ColumnType::Text.fixed_size(), None);
}

#[test]
fn test_page_header_legacy_2003() {
    let mut data = vec![0u8; 256];
    data[0..4].copy_from_slice(&0x12345678u32.to_le_bytes());
    data[4..8].copy_from_slice(&42u32.to_le_bytes());

    let (header, size) = PageHeader::parse(&data, 0x619, 0, 8192).unwrap();
    assert!(matches!(header, PageHeader::Legacy2003 { .. }));
    assert_eq!(size, 40);
}

#[test]
fn test_page_header_vista() {
    let mut data = vec![0u8; 256];
    data[0..4].copy_from_slice(&0x12345678u32.to_le_bytes());
    data[4..8].copy_from_slice(&0x87654321u32.to_le_bytes());

    // Vista format: version 0x620, revision between 0x0b and 0x10
    let (header, size) = PageHeader::parse(&data, 0x620, 0x0b, 8192).unwrap();
    assert!(matches!(header, PageHeader::Vista { .. }));
    assert_eq!(size, 40);
}

#[test]
fn test_page_header_win7() {
    let mut data = vec![0u8; 256];
    data[0..8].copy_from_slice(&0x123456789abcdef0u64.to_le_bytes());

    let (header, size) = PageHeader::parse(&data, 0x620, 0x11, 8192).unwrap();
    assert!(matches!(header, PageHeader::Win7 { extended: None, .. }));
    assert_eq!(size, 40);
}

#[test]
fn test_jet_signature() {
    let sig = JetSignature {
        random: 12345,
        creation_time: 0,
        netbios_name: [0; 16],
    };

    // Copy value to avoid unaligned reference to packed struct
    let random = sig.random;
    assert_eq!(random, 12345);
}
