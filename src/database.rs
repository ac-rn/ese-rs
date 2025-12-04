//! Main database interface.

use crate::catalog::parser::CatalogParser;
use crate::catalog::table_info::TableInfo;
use crate::constants::CATALOG_PAGE_NUMBER;
use crate::cursor::TableCursor;
use crate::error::{EseError, Result};
use crate::header::DbHeader;
use crate::page::Page;
use indexmap::IndexMap;
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

/// ESE database handle.
///
/// This is the main entry point for accessing an ESE database.
pub struct Database {
    mmap: Mmap,
    header: DbHeader,
    page_size: u32,
    total_pages: u32,
    tables: IndexMap<Vec<u8>, TableInfo>,
}

impl Database {
    /// Opens an ESE database from a file path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the .edb database file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be opened
    /// - The file cannot be memory-mapped
    /// - The database header is invalid
    /// - The catalog cannot be parsed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ese_rs::Database;
    ///
    /// let db = Database::open("database.edb")?;
    /// # Ok::<(), ese_rs::EseError>(())
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        // Parse database header (first page, offset 0)
        if mmap.len() < 4096 {
            return Err(EseError::InvalidHeader);
        }

        // Copy header before creating Database struct to avoid borrow issues
        let header = {
            let header_data = &mmap[0..4096];
            *DbHeader::from_bytes(header_data)?
        };

        let page_size = header.page_size();
        let total_pages = (mmap.len() / page_size as usize).saturating_sub(2);

        let mut db = Database {
            mmap,
            header,
            page_size,
            total_pages: total_pages as u32,
            tables: IndexMap::new(),
        };

        // Parse catalog
        db.parse_catalog()?;

        Ok(db)
    }

    /// Parses the catalog to extract table metadata.
    fn parse_catalog(&mut self) -> Result<()> {
        let get_page_fn = |page_num: u32| -> Result<Page> {
            let page_data = self.get_page_data(page_num)?;
            Page::parse(
                page_data,
                self.header.version(),
                self.header.file_format_revision(),
                self.page_size,
            )
        };

        let parser = CatalogParser::new(
            &get_page_fn,
            self.header.version(),
            self.header.file_format_revision(),
            self.page_size,
        );

        self.tables = parser.parse(CATALOG_PAGE_NUMBER)?;

        Ok(())
    }

    /// Returns a reference to the database header.
    pub fn header(&self) -> &DbHeader {
        &self.header
    }

    /// Returns the page size in bytes.
    pub fn page_size(&self) -> u32 {
        self.page_size
    }

    /// Returns the total number of pages in the database.
    pub fn total_pages(&self) -> u32 {
        self.total_pages
    }

    /// Returns a reference to the tables map.
    pub fn tables(&self) -> &IndexMap<Vec<u8>, TableInfo> {
        &self.tables
    }

    /// Opens a table and returns a cursor for iteration.
    ///
    /// # Arguments
    ///
    /// * `table_name` - Name of the table as bytes
    ///
    /// # Errors
    ///
    /// Returns an error if the table is not found or cannot be opened.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ese_rs::Database;
    ///
    /// let db = Database::open("database.edb")?;
    /// let mut cursor = db.open_table(b"MyTable")?;
    /// while let Some(record) = cursor.next_row()? {
    ///     println!("{:?}", record);
    /// }
    /// # Ok::<(), ese_rs::EseError>(())
    /// ```
    pub fn open_table(&self, table_name: &[u8]) -> Result<TableCursor> {
        let table_info = self
            .tables
            .get(table_name)
            .ok_or_else(|| EseError::TableNotFound(table_name.to_vec()))?;

        TableCursor::new(self, table_info)
    }

    /// Prints the database catalog to stdout.
    ///
    /// This displays all tables, their columns, and indexes.
    pub fn print_catalog(&self) {
        println!("Database version: {}", self.header.version_string());
        println!("Page size: {}", self.page_size);
        println!("Number of pages: {}", self.total_pages);
        println!();
        println!("Catalog:");

        for (table_name, table_info) in &self.tables {
            println!("[{}]", String::from_utf8_lossy(table_name));
            println!("    Columns:");
            for (col_name, col_info) in &table_info.columns {
                println!(
                    "      {:5} {:30} {}",
                    col_info.identifier,
                    String::from_utf8_lossy(col_name),
                    col_info.column_type.name()
                );
            }
            println!("    Indexes:");
            for index_name in table_info.indexes.keys() {
                println!("      {}", String::from_utf8_lossy(index_name));
            }
            println!();
        }
    }

    /// Gets page data by page number.
    ///
    /// Page numbers are 1-indexed (page 1 is the database header).
    /// This function returns a zero-copy slice into the memory-mapped file.
    pub(crate) fn get_page_data(&self, page_num: u32) -> Result<&[u8]> {
        let offset = ((page_num + 1) * self.page_size) as usize;
        let end = offset + self.page_size as usize;

        if end > self.mmap.len() {
            return Err(EseError::InvalidPageNumber(page_num));
        }

        Ok(&self.mmap[offset..end])
    }

    /// Gets a parsed page by page number.
    pub(crate) fn get_page(&self, page_num: u32) -> Result<Page> {
        let data = self.get_page_data(page_num)?;
        Page::parse(
            data,
            self.header.version(),
            self.header.file_format_revision(),
            self.page_size,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_header_size() {
        // Verify the header size is reasonable
        assert_eq!(DbHeader::SIZE, std::mem::size_of::<DbHeader>());
    }
}
