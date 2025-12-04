#!/usr/bin/env python3
"""
Investigate tables where Rust finds more rows than Python
"""

import json
from pathlib import Path

def investigate_extra_rows(table_name):
    """Show what extra rows Rust has"""
    py_file = Path(f"python_exports/{table_name}.jsonl")
    rust_file = Path(f"rust_exports/{table_name}.jsonl")
    
    # Load rows
    py_rows = []
    if py_file.exists():
        with open(py_file, 'r', encoding='utf-8') as f:
            for line in f:
                if line.strip():
                    py_rows.append(json.loads(line))
    
    rust_rows = []
    with open(rust_file, 'r', encoding='utf-8') as f:
        for line in f:
            if line.strip():
                rust_rows.append(json.loads(line))
    
    print(f"Table: {table_name}")
    print(f"Python: {len(py_rows)} rows")
    print(f"Rust: {len(rust_rows)} rows")
    print(f"Extra in Rust: {len(rust_rows) - len(py_rows)} rows")
    print()
    
    # If there's an EntryId, show which IDs are extra
    if rust_rows and 'EntryId' in rust_rows[0]:
        py_ids = set(row.get('EntryId') for row in py_rows)
        rust_ids = set(row.get('EntryId') for row in rust_rows)
        extra_ids = rust_ids - py_ids
        
        print(f"Extra EntryIds in Rust: {sorted(extra_ids)}")
        print()
        
        # Show the extra rows
        for row in rust_rows:
            if row.get('EntryId') in extra_ids:
                print(f"Extra row (EntryId={row['EntryId']}):")
                for key, value in sorted(row.items()):
                    if value is not None:
                        val_str = str(value)[:100]
                        print(f"  {key}: {val_str}")
                print()
    else:
        # Show all Rust rows
        print("Rust rows:")
        for i, row in enumerate(rust_rows):
            print(f"\nRow {i}:")
            for key, value in sorted(row.items()):
                if value is not None:
                    val_str = str(value)[:100]
                    print(f"  {key}: {val_str}")

if __name__ == '__main__':
    tables = [
        "WebCacheV01_BlobEntry_1",
        "WebCacheV01_BlobEntry_5",
        "WebCacheV01_Container_2",
        "WebCacheV01_Container_3",
    ]
    
    for table in tables:
        investigate_extra_rows(table)
        print("="*80 + "\n")
