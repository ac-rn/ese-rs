//! Integration tests for SRUDB.dat database parsing
//!
//! SRUDB.dat is a Windows System Resource Usage Monitor database.
//! These tests require the fixture file at the crate root; they are
//! skipped (treated as passing) when it is not present — e.g. in CI
//! environments where forensic fixtures are not checked in.

use ese_parser::{Database, EseError};
use std::path::Path;

fn open_fixture(name: &str) -> Result<Option<Database>, EseError> {
    if !Path::new(name).exists() {
        eprintln!("skipping: fixture {name} not present");
        return Ok(None);
    }
    Ok(Some(Database::open(name)?))
}

#[test]
fn test_srudb_opens() -> Result<(), EseError> {
    let Some(db) = open_fixture("SRUDB.dat")? else {
        return Ok(());
    };
    assert!(!db.tables().is_empty(), "Database should have tables");
    Ok(())
}

#[test]
fn test_srudb_has_expected_structure() -> Result<(), EseError> {
    let Some(db) = open_fixture("SRUDB.dat")? else {
        return Ok(());
    };

    // SRUDB typically has system tables
    let table_names: Vec<String> = db
        .tables()
        .keys()
        .map(|name| String::from_utf8_lossy(name).to_string())
        .collect();

    println!("Found {} tables in SRUDB.dat:", table_names.len());
    for name in &table_names {
        println!("  - {}", name);
    }

    // Basic sanity check - SRUDB should have multiple tables
    assert!(
        table_names.len() >= 5,
        "SRUDB should have at least 5 tables"
    );

    Ok(())
}

#[test]
fn test_srudb_catalog_parsing() -> Result<(), EseError> {
    let Some(db) = open_fixture("SRUDB.dat")? else {
        return Ok(());
    };

    // Verify each table has columns (except MSysObjects which may be empty)
    for (table_name, table_info) in db.tables() {
        let name = String::from_utf8_lossy(table_name);

        // Most tables should have at least one column (MSysObjects may be empty)
        if name != "MSysObjects" {
            assert!(
                !table_info.columns.is_empty(),
                "Table '{}' should have columns",
                name
            );
        }

        // Verify column identifiers are valid
        for (col_name, col_info) in &table_info.columns {
            let col_name_str = String::from_utf8_lossy(col_name);
            assert!(
                col_info.identifier > 0,
                "Column '{}' in table '{}' should have valid identifier",
                col_name_str,
                name
            );
        }
    }

    Ok(())
}

#[test]
fn test_srudb_table_iteration() -> Result<(), EseError> {
    let Some(db) = open_fixture("SRUDB.dat")? else {
        return Ok(());
    };

    // Try to iterate through the first table
    if let Some((table_name, _)) = db.tables().iter().next() {
        let name = String::from_utf8_lossy(table_name);
        println!("Testing iteration on table: {}", name);

        let mut cursor = db.open_table(table_name)?;
        let mut record_count = 0;
        let max_records = 10; // Just test first 10 records

        while let Some(record) = cursor.next_row()? {
            record_count += 1;

            // Verify record has data
            assert!(!record.is_empty(), "Record should have columns");

            if record_count >= max_records {
                break;
            }
        }

        println!("Successfully read {} records from '{}'", record_count, name);
    }

    Ok(())
}

#[test]
fn test_srudb_database_metadata() -> Result<(), EseError> {
    let Some(db) = open_fixture("SRUDB.dat")? else {
        return Ok(());
    };

    // Check database header
    let header = db.header();
    println!("Database version: {}", header.version_string());
    println!("Page size: {}", db.page_size());
    println!("Total pages: {}", db.total_pages());

    // SRUDB uses standard Windows page size
    assert!(
        db.page_size() == 8192 || db.page_size() == 4096 || db.page_size() == 32768,
        "Page size should be a standard Windows value"
    );

    // Database should have multiple pages
    assert!(db.total_pages() > 10, "Database should have multiple pages");

    Ok(())
}

#[test]
fn test_srudb_column_types() -> Result<(), EseError> {
    let Some(db) = open_fixture("SRUDB.dat")? else {
        return Ok(());
    };

    let mut column_types_found = std::collections::HashSet::new();

    // Collect all column types used in the database
    for (_table_name, table_info) in db.tables() {
        for (_col_name, col_info) in &table_info.columns {
            column_types_found.insert(col_info.column_type);
        }
    }

    println!(
        "Found {} different column types in SRUDB.dat:",
        column_types_found.len()
    );
    for col_type in &column_types_found {
        println!("  - {}", col_type.name());
    }

    // SRUDB should use multiple column types
    assert!(
        column_types_found.len() >= 3,
        "SRUDB should use at least 3 different column types"
    );

    Ok(())
}
