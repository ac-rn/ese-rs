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

#[derive(Debug, Clone)]
pub struct CarvedString {
    pub page_number: u32,
    pub offset_in_page: usize,
    pub slack_start: usize,
    pub slack_end: usize,
    pub region_kind: String,
    pub page_flags: u32,
    pub page_type: String,
    pub table: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarveScope {
    Slack,
    All,
    TagData,
    LongValueAll,
    LongValueSlack,
}

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

    pub fn carve_utf16le_strings(
        &self,
        needle: Option<&str>,
        min_chars: usize,
        max_hits: usize,
    ) -> Result<Vec<CarvedString>> {
        self.carve_utf16le_strings_scoped(CarveScope::Slack, needle, min_chars, max_hits)
    }

    pub fn carve_utf16le_strings_scoped(
        &self,
        scope: CarveScope,
        needle: Option<&str>,
        min_chars: usize,
        max_hits: usize,
    ) -> Result<Vec<CarvedString>> {
        let mut hits = Vec::new();

        let min_chars = min_chars.max(2);
        let max_hits = max_hits.max(1);

        for page_number in 1..=self.total_pages {
            if hits.len() >= max_hits {
                break;
            }

            let page = match self.get_page(page_number) {
                Ok(p) => p,
                Err(_) => continue,
            };

            if matches!(scope, CarveScope::LongValueAll | CarveScope::LongValueSlack)
                && !page.is_long_value()
            {
                continue;
            }

            let extractor = page.tag_extractor(
                self.header.version(),
                self.header.file_format_revision(),
                self.page_size,
            );

            let page_flags = page.common().page_flags;
            let page_type = page_type_string(&page);

            let table = self.guess_table_for_page(page_number);

            let page_len = page.data.len();
            let num_tags = extractor.num_tags() as usize;
            let tag_array_start = page_len.saturating_sub(num_tags.saturating_mul(4));

            match scope {
                CarveScope::All | CarveScope::LongValueAll => {
                    carve_utf16le_from_range(
                        page_number,
                        page.data,
                        0,
                        page_len,
                        "all",
                        page_flags,
                        &page_type,
                        table.as_deref(),
                        needle,
                        min_chars,
                        max_hits,
                        &mut hits,
                    );
                }
                CarveScope::TagData => {
                    for tag_num in 0..extractor.num_tags() {
                        if hits.len() >= max_hits {
                            break;
                        }
                        if let Ok((_flags, start, end)) = extractor.extract_tag_bounds(tag_num) {
                            if start < end && end <= page_len {
                                carve_utf16le_from_range(
                                    page_number,
                                    page.data,
                                    start,
                                    end,
                                    "tag_data",
                                    page_flags,
                                    &page_type,
                                    table.as_deref(),
                                    needle,
                                    min_chars,
                                    max_hits,
                                    &mut hits,
                                );
                            }
                        }
                    }
                }
                CarveScope::Slack | CarveScope::LongValueSlack => {
                    let mut used_ranges: Vec<(usize, usize)> = Vec::new();
                    used_ranges.push((0, page.header_len.min(page_len)));
                    used_ranges.push((tag_array_start, page_len));

                    for tag_num in 0..extractor.num_tags() {
                        if let Ok((_flags, start, end)) = extractor.extract_tag_bounds(tag_num) {
                            if start < end && end <= page_len {
                                used_ranges.push((start, end));
                            }
                        }
                    }

                    used_ranges.sort_by_key(|r| r.0);
                    let mut merged: Vec<(usize, usize)> = Vec::new();
                    for (s, e) in used_ranges {
                        if let Some(last) = merged.last_mut() {
                            if s <= last.1 {
                                last.1 = last.1.max(e);
                                continue;
                            }
                        }
                        merged.push((s, e));
                    }

                    let mut cursor = 0usize;
                    for (s, e) in merged {
                        if cursor < s {
                            carve_utf16le_from_range(
                                page_number,
                                page.data,
                                cursor,
                                s,
                                "slack",
                                page_flags,
                                &page_type,
                                table.as_deref(),
                                needle,
                                min_chars,
                                max_hits,
                                &mut hits,
                            );
                            if hits.len() >= max_hits {
                                break;
                            }
                        }
                        cursor = cursor.max(e);
                    }

                    if hits.len() < max_hits && cursor < page_len {
                        carve_utf16le_from_range(
                            page_number,
                            page.data,
                            cursor,
                            page_len,
                            "slack",
                            page_flags,
                            &page_type,
                            table.as_deref(),
                            needle,
                            min_chars,
                            max_hits,
                            &mut hits,
                        );
                    }
                }
            }
        }

        Ok(hits)
    }

    fn guess_table_for_page(&self, page_number: u32) -> Option<String> {
        // Best-effort: only attempt to map DATA leaf pages. Many pages (LV, index, space tree)
        // or reused/free pages won't be attributable.
        let page = self.get_page(page_number).ok()?;
        if !page.is_leaf() || page.is_space_tree() || page.is_index() || page.is_long_value() {
            return None;
        }

        for (table_name, table_info) in &self.tables {
            let mut pnum = table_info.father_data_page_number;
            let mut p = match self.get_page(pnum) {
                Ok(x) => x,
                Err(_) => continue,
            };

            // Descend to first leaf page (same strategy as TableCursor::new)
            while !p.is_leaf() {
                let extractor = p.tag_extractor(
                    self.header.version(),
                    self.header.file_format_revision(),
                    self.page_size,
                );

                if extractor.num_tags() <= 1 {
                    break;
                }

                let (flags, data) = match extractor.extract_tag(1) {
                    Ok(r) => r,
                    Err(_) => break,
                };

                let branch_entry = match crate::page::BranchEntry::parse(flags, data) {
                    Ok(be) => be,
                    Err(_) => break,
                };

                pnum = branch_entry.child_page_number;
                p = match self.get_page(pnum) {
                    Ok(x) => x,
                    Err(_) => break,
                };
            }

            // Walk leaf chain via next_page_number, looking for our page number.
            let mut current = pnum;
            let mut guard = 0u32;
            while current != 0 {
                if current == page_number {
                    return Some(String::from_utf8_lossy(table_name).to_string());
                }

                let leaf_page = match self.get_page(current) {
                    Ok(x) => x,
                    Err(_) => break,
                };
                let next = leaf_page.common().next_page_number;
                current = next;

                guard += 1;
                if guard > self.total_pages {
                    break;
                }
            }
        }

        None
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

fn carve_utf16le_from_range(
    page_number: u32,
    page_data: &[u8],
    start: usize,
    end: usize,
    region_kind: &str,
    page_flags: u32,
    page_type: &str,
    table: Option<&str>,
    needle: Option<&str>,
    min_chars: usize,
    max_hits: usize,
    hits: &mut Vec<CarvedString>,
) {
    if start >= end {
        return;
    }

    let mut i = start;
    while i + 2 <= end {
        if hits.len() >= max_hits {
            return;
        }

        // Keep word alignment (UTF-16LE).
        if i % 2 != 0 {
            i += 1;
            continue;
        }

        let mut chars: Vec<u16> = Vec::new();
        let mut j = i;
        while j + 2 <= end {
            let w = u16::from_le_bytes([page_data[j], page_data[j + 1]]);
            if w == 0 {
                break;
            }
            if !is_plausible_utf16le_char(w) {
                break;
            }
            chars.push(w);
            j += 2;
        }

        if chars.len() >= min_chars {
            let text = String::from_utf16_lossy(&chars);
            let ok = needle.map(|n| text.contains(n)).unwrap_or(true);
            if ok {
                hits.push(CarvedString {
                    page_number,
                    offset_in_page: i,
                    slack_start: start,
                    slack_end: end,
                    region_kind: region_kind.to_string(),
                    page_flags,
                    page_type: page_type.to_string(),
                    table: table.map(|t| t.to_string()),
                    text,
                });
                if hits.len() >= max_hits {
                    return;
                }
            }

            // Skip past this run (+ optional terminator)
            i = j.saturating_add(2);
        } else {
            i += 2;
        }
    }
}

fn is_plausible_utf16le_char(w: u16) -> bool {
    // Conservative: printable ASCII + space. Good enough for URLs/hostnames.
    matches!(w, 0x20..=0x7e)
}

fn page_type_string(page: &Page<'_>) -> String {
    if page.is_long_value() {
        return "long_value".to_string();
    }
    if page.is_space_tree() {
        return "space_tree".to_string();
    }
    if page.is_index() {
        return "index".to_string();
    }
    if page.is_leaf() {
        return "leaf".to_string();
    }
    "branch".to_string()
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
