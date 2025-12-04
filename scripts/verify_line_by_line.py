#!/usr/bin/env python3
"""
Line-by-line JSONL verification with normalization
"""

import json
from pathlib import Path
from collections import defaultdict

def normalize_guid(guid_str):
    """Convert GUID to consistent format (raw hex)"""
    if '-' in guid_str:
        # Rust format: ad495fc3-0eaa-413d-ba7d-8b13fa7ec598
        # Convert to raw hex
        return guid_str.replace('-', '')
    return guid_str

def normalize_row(row):
    """Normalize a row for comparison"""
    normalized = {}
    for key, value in row.items():
        # Normalize GUIDs
        if isinstance(value, str) and len(value) in [32, 36]:
            # Might be a GUID
            if '-' in value or all(c in '0123456789abcdefABCDEF-' for c in value):
                value = normalize_guid(value)
        normalized[key] = value
    return normalized

def compare_tables(table_name):
    """Compare a table line-by-line"""
    # Load files
    py_file = Path(f"python_exports/{table_name}.jsonl")
    rust_file = Path(f"rust_exports/{table_name}.jsonl")
    
    if not py_file.exists() or not rust_file.exists():
        return {'status': 'missing', 'table': table_name}
    
    # Load and normalize rows
    py_rows = []
    with open(py_file, 'r', encoding='utf-8') as f:
        for line in f:
            if line.strip():
                py_rows.append(normalize_row(json.loads(line)))
    
    rust_rows = []
    with open(rust_file, 'r', encoding='utf-8') as f:
        for line in f:
            if line.strip():
                rust_rows.append(normalize_row(json.loads(line)))
    
    # Sort for comparison
    def sort_key(row):
        if 'EntryId' in row:
            return row['EntryId']
        return json.dumps(row, sort_keys=True)
    
    py_sorted = sorted(py_rows, key=sort_key)
    rust_sorted = sorted(rust_rows, key=sort_key)
    
    # Compare
    if len(py_sorted) != len(rust_sorted):
        return {
            'status': 'count_mismatch',
            'table': table_name,
            'py_count': len(py_rows),
            'rust_count': len(rust_rows)
        }
    
    # Line-by-line comparison
    differences = []
    for i in range(len(py_sorted)):
        if py_sorted[i] != rust_sorted[i]:
            # Find which fields differ
            field_diffs = {}
            all_keys = set(py_sorted[i].keys()) | set(rust_sorted[i].keys())
            for key in all_keys:
                py_val = py_sorted[i].get(key)
                rust_val = rust_sorted[i].get(key)
                if py_val != rust_val:
                    field_diffs[key] = {
                        'python': py_val,
                        'rust': rust_val
                    }
            differences.append({
                'line': i,
                'fields': field_diffs
            })
    
    if differences:
        return {
            'status': 'data_mismatch',
            'table': table_name,
            'row_count': len(py_rows),
            'differences': differences
        }
    
    return {
        'status': 'perfect',
        'table': table_name,
        'row_count': len(py_rows)
    }

def main():
    """Verify all tables line-by-line"""
    py_dir = Path("python_exports")
    results = defaultdict(list)
    
    for py_file in sorted(py_dir.glob("*.jsonl")):
        table_name = py_file.stem
        result = compare_tables(table_name)
        results[result['status']].append(result)
        
        # Print progress
        if result['status'] == 'perfect':
            print(f"[OK] {table_name}: {result['row_count']} rows")
        elif result['status'] == 'count_mismatch':
            print(f"[COUNT] {table_name}: COUNT MISMATCH (Py={result['py_count']}, Rust={result['rust_count']})")
        elif result['status'] == 'data_mismatch':
            print(f"[DIFF] {table_name}: {len(result['differences'])} rows differ")
    
    # Summary
    print("\n" + "="*80)
    print("SUMMARY")
    print("="*80)
    print(f"Perfect: {len(results['perfect'])}")
    print(f"Count mismatches: {len(results['count_mismatch'])}")
    print(f"Data mismatches: {len(results['data_mismatch'])}")
    
    # Detailed report for mismatches
    if results['data_mismatch']:
        print("\n" + "="*80)
        print("DETAILED DATA MISMATCHES")
        print("="*80)
        for result in results['data_mismatch']:
            print(f"\n{result['table']}:")
            for diff in result['differences'][:3]:  # Show first 3
                print(f"  Line {diff['line']}:")
                for field, values in diff['fields'].items():
                    print(f"    {field}:")
                    print(f"      Python: {values['python']!r}")
                    print(f"      Rust:   {values['rust']!r}")
            if len(result['differences']) > 3:
                print(f"  ... and {len(result['differences']) - 3} more differences")

if __name__ == '__main__':
    main()
