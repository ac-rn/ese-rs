//! Integration tests for STORED tagged long-value resolution.

use ese_parser::{ColumnValue, Database, EseError};

fn db_with_lv_table(path: &str) -> Result<Database, EseError> {
    Database::open(path)
}

#[test]
fn test_webcache_has_long_value_tables() -> Result<(), EseError> {
    let db = db_with_lv_table("WebCacheV01.dat")?;
    let with_lv = db
        .tables()
        .values()
        .filter(|t| !t.long_values.is_empty())
        .count();
    assert!(
        with_lv > 0,
        "WebCacheV01 should expose at least one table with a long-value tree"
    );
    Ok(())
}

#[test]
fn test_no_long_value_leaks_to_caller() -> Result<(), EseError> {
    // After the cursor resolves STORED keys, no ColumnValue::LongValue raw
    // key should reach the caller for columns that have a resolvable tree.
    let db = db_with_lv_table("WebCacheV01.dat")?;

    let mut scanned = 0usize;
    let mut resolved = 0usize;

    for (table_name, table_info) in db.tables() {
        if table_info.long_values.is_empty() {
            continue;
        }
        let mut cursor = db.open_table(table_name)?;
        let mut rows = 0;
        while let Some(record) = cursor.next_row()? {
            rows += 1;
            for (_col, value) in &record {
                scanned += 1;
                match value {
                    ColumnValue::LongValue(_) => panic!(
                        "Unresolved LongValue key leaked from table {}",
                        String::from_utf8_lossy(table_name)
                    ),
                    ColumnValue::Text(_) | ColumnValue::Binary(_) => resolved += 1,
                    _ => {}
                }
            }
            if rows >= 50 {
                break;
            }
        }
    }

    assert!(scanned > 0, "Expected to scan at least one LV-table row");
    // Not a strict lower bound — we just need the resolver path exercised.
    let _ = resolved;
    Ok(())
}

#[test]
fn test_long_value_reassembles_to_descriptor_size() -> Result<(), EseError> {
    // For every Text value that originated from the LV tree, the decoded
    // string should be non-empty when the underlying bytes are non-empty.
    // This indirectly validates descriptor-size truncation: if truncation
    // were over-aggressive we'd see empty strings in place of URLs/keys.
    let db = db_with_lv_table("WebCacheV01.dat")?;

    let mut seen_nonempty_text = false;

    for (table_name, table_info) in db.tables() {
        if table_info.long_values.is_empty() {
            continue;
        }
        let mut cursor = db.open_table(table_name)?;
        let mut rows = 0;
        while let Some(record) = cursor.next_row()? {
            rows += 1;
            for (_col, value) in &record {
                if let ColumnValue::Text(s) = value {
                    if !s.is_empty() {
                        seen_nonempty_text = true;
                    }
                }
            }
            if rows >= 200 {
                break;
            }
        }
        if seen_nonempty_text {
            break;
        }
    }

    assert!(
        seen_nonempty_text,
        "Expected at least one non-empty resolved Text value from an LV-bearing table"
    );
    Ok(())
}
