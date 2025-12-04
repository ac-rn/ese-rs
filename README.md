# ESE-RS

High-performance Microsoft ESE (Extensible Storage Engine) database parser written in Rust with Python bindings.

## Features

- 🚀 **40x faster** than Impacket's Python implementations
- 🦀 **Memory-safe** Rust implementation
- 🐍 **Python bindings** via PyO3
- 📦 **Zero-copy parsing** where possible
- 🔧 **Cross-platform** (Windows, Linux, macOS)

## Installation

### Python

```bash
pip install ese-parser
```

### Rust

```toml
[dependencies]
ese-rs = "0.1"
```

## Quick Start

### Python

```python
from ese_parser import EseDatabase

# Open database
db = EseDatabase("database.edb")

# List tables
for table in db.get_tables():
    print(table)

# Read table
records = db.read_table("MSysObjects")
for record in records:
    print(record)
```

### Rust

```rust
use ese_rs::Database;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open("database.edb")?;
    
    let mut cursor = db.open_table(b"MSysObjects")?;
    while let Some(record) = cursor.next_row()? {
        println!("{:?}", record);
    }
    
    Ok(())
}
```

## Documentation

- [Python API Documentation](python/README.md)
- [Examples](examples/)

## Performance

Benchmark parsing 340,288+ records from 3 databases:

- **Python (Impacket)**: 82.12 seconds
- **Rust (ese-rs)**: 2.18 seconds
- **Speedup**: 37.69x

## Supported Database Types

- Windows Search (`.edb`)
- Active Directory (`.dit`)
- Exchange (`.edb`)
- SRUM (`SRUDB.dat`)
- WebCache (`WebCacheV*.dat`)
- Any ESE database (Windows 2003+)

## License

Dual-licensed under MIT OR Apache-2.0.

## Acknowledgments

Based on the ESE format specification and inspired by Impacket's ese.py implementation.
