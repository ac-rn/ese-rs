//! Table cursor for iterating over records.

use crate::catalog::table_info::TableInfo;
use crate::constants::is_large_page_format;
use crate::database::Database;
use crate::error::Result;
use crate::page::{BranchEntry, LeafEntry, Page};
use crate::record::RecordParser;
use crate::types::ColumnValue;
use indexmap::IndexMap;

/// Cursor for iterating over records in a table.
///
/// The cursor maintains state about the current position within the table's
/// B-tree structure and allows sequential iteration through records.
pub struct TableCursor<'a> {
    db: &'a Database,
    table_info: &'a TableInfo,
    current_page: Option<Page<'a>>,
    current_tag: u16,
}

impl<'a> TableCursor<'a> {
    /// Creates a new table cursor.
    ///
    /// The cursor is positioned at the first leaf page of the table.
    pub(crate) fn new(db: &'a Database, table_info: &'a TableInfo) -> Result<Self> {
        // Navigate to first leaf page by following tag 1 of branch pages
        // This matches Python's openTable logic (lines 724-741 in ese.py)
        let mut page_num = table_info.father_data_page_number;
        let mut page = db.get_page(page_num)?;

        // Keep following tag 1 until we reach a leaf page
        while !page.is_leaf() {
            let extractor = page.tag_extractor(
                db.header().version(),
                db.header().file_format_revision(),
                db.page_size(),
            );

            // Check if there are any tags
            if extractor.num_tags() <= 1 {
                // No records, start with empty page
                break;
            }

            // Get tag 1 (Python uses range(1, FirstAvailablePageTag))
            let (flags, data) = extractor.extract_tag(1)?;
            let branch_entry = BranchEntry::parse(flags, data)?;
            page_num = branch_entry.child_page_number;
            page = db.get_page(page_num)?;
        }

        Ok(TableCursor {
            db,
            table_info,
            current_page: Some(page),
            current_tag: 1, // Start at tag 1 (tag 0 is the header)
        })
    }

    /// Advances to the next row and returns the parsed record.
    ///
    /// Returns `None` when there are no more records.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ese_rs::Database;
    ///
    /// let db = Database::open("database.edb")?;
    /// let mut cursor = db.open_table(b"MyTable")?;
    ///
    /// while let Some(record) = cursor.next_row()? {
    ///     for (column_name, value) in &record {
    ///         println!("{}: {}", String::from_utf8_lossy(column_name), value);
    ///     }
    /// }
    /// # Ok::<(), ese_rs::EseError>(())
    /// ```
    pub fn next_row(&mut self) -> Result<Option<IndexMap<Vec<u8>, ColumnValue>>> {
        loop {
            // Check if we have a current page
            let page = match &self.current_page {
                Some(p) => p,
                None => return Ok(None),
            };

            // Check if we have more tags in the current page
            if self.current_tag >= page.common().first_available_page_tag {
                // Move to next page via next_page_number link (like Python)
                let next_page_num = page.common().next_page_number;

                if next_page_num == 0 {
                    // No more pages
                    return Ok(None);
                }

                // Load next leaf page
                let next_page = self.db.get_page(next_page_num)?;
                self.current_page = Some(next_page);
                self.current_tag = 1;
                continue;
            }

            // Extract the current tag
            let extractor = page.tag_extractor(
                self.db.header().version(),
                self.db.header().file_format_revision(),
                self.db.page_size(),
            );

            // For large pages, use extract_tag_owned to get data with flags cleared
            let (flags, data_vec);
            let data: &[u8] = if is_large_page_format(
                self.db.header().version(),
                self.db.header().file_format_revision(),
                self.db.page_size(),
            ) {
                match extractor.extract_tag_owned(self.current_tag) {
                    Ok(result) => {
                        flags = result.0;
                        data_vec = result.1;
                        &data_vec
                    }
                    Err(_) => {
                        self.current_tag += 1;
                        continue;
                    }
                }
            } else {
                match extractor.extract_tag(self.current_tag) {
                    Ok(result) => {
                        flags = result.0;
                        result.1
                    }
                    Err(_) => {
                        self.current_tag += 1;
                        continue;
                    }
                }
            };

            self.current_tag += 1;

            // Verify this is a leaf page (sanity check)
            if !page.is_leaf() {
                continue; // Skip non-leaf pages instead of failing
            }

            // Skip non-table leaf entries
            if page.is_space_tree() || page.is_index() || page.is_long_value() {
                continue;
            }

            // Parse leaf entry - skip if it fails
            let leaf_entry = match LeafEntry::parse(flags, data) {
                Ok(entry) => entry,
                Err(e) => {
                    #[cfg(feature = "logging")]
                    log::debug!(
                        "Skipping unparseable leaf entry at tag {}: {}",
                        self.current_tag - 1,
                        e
                    );
                    continue;
                }
            };

            // Parse the record - skip if it fails
            let parser =
                RecordParser::new(&leaf_entry.entry_data, self.table_info, self.db.header());

            match parser.parse_record() {
                Ok(record) => {
                    // Skip rows where ALL values are null (empty records)
                    let all_null = record.values().all(|v| v.is_null());
                    if all_null {
                        continue;
                    }
                    return Ok(Some(record));
                }
                Err(e) => {
                    #[cfg(feature = "logging")]
                    log::warn!(
                        "Failed to parse record at tag {}: {}",
                        self.current_tag - 1,
                        e
                    );
                    continue;
                }
            }
        }
    }

    /// Returns a reference to the table information.
    pub fn table_info(&self) -> &TableInfo {
        self.table_info
    }
}
