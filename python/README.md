# ESE Parser - Python Bindings

Fast Microsoft ESE (Extensible Storage Engine) database parser with a Rust backend.

## Features

- 🚀 **High Performance**: 30-40x faster than pure Python implementations
- 🦀 **Rust Backend**: Memory-safe, zero-copy parsing where possible
- 🐍 **Pythonic API**: Easy to use, feels natural to Python developers
- 📦 **Type Hints**: Full type hint support for IDE autocomplete
- 🔧 **Cross-Platform**: Works on Windows, Linux, and macOS

## Installation

```bash
pip install ese-parser
```

### Building from Source

Requirements:
- Python 3.8+
- Rust toolchain (install from https://rustup.rs)

```bash
# Clone the repository
git clone https://github.com/ac-rn/ese-rs.git
cd ese-rs/python

# Install maturin
pip install maturin

# Build and install
maturin develop --release
```

## Quick Start

```python
from ese_parser import EseDatabase

# Open a database
db = EseDatabase("Current.mdb")

# List all tables
tables = db.get_tables()
print(f"Found {len(tables)} tables")

# Read a specific table
records = db.read_table("MSysObjects")
for record in records:
    print(record["Name"], record["Type"])

# Export to JSONL
db.export_table("MSysObjects", "output.jsonl")

# Export all tables
db.export_all("output_directory/")
```

## Usage Examples

### Context Manager

```python
with EseDatabase("SRUDB.dat") as db:
    tables = db.get_tables()
    for table_name in tables:
        print(f"Table: {table_name}")
        records = db.read_table(table_name)
        print(f"  Records: {len(records)}")
```

### Get Table Schema

```python
db = EseDatabase("Current.mdb")
schema = db.get_table_schema("MSysObjects")

for column in schema:
    print(f"{column['name']}: {column['type']} (nullable: {column['nullable']})")
```

### Export Data

```python
# Export a single table
db = EseDatabase("SRUDB.dat")
db.export_table("SruDbIdMapTable", "output.jsonl")

# Export all tables to a directory
db.export_all("exports/")
```

## API Reference

### `EseDatabase`

Main class for accessing ESE databases.

#### Constructor

```python
EseDatabase(path: str)
```

Opens an ESE database file.

**Parameters:**
- `path` (str): Path to the database file (.mdb, .edb, .dat)

**Raises:**
- `FileNotFoundError`: If the database file doesn't exist
- `IOError`: If the database cannot be opened or is invalid

#### Properties

- `path` (str): Path to the database file
- `page_size` (int): Database page size in bytes
- `total_pages` (int): Total number of pages in the database

#### Methods

##### `get_tables() -> List[str]`

Returns a list of all table names in the database.

##### `read_table(table_name: str) -> List[Dict[str, Any]]`

Reads all records from a table.

**Parameters:**
- `table_name` (str): Name of the table to read

**Returns:**
- List of records as dictionaries

**Raises:**
- `ValueError`: If the table doesn't exist
- `IOError`: If there's an error reading the table

##### `get_table_schema(table_name: str) -> List[Dict[str, Any]]`

Gets the schema (columns) for a table.

**Parameters:**
- `table_name` (str): Name of the table

**Returns:**
- List of column information dictionaries with keys:
  - `name`: Column name
  - `type`: Column type
  - `id`: Column identifier
  - `nullable`: Whether the column can be null

**Raises:**
- `ValueError`: If the table doesn't exist

##### `export_table(table_name: str, output_path: str) -> None`

Exports a table to JSONL format.

**Parameters:**
- `table_name` (str): Name of the table to export
- `output_path` (str): Path to the output JSONL file

**Raises:**
- `ValueError`: If the table doesn't exist
- `IOError`: If there's an error writing the file

##### `export_all(output_dir: str) -> None`

Exports all tables to JSONL files in a directory.

**Parameters:**
- `output_dir` (str): Directory to write JSONL files to

**Raises:**
- `IOError`: If there's an error creating the directory or writing files

## Performance

The Rust backend provides significant performance improvements over pure Python implementations:

- **30-40x faster** for parsing and reading tables
- **Lower memory usage** through zero-copy parsing
- **Efficient I/O** using memory-mapped files

## Supported Database Types

- Windows Search databases (`.edb`)
- Active Directory databases (`.dit`)
- Exchange databases (`.edb`)
- SRUM databases (`SRUDB.dat`)
- WebCache databases (`WebCacheV*.dat`)
- Any other ESE database files

## License

Dual-licensed under MIT OR Apache-2.0

## Contributing

Contributions are welcome! Please see the main repository for contribution guidelines.

## Links

- [GitHub Repository](https://github.com/ac-rn/ese-rs)
- [Issue Tracker](https://github.com/ac-rn/ese-rs/issues)
