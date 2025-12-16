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

## Forensic carving (UTF-16LE)

When strings exist in the database file but do not appear in parsed table output (e.g., deleted records or page slack), you can scan for UTF-16LE strings directly.

```python
from ese_parser import EseDatabase

db = EseDatabase("WebCacheV01.dat")

# Scan unreferenced page slack
hits = db.carve_utf16le_strings_scoped("slack", "hopto.org", min_chars=6, max_hits=50)
for h in hits:
    print(h)

# Scan only tag-referenced regions (closest to "live" record bytes)
hits = db.carve_utf16le_strings_scoped("tag_data", "hopto.org", min_chars=6, max_hits=50)
```

Supported scopes:

- `"slack"`: gaps not referenced by any tag (page slack)
- `"tag_data"`: only tag-referenced byte ranges
- `"all"`: entire page bytes
- `"lv_slack"`: slack but only on LONG_VALUE pages
- `"lv_all"`: entire LONG_VALUE pages

Each hit is returned as a dictionary with keys including:

- `page`, `offset`
- `slack_start`, `slack_end`
- `region_kind`
- `page_flags`, `page_type`
- `table` (best-effort guess; may be null)
- `text`
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
