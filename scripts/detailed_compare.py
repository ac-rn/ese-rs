#!/usr/bin/env python3
"""
Detailed comparison of specific tables to show field-level differences
"""

import json
import sys
from pathlib import Path

def compare_table_detailed(table_name):
    """Compare a specific table and show detailed differences"""
    py_file = Path(f"python_exports/{table_name}.jsonl")
    rust_file = Path(f"rust_exports/{table_name}.jsonl")
    
    if not py_file.exists() or not rust_file.exists():
        print(f"Files not found for {table_name}")
        return
    
    # Load rows
    py_rows = []
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
    print(f"Python rows: {len(py_rows)}")
    print(f"Rust rows: {len(rust_rows)}")
    print()
    
    if len(py_rows) != len(rust_rows):
        print(f"ROW COUNT MISMATCH: Python={len(py_rows)}, Rust={len(rust_rows)}")
        print()
        
        # Show which rows are missing/extra
        if py_rows and 'EntryId' in py_rows[0]:
            py_ids = set(row.get('EntryId') for row in py_rows)
            rust_ids = set(row.get('EntryId') for row in rust_rows)
            
            missing = py_ids - rust_ids
            extra = rust_ids - py_ids
            
            if missing:
                print(f"Missing in Rust: {sorted(missing)}")
            if extra:
                print(f"Extra in Rust: {sorted(extra)}")
        return
    
    # Sort rows for comparison
    def sort_key(row):
        if 'EntryId' in row:
            return row['EntryId']
        return json.dumps(row, sort_keys=True)
    
    py_sorted = sorted(py_rows, key=sort_key)
    rust_sorted = sorted(rust_rows, key=sort_key)
    
    # Find differences
    diff_count = 0
    field_diffs = {}
    
    for i in range(len(py_sorted)):
        py_row = py_sorted[i]
        rust_row = rust_sorted[i]
        
        if py_row != rust_row:
            diff_count += 1
            
            # Find which fields differ
            all_keys = set(py_row.keys()) | set(rust_row.keys())
            for key in all_keys:
                py_val = py_row.get(key)
                rust_val = rust_row.get(key)
                
                if py_val != rust_val:
                    if key not in field_diffs:
                        field_diffs[key] = []
                    field_diffs[key].append({
                        'row': i,
                        'py': py_val,
                        'rust': rust_val
                    })
    
    if diff_count == 0:
        print("[OK] ALL ROWS MATCH PERFECTLY!")
    else:
        print(f"[DIFF] {diff_count} rows differ")
        print()
        print("Field-level differences:")
        for field, diffs in sorted(field_diffs.items()):
            print(f"\n  {field}: {len(diffs)} differences")
            # Show first 3 examples
            for diff in diffs[:3]:
                print(f"    Row {diff['row']}:")
                print(f"      Python: {diff['py']!r}")
                print(f"      Rust:   {diff['rust']!r}")
            if len(diffs) > 3:
                print(f"    ... and {len(diffs) - 3} more")

if __name__ == '__main__':
    tables = [
        "Current_CLIENTS",
        "Current_ROLE_ACCESS",
        "Windows_SystemIndex_1_DATA_59",
        "Windows_SystemIndex_1_Properties",
        "Windows_SystemIndex_PropertyStore",
    ]
    
    if len(sys.argv) > 1:
        tables = [sys.argv[1]]
    
    for table in tables:
        compare_table_detailed(table)
        print("\n" + "="*80 + "\n")
