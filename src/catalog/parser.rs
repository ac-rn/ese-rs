//! Catalog parser for extracting table metadata.

use crate::catalog::entry::{
    CatalogDataDefinitionEntry, CatalogEntryDetails, DataDefinitionHeader,
};
use crate::catalog::table_info::{ColumnInfo, IndexInfo, LongValueInfo, TableInfo};
use crate::constants::{is_large_page_format, page_flags};
use crate::error::{EseError, Result};
use crate::page::{BranchEntry, LeafEntry, Page};
use indexmap::IndexMap;

/// Parses catalog pages to extract table metadata.
pub struct CatalogParser<'a> {
    get_page_fn: &'a dyn Fn(u32) -> Result<Page<'a>>,
    version: u32,
    revision: u32,
    page_size: u32,
}

impl<'a> CatalogParser<'a> {
    /// Creates a new catalog parser.
    pub fn new(
        get_page_fn: &'a dyn Fn(u32) -> Result<Page<'a>>,
        version: u32,
        revision: u32,
        page_size: u32,
    ) -> Self {
        CatalogParser {
            get_page_fn,
            version,
            revision,
            page_size,
        }
    }

    /// Parses the catalog starting from the given page number.
    pub fn parse(&self, page_num: u32) -> Result<IndexMap<Vec<u8>, TableInfo>> {
        let mut tables = IndexMap::new();
        // Map from object ID to table name for column assignment
        let mut objid_to_table: IndexMap<u32, Vec<u8>> = IndexMap::new();

        // Store pending columns/indexes/longvalues that reference tables not yet seen
        let mut pending_entries: Vec<(
            u32,
            Vec<u8>,
            CatalogDataDefinitionEntry,
            CatalogEntryDetails,
        )> = Vec::new();

        self.parse_catalog_page(
            page_num,
            &mut tables,
            &mut objid_to_table,
            &mut pending_entries,
        )?;

        // Second pass: process pending entries now that all tables are registered
        for (father_id, item_name, catalog_entry, details) in pending_entries {
            self.assign_entry_to_table(
                father_id,
                item_name,
                catalog_entry,
                details,
                &mut tables,
                &objid_to_table,
            )?;
        }

        Ok(tables)
    }

    /// Recursively parses catalog pages.
    fn parse_catalog_page(
        &self,
        page_num: u32,
        tables: &mut IndexMap<Vec<u8>, TableInfo>,
        objid_to_table: &mut IndexMap<u32, Vec<u8>>,
        pending_entries: &mut Vec<(
            u32,
            Vec<u8>,
            CatalogDataDefinitionEntry,
            CatalogEntryDetails,
        )>,
    ) -> Result<()> {
        let page = (self.get_page_fn)(page_num)?;
        let extractor = page.tag_extractor(self.version, self.revision, self.page_size);

        // CRITICAL: Match Python's EXACT traversal order
        // PASS 1: Process ALL leaf entries on this page FIRST
        // PASS 2: Then recurse to branch children
        // This matches Python's parseCatalog() logic

        for tag_num in 0..extractor.num_tags() {
            // For large pages (>8KB), we need to use extract_tag_owned to get data with flags cleared
            let (flags, data_vec);
            let data: &[u8] = if is_large_page_format(self.version, self.revision, self.page_size) {
                let result = extractor.extract_tag_owned(tag_num).map_err(|e| {
                    EseError::Parse(format!(
                        "Failed to extract tag {} from page {}: {}",
                        tag_num, page_num, e
                    ))
                })?;
                flags = result.0;
                data_vec = result.1;
                &data_vec
            } else {
                let result = extractor.extract_tag(tag_num).map_err(|e| {
                    EseError::Parse(format!(
                        "Failed to extract tag {} from page {}: {}",
                        tag_num, page_num, e
                    ))
                })?;
                flags = result.0;
                result.1
            };

            // Skip empty tags
            if data.is_empty() {
                continue;
            }

            if page.common().page_flags & page_flags::LEAF != 0 {
                // Leaf page - check what type before parsing
                let page_flags_val = page.common().page_flags;

                // Skip special page types (matching Python behavior)
                if page_flags_val & page_flags::SPACE_TREE != 0 {
                    // Space tree page - skip for catalog parsing
                    continue;
                } else if page_flags_val & page_flags::INDEX != 0 {
                    // Index page - skip for catalog parsing
                    continue;
                } else if page_flags_val & page_flags::LONG_VALUE != 0 {
                    // Long value page - skip for catalog parsing
                    continue;
                }

                // Regular catalog leaf entry
                // Try to parse as leaf entry - if it fails, it might be metadata, skip it
                match LeafEntry::parse(flags, data) {
                    Ok(leaf_entry) => {
                        // Process ALL entries, even with empty entry_data
                        // Empty entry_data can be valid (e.g., tables with only inherited columns)
                        if let Err(e) = self.process_catalog_entry(
                            &leaf_entry.entry_data,
                            tables,
                            objid_to_table,
                            pending_entries,
                        ) {
                            // For large pages, catalog entry parsing might fail due to format differences
                            // Log but don't fail the entire parse
                            if self.page_size > 8192 {
                                // Skip this entry for now - large page format needs more work
                                continue;
                            } else {
                                return Err(e);
                            }
                        }
                    }
                    Err(_) => {
                        // Failed to parse - likely metadata tag, skip it
                        continue;
                    }
                }
            }
        }

        // PASS 2: Now process branches and recurse (matching Python's second loop)
        for tag_num in 0..extractor.num_tags() {
            let (flags, data_vec);
            let data: &[u8] = if is_large_page_format(self.version, self.revision, self.page_size) {
                match extractor.extract_tag_owned(tag_num) {
                    Ok(result) => {
                        flags = result.0;
                        data_vec = result.1;
                        &data_vec
                    }
                    Err(_) => continue,
                }
            } else {
                match extractor.extract_tag(tag_num) {
                    Ok(result) => {
                        flags = result.0;
                        result.1
                    }
                    Err(_) => continue,
                }
            };

            if data.is_empty() {
                continue;
            }

            // Only process branches if this is a branch page
            if page.common().page_flags & page_flags::LEAF == 0 {
                // Try to parse as branch entry and recurse
                if let Ok(branch_entry) = BranchEntry::parse(flags, data) {
                    self.parse_catalog_page(
                        branch_entry.child_page_number,
                        tables,
                        objid_to_table,
                        pending_entries,
                    )?;
                }
            }
        }

        Ok(())
    }

    /// Processes a single catalog entry.
    fn process_catalog_entry(
        &self,
        entry_data: &[u8],
        tables: &mut IndexMap<Vec<u8>, TableInfo>,
        objid_to_table: &mut IndexMap<u32, Vec<u8>>,
        pending_entries: &mut Vec<(
            u32,
            Vec<u8>,
            CatalogDataDefinitionEntry,
            CatalogEntryDetails,
        )>,
    ) -> Result<()> {
        // Empty entry data might be valid - e.g., tables with only inherited columns
        // Try to process it, but return Ok if it's truly empty
        if entry_data.is_empty() {
            // This might be a valid but minimal entry - for now skip it
            // TODO: investigate if we need special handling
            return Ok(());
        }

        // Check minimum size for header
        if entry_data.len() < DataDefinitionHeader::SIZE {
            // Not enough data for even the header - skip it
            return Ok(());
        }

        // Parse the data definition header
        let _header = DataDefinitionHeader::from_bytes(entry_data)?;
        let catalog_entry =
            CatalogDataDefinitionEntry::parse(&entry_data[DataDefinitionHeader::SIZE..])?;
        let item_name = CatalogDataDefinitionEntry::extract_item_name(entry_data)?;

        match catalog_entry.details {
            CatalogEntryDetails::Table {
                father_data_page_number,
                space_usage,
            } => {
                let table_info =
                    TableInfo::new(item_name.clone(), father_data_page_number, space_usage);
                // Map this table's identifier to its name for column assignment
                objid_to_table.insert(catalog_entry.identifier, item_name.clone());
                tables.insert(item_name, table_info);
            }

            CatalogEntryDetails::Column { .. }
            | CatalogEntryDetails::Index { .. }
            | CatalogEntryDetails::LongValue { .. } => {
                // Store for second pass - table might not be registered yet
                pending_entries.push((
                    catalog_entry.father_data_page_id,
                    item_name,
                    catalog_entry.clone(),
                    catalog_entry.details.clone(),
                ));
            }
        }

        Ok(())
    }

    /// Assigns a catalog entry to its table (used in second pass)
    fn assign_entry_to_table(
        &self,
        father_id: u32,
        item_name: Vec<u8>,
        catalog_entry: CatalogDataDefinitionEntry,
        details: CatalogEntryDetails,
        tables: &mut IndexMap<Vec<u8>, TableInfo>,
        objid_to_table: &IndexMap<u32, Vec<u8>>,
    ) -> Result<()> {
        if let Some(table_name) = objid_to_table.get(&father_id) {
            if let Some(table) = tables.get_mut(table_name) {
                match details {
                    CatalogEntryDetails::Column {
                        column_type,
                        space_usage,
                        column_flags,
                        code_page,
                    } => {
                        let column_info = ColumnInfo {
                            name: item_name.clone(),
                            identifier: catalog_entry.identifier,
                            column_type,
                            space_usage,
                            flags: column_flags,
                            code_page: Some(code_page),
                        };
                        table.columns.insert(item_name, column_info);
                    }
                    CatalogEntryDetails::Index {
                        father_data_page_number,
                        space_usage,
                        index_flags,
                        locale,
                    } => {
                        let index_info = IndexInfo {
                            name: item_name.clone(),
                            father_data_page_number,
                            space_usage,
                            flags: index_flags,
                            locale,
                        };
                        table.indexes.insert(item_name, index_info);
                    }
                    CatalogEntryDetails::LongValue {
                        father_data_page_number,
                        space_usage,
                    } => {
                        let lv_info = LongValueInfo {
                            name: item_name.clone(),
                            father_data_page_number,
                            space_usage,
                        };
                        table.long_values.insert(item_name, lv_info);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
