# ESE Parser Validation Test Suite

This directory contains scripts for comprehensive validation testing of all three ESE parser implementations.

## Overview

The test suite validates that three different implementations produce identical results:
1. **ese.py** - Original Python implementation (Impacket-based)
2. **Rust** - Native Rust implementation
3. **Python Bindings** - Python wrapper around the Rust implementation

## Test Databases

The tests run against 4 ESE databases:
- `Current.mdb`
- `SRUDB.dat`
- `WebCacheV01.dat`
- `Windows.edb`

## Quick Start

Run the complete validation test:

```bash
python scripts/run_full_test.py
```

This will:
1. Export all tables from all 4 databases using each implementation
2. Compare the outputs for consistency
3. Report any differences

## Individual Scripts

### Export Scripts

**Export using ese.py (original Python):**
```bash
python scripts/export_ese_py.py
```
Output: `ese_py_exports/`

**Export using Rust:**
```bash
cargo run --release --example export_all_rust
```
Output: `rust_exports/`

**Export using Python bindings:**
```bash
python scripts/export_python_bindings.py
```
Output: `python_bindings_exports/`

### Comparison Script

**Compare all three versions:**
```bash
python scripts/compare_all_three.py
```

This performs:
- Row count validation
- Sort-agnostic data comparison
- Detailed difference reporting

### Cleanup

**Remove all test artifacts:**
```bash
python scripts/cleanup_test_artifacts.py
```

This removes:
- `ese_py_exports/`
- `rust_exports/`
- `python_bindings_exports/`

## Output Format

All exports use JSONL (JSON Lines) format:
- One JSON object per line
- Keys sorted alphabetically for consistency
- Binary data hex-encoded
- Null terminators stripped from strings

## Validation Criteria

The comparison validates:

1. **Row Count**: Each table must have the same number of rows across all implementations
2. **Data Integrity**: Every row from ese.py must exist in both Rust and Python bindings
3. **Sort Agnostic**: Comparison handles different row ordering

## Prerequisites

### For ese.py export:
```bash
pip install impacket
```

### For Python bindings export:
```bash
pip install maturin
maturin develop --release
```

### For Rust export:
```bash
cargo build --release
```

## Expected Results

A successful test run will show:
```
🎉 SUCCESS! ALL THREE VERSIONS MATCH PERFECTLY! 🎉

All three parser implementations produce identical results:
  ✓ ese.py (original Python)
  ✓ Rust implementation
  ✓ Python bindings

You are ready to publish to GitHub and PyPI!
```

## Troubleshooting

### Missing databases
If some databases are missing, the test will skip them and continue with available ones.

### Python bindings not built
If Python bindings aren't available, `run_full_test.py` will attempt to build them automatically using maturin.

### Data mismatches
If differences are detected, the comparison script will show:
- Which tables have differences
- Row count mismatches
- Which implementations differ

## Files

- `run_full_test.py` - Master script to run all tests
- `export_ese_py.py` - Export using original Python parser
- `export_python_bindings.py` - Export using Python bindings
- `compare_all_three.py` - Compare all three versions
- `cleanup_test_artifacts.py` - Clean up test outputs
- `TEST_README.md` - This file
