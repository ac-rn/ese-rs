//! Python bindings for the ESE parser using PyO3.
//!
//! This module provides a Pythonic interface to the Rust ESE parser.

#![allow(deprecated)]

use indexmap::IndexMap;
use pyo3::exceptions::{PyFileNotFoundError, PyIOError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};
use std::path::PathBuf;

use crate::{ColumnValue, Database, EseError};

/// Convert Rust EseError to Python exception
impl From<EseError> for PyErr {
    fn from(err: EseError) -> PyErr {
        match err {
            EseError::Io(e) => PyIOError::new_err(e.to_string()),
            EseError::TableNotFound(name) => PyValueError::new_err(format!(
                "Table not found: {}",
                String::from_utf8_lossy(&name)
            )),
            _ => PyIOError::new_err(err.to_string()),
        }
    }
}

/// Python wrapper for ESE database.
///
/// This class provides access to Microsoft ESE databases from Python.
///
/// Example:
///     >>> from ese_parser import EseDatabase
///     >>> db = EseDatabase("Current.mdb")
///     >>> tables = db.get_tables()
///     >>> records = db.read_table("MSysObjects")
#[pyclass(name = "EseDatabase")]
pub struct PyEseDatabase {
    db: Database,
    path: PathBuf,
}

#[pymethods]
impl PyEseDatabase {
    /// Create a new ESE database instance.
    ///
    /// Args:
    ///     path: Path to the ESE database file (.mdb, .edb, .dat)
    ///
    /// Returns:
    ///     EseDatabase instance
    ///
    /// Raises:
    ///     FileNotFoundError: If the database file doesn't exist
    ///     IOError: If the database cannot be opened or is invalid
    #[new]
    fn new(path: &str) -> PyResult<Self> {
        let path_buf = PathBuf::from(path);

        if !path_buf.exists() {
            return Err(PyFileNotFoundError::new_err(format!(
                "Database file not found: {}",
                path
            )));
        }

        let db = Database::open(&path_buf)
            .map_err(|e| PyIOError::new_err(format!("Failed to open database: {}", e)))?;

        Ok(PyEseDatabase { db, path: path_buf })
    }

    /// Get the path to the database file.
    ///
    /// Returns:
    ///     str: Path to the database file
    #[getter]
    fn path(&self) -> String {
        self.path.to_string_lossy().to_string()
    }

    /// Get the database page size.
    ///
    /// Returns:
    ///     int: Page size in bytes
    #[getter]
    fn page_size(&self) -> u32 {
        self.db.page_size()
    }

    /// Get the total number of pages in the database.
    ///
    /// Returns:
    ///     int: Total number of pages
    #[getter]
    fn total_pages(&self) -> u32 {
        self.db.total_pages()
    }

    /// Get a list of all table names in the database.
    ///
    /// Returns:
    ///     list[str]: List of table names
    fn get_tables(&self) -> Vec<String> {
        self.db
            .tables()
            .keys()
            .map(|name| String::from_utf8_lossy(name).to_string())
            .collect()
    }

    /// Read all records from a table.
    ///
    /// Args:
    ///     table_name: Name of the table to read
    ///
    /// Returns:
    ///     list[dict]: List of records as dictionaries
    ///
    /// Raises:
    ///     ValueError: If the table doesn't exist
    ///     IOError: If there's an error reading the table
    fn read_table(&self, py: Python, table_name: &str) -> PyResult<PyObject> {
        let table_name_bytes = table_name.as_bytes();
        let mut cursor = self.db.open_table(table_name_bytes)?;

        let records = PyList::empty_bound(py);

        while let Some(record) = cursor
            .next_row()
            .map_err(|e| PyIOError::new_err(format!("Error reading record: {}", e)))?
        {
            let py_record = record_to_pydict(py, &record)?;
            records.append(py_record)?;
        }

        Ok(records.into())
    }

    /// Get the schema (columns) for a table.
    ///
    /// Args:
    ///     table_name: Name of the table
    ///
    /// Returns:
    ///     list[dict]: List of column information with 'name', 'type', 'id'
    ///
    /// Raises:
    ///     ValueError: If the table doesn't exist
    fn get_table_schema(&self, py: Python, table_name: &str) -> PyResult<PyObject> {
        let table_name_bytes = table_name.as_bytes();
        let table_info = self
            .db
            .tables()
            .get(table_name_bytes)
            .ok_or_else(|| PyValueError::new_err(format!("Table not found: {}", table_name)))?;

        let columns = PyList::empty_bound(py);

        for (col_name, col_info) in &table_info.columns {
            let col_dict = PyDict::new_bound(py);
            col_dict.set_item("name", String::from_utf8_lossy(col_name).to_string())?;
            col_dict.set_item("type", col_info.column_type.name())?;
            col_dict.set_item("id", col_info.identifier)?;
            // Check if NOT NULL flag is set (flag bit 1)
            let nullable = (col_info.flags & 0x01) == 0;
            col_dict.set_item("nullable", nullable)?;
            columns.append(col_dict)?;
        }

        Ok(columns.into())
    }

    /// Export a table to JSONL format.
    ///
    /// Args:
    ///     table_name: Name of the table to export
    ///     output_path: Path to the output JSONL file
    ///
    /// Raises:
    ///     ValueError: If the table doesn't exist
    ///     IOError: If there's an error writing the file
    fn export_table(&self, table_name: &str, output_path: &str) -> PyResult<()> {
        use std::fs::File;
        use std::io::Write;

        let table_name_bytes = table_name.as_bytes();
        let mut cursor = self.db.open_table(table_name_bytes)?;

        let mut file = File::create(output_path)
            .map_err(|e| PyIOError::new_err(format!("Failed to create output file: {}", e)))?;

        while let Some(record) = cursor
            .next_row()
            .map_err(|e| PyIOError::new_err(format!("Error reading record: {}", e)))?
        {
            let json_record = record_to_json_map(&record);
            let json_str = serde_json::to_string(&json_record)
                .map_err(|e| PyIOError::new_err(format!("JSON serialization error: {}", e)))?;

            writeln!(file, "{}", json_str)
                .map_err(|e| PyIOError::new_err(format!("Write error: {}", e)))?;
        }

        Ok(())
    }

    /// Export all tables to JSONL files in a directory.
    ///
    /// Args:
    ///     output_dir: Directory to write JSONL files to
    ///
    /// Raises:
    ///     IOError: If there's an error creating the directory or writing files
    fn export_all(&self, output_dir: &str) -> PyResult<()> {
        use std::fs;
        use std::path::Path;

        let output_path = Path::new(output_dir);
        fs::create_dir_all(output_path)
            .map_err(|e| PyIOError::new_err(format!("Failed to create output directory: {}", e)))?;

        for table_name in self.get_tables() {
            let output_file = output_path.join(format!("{}.jsonl", table_name));
            let output_path_str = output_file
                .to_str()
                .ok_or_else(|| PyIOError::new_err("Invalid UTF-8 in output path"))?;
            self.export_table(&table_name, output_path_str)?;
        }

        Ok(())
    }

    /// Context manager support: enter
    fn __enter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    /// Context manager support: exit
    fn __exit__(
        &mut self,
        _exc_type: PyObject,
        _exc_value: PyObject,
        _traceback: PyObject,
    ) -> PyResult<bool> {
        Ok(false)
    }

    /// String representation
    fn __repr__(&self) -> String {
        format!(
            "EseDatabase(path='{}', tables={}, page_size={})",
            self.path.display(),
            self.db.tables().len(),
            self.db.page_size()
        )
    }
}

/// Convert a record (IndexMap) to a Python dictionary
fn record_to_pydict(py: Python, record: &IndexMap<Vec<u8>, ColumnValue>) -> PyResult<PyObject> {
    let dict = PyDict::new_bound(py);

    for (key, value) in record {
        let key_str = String::from_utf8_lossy(key).to_string();
        let py_value = column_value_to_pyobject(py, value)?;
        dict.set_item(key_str, py_value)?;
    }

    Ok(dict.into_any().unbind())
}

/// Convert a record to a JSON-serializable map
fn record_to_json_map(
    record: &IndexMap<Vec<u8>, ColumnValue>,
) -> serde_json::Map<String, serde_json::Value> {
    record
        .iter()
        .map(|(k, v)| {
            (
                String::from_utf8_lossy(k).to_string(),
                column_value_to_json(v),
            )
        })
        .collect()
}

/// Convert ColumnValue to Python object
fn column_value_to_pyobject(py: Python, value: &ColumnValue) -> PyResult<PyObject> {
    let obj = match value {
        ColumnValue::Null => py.None(),
        ColumnValue::Boolean(b) => b.to_object(py),
        ColumnValue::I8(n) => n.to_object(py),
        ColumnValue::U8(n) => n.to_object(py),
        ColumnValue::I16(n) => n.to_object(py),
        ColumnValue::I32(n) => n.to_object(py),
        ColumnValue::I64(n) => n.to_object(py),
        ColumnValue::U16(n) => n.to_object(py),
        ColumnValue::U32(n) => n.to_object(py),
        ColumnValue::U64(n) => n.to_object(py),
        ColumnValue::F32(f) => f.to_object(py),
        ColumnValue::F64(f) => f.to_object(py),
        ColumnValue::DateTime(dt) => dt.to_object(py),
        ColumnValue::Currency(c) => c.to_object(py),
        ColumnValue::Guid(bytes) => {
            // Format GUID as string with RFC 4122 little-endian byte order
            // GUIDs in ESE are stored in little-endian format, so we need to swap bytes
            let guid_str = format!(
                "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                bytes[3], bytes[2], bytes[1], bytes[0],  // First 4 bytes (little-endian)
                bytes[5], bytes[4],                       // Next 2 bytes (little-endian)
                bytes[7], bytes[6],                       // Next 2 bytes (little-endian)
                bytes[8], bytes[9],                       // Last 8 bytes (big-endian)
                bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
            );
            guid_str.to_object(py)
        }
        ColumnValue::Binary(b) => PyBytes::new_bound(py, b).to_object(py),
        ColumnValue::Text(s) => s.to_object(py),
        ColumnValue::MultiValue(b) => PyBytes::new_bound(py, b).to_object(py),
        ColumnValue::LongValue(b) => PyBytes::new_bound(py, b).to_object(py),
    };
    Ok(obj)
}

/// Convert ColumnValue to JSON value
fn column_value_to_json(value: &ColumnValue) -> serde_json::Value {
    match value {
        ColumnValue::Null => serde_json::Value::Null,
        ColumnValue::Boolean(b) => serde_json::Value::Bool(*b),
        ColumnValue::I8(n) => serde_json::Value::Number((*n).into()),
        ColumnValue::U8(n) => serde_json::Value::Number((*n).into()),
        ColumnValue::I16(n) => serde_json::Value::Number((*n).into()),
        ColumnValue::I32(n) => serde_json::Value::Number((*n).into()),
        ColumnValue::I64(n) => serde_json::Value::Number((*n).into()),
        ColumnValue::U16(n) => serde_json::Value::Number((*n).into()),
        ColumnValue::U32(n) => serde_json::Value::Number((*n).into()),
        ColumnValue::U64(n) => serde_json::Value::Number((*n).into()),
        ColumnValue::F32(f) => serde_json::Value::Number(
            serde_json::Number::from_f64(*f as f64).unwrap_or(serde_json::Number::from(0)),
        ),
        ColumnValue::F64(f) => serde_json::Value::Number(
            serde_json::Number::from_f64(*f).unwrap_or(serde_json::Number::from(0)),
        ),
        ColumnValue::DateTime(dt) => serde_json::Value::Number((*dt).into()),
        ColumnValue::Currency(c) => serde_json::Value::Number((*c).into()),
        ColumnValue::Guid(bytes) => {
            // Format GUID with RFC 4122 little-endian byte order
            let guid_str = format!(
                "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                bytes[3], bytes[2], bytes[1], bytes[0],  // First 4 bytes (little-endian)
                bytes[5], bytes[4],                       // Next 2 bytes (little-endian)
                bytes[7], bytes[6],                       // Next 2 bytes (little-endian)
                bytes[8], bytes[9],                       // Last 8 bytes (big-endian)
                bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
            );
            serde_json::Value::String(guid_str)
        }
        ColumnValue::Binary(b) => serde_json::Value::String(hex::encode(b)),
        ColumnValue::Text(s) => serde_json::Value::String(s.clone()),
        ColumnValue::MultiValue(b) => serde_json::Value::String(hex::encode(b)),
        ColumnValue::LongValue(b) => serde_json::Value::String(hex::encode(b)),
    }
}
