//! # ESE-RS: High-Performance ESE Database Parser
//!
//! A Rust implementation of a Microsoft ESE (Extensible Storage Engine) database parser,
//! designed for high performance and memory efficiency with 1-to-1 functional parity
//! with the Impacket Python implementation.
//!
//! ## Features
//!
//! - Zero-copy parsing where possible
//! - Memory-mapped I/O for efficient large file handling
//! - Type-safe parsing with comprehensive error handling
//! - Support for multiple ESE versions (Windows 2003+)
//!
//! ## Example
//!
//! ```no_run
//! use ese_rs::Database;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Open an ESE database
//! let db = Database::open("database.edb")?;
//!
//! // Print catalog information
//! db.print_catalog();
//!
//! // Open a table and iterate records
//! let mut cursor = db.open_table(b"MyTable")?;
//! while let Some(record) = cursor.next_row()? {
//!     println!("{:?}", record);
//! }
//! # Ok(())
//! # }
//! ```

pub mod catalog;
pub mod constants;
pub mod cursor;
pub mod database;
pub mod error;
pub mod header;
pub mod page;
pub mod record;
pub mod types;
pub mod utils;

// Python bindings (optional)
#[cfg(feature = "python")]
pub mod python;

// Re-export commonly used types
pub use constants::{CatalogType, CodePage, ColumnType};
pub use cursor::TableCursor;
pub use database::Database;
pub use error::{EseError, Result};
pub use types::ColumnValue;

// Python module initialization
#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pymodule]
fn ese_parser(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<python::PyEseDatabase>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
