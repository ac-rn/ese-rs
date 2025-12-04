#!/usr/bin/env python3
"""
Compare all three parser versions: ese.py, Rust, and Python bindings
Validates that all three produce identical data.
"""

import json
from pathlib import Path
from collections import defaultdict
import sys

def normalize_value(value):
    """Normalize values for comparison."""
    if value is None:
        return None
    elif isinstance(value, str):
        # Remove trailing nulls and normalize
        return value.rstrip('\x00')
    elif isinstance(value, (int, float, bool)):
        return value
    else:
        return str(value)

def normalize_row(row):
    """Normalize a row for comparison."""
    return {k: normalize_value(v) for k, v in row.items()}

def load_jsonl(file_path):
    """Load JSONL file into list of rows."""
    rows = []
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            for line in f:
                if line.strip():
                    row = json.loads(line)
                    rows.append(normalize_row(row))
    except Exception as e:
        print(f"Error loading {file_path}: {e}")
        return None
    return rows

def rows_to_set(rows):
    """Convert rows to a set of JSON strings for comparison."""
    return {json.dumps(row, sort_keys=True) for row in rows}

def compare_three_versions(ese_py_file, rust_file, python_bindings_file):
    """Compare three versions of the same table."""
    # Load all three versions
    ese_py_rows = load_jsonl(ese_py_file) if ese_py_file.exists() else None
    rust_rows = load_jsonl(rust_file) if rust_file.exists() else None
    python_bindings_rows = load_jsonl(python_bindings_file) if python_bindings_file.exists() else None
    
    # Check if all files exist
    missing = []
    if ese_py_rows is None:
        missing.append("ese.py")
    if rust_rows is None:
        missing.append("rust")
    if python_bindings_rows is None:
        missing.append("python_bindings")
    
    if missing:
        return {
            'status': 'missing_files',
            'missing': missing
        }
    
    # Compare counts
    ese_py_count = len(ese_py_rows)
    rust_count = len(rust_rows)
    python_bindings_count = len(python_bindings_rows)
    
    if not (ese_py_count == rust_count == python_bindings_count):
        return {
            'status': 'count_mismatch',
            'ese_py_count': ese_py_count,
            'rust_count': rust_count,
            'python_bindings_count': python_bindings_count
        }
    
    # Convert to sets for order-agnostic comparison
    ese_py_set = rows_to_set(ese_py_rows)
    rust_set = rows_to_set(rust_rows)
    python_bindings_set = rows_to_set(python_bindings_rows)
    
    # Check if all sets are equal
    if ese_py_set == rust_set == python_bindings_set:
        return {
            'status': 'perfect',
            'row_count': ese_py_count
        }
    
    # Find differences
    result = {
        'status': 'data_mismatch',
        'row_count': ese_py_count,
        'differences': {}
    }
    
    # Check ese.py vs rust
    if ese_py_set != rust_set:
        only_in_ese_py = ese_py_set - rust_set
        only_in_rust = rust_set - ese_py_set
        result['differences']['ese_py_vs_rust'] = {
            'only_in_ese_py': len(only_in_ese_py),
            'only_in_rust': len(only_in_rust)
        }
    
    # Check ese.py vs python_bindings
    if ese_py_set != python_bindings_set:
        only_in_ese_py = ese_py_set - python_bindings_set
        only_in_python_bindings = python_bindings_set - ese_py_set
        result['differences']['ese_py_vs_python_bindings'] = {
            'only_in_ese_py': len(only_in_ese_py),
            'only_in_python_bindings': len(only_in_python_bindings)
        }
    
    # Check rust vs python_bindings
    if rust_set != python_bindings_set:
        only_in_rust = rust_set - python_bindings_set
        only_in_python_bindings = python_bindings_set - rust_set
        result['differences']['rust_vs_python_bindings'] = {
            'only_in_rust': len(only_in_rust),
            'only_in_python_bindings': len(only_in_python_bindings)
        }
    
    return result

def main():
    ese_py_dir = Path("ese_py_exports")
    rust_dir = Path("rust_exports")
    python_bindings_dir = Path("python_bindings_exports")
    
    # Check if directories exist
    missing_dirs = []
    if not ese_py_dir.exists():
        missing_dirs.append("ese_py_exports")
    if not rust_dir.exists():
        missing_dirs.append("rust_exports")
    if not python_bindings_dir.exists():
        missing_dirs.append("python_bindings_exports")
    
    if missing_dirs:
        print("Error: Missing export directories:")
        for d in missing_dirs:
            print(f"  - {d}")
        print("\nPlease run the export scripts first:")
        print("  1. python scripts/export_ese_py.py")
        print("  2. cargo run --release --example export_all_rust")
        print("  3. python scripts/export_python_bindings.py")
        return 1
    
    # Get all files from ese.py exports (reference)
    ese_py_files = {f.name: f for f in ese_py_dir.glob("*.jsonl")}
    rust_files = {f.name: f for f in rust_dir.glob("*.jsonl")}
    python_bindings_files = {f.name: f for f in python_bindings_dir.glob("*.jsonl")}
    
    # Results tracking
    perfect_matches = []
    count_mismatches = []
    data_mismatches = []
    missing_files = []
    
    # Group by database
    by_database = defaultdict(lambda: {
        'perfect': [],
        'count_diff': [],
        'data_diff': [],
        'missing': []
    })
    
    print("Comparing exports...")
    print("=" * 80)
    
    for filename in sorted(ese_py_files.keys()):
        ese_py_file = ese_py_files[filename]
        rust_file = rust_files.get(filename, rust_dir / filename)
        python_bindings_file = python_bindings_files.get(filename, python_bindings_dir / filename)
        
        result = compare_three_versions(ese_py_file, rust_file, python_bindings_file)
        
        # Extract database name
        db_name = filename.split('_')[0]
        
        if result['status'] == 'perfect':
            perfect_matches.append((filename, result['row_count']))
            by_database[db_name]['perfect'].append((filename, result['row_count']))
            print(f"✓ {filename}: {result['row_count']} rows - PERFECT MATCH")
        elif result['status'] == 'count_mismatch':
            count_mismatches.append((filename, result))
            by_database[db_name]['count_diff'].append((filename, result))
            print(f"✗ {filename}: COUNT MISMATCH")
            print(f"    ese.py: {result['ese_py_count']}, rust: {result['rust_count']}, python_bindings: {result['python_bindings_count']}")
        elif result['status'] == 'data_mismatch':
            data_mismatches.append((filename, result))
            by_database[db_name]['data_diff'].append((filename, result))
            print(f"✗ {filename}: DATA MISMATCH")
            for comparison, diff in result['differences'].items():
                print(f"    {comparison}: {diff}")
        elif result['status'] == 'missing_files':
            missing_files.append((filename, result['missing']))
            by_database[db_name]['missing'].append((filename, result['missing']))
            print(f"✗ {filename}: MISSING FILES - {', '.join(result['missing'])}")
    
    # Print results by database
    print("\n" + "=" * 80)
    print("RESULTS BY DATABASE")
    print("=" * 80)
    
    for db_name in sorted(by_database.keys()):
        db_results = by_database[db_name]
        total = (len(db_results['perfect']) + len(db_results['data_diff']) + 
                 len(db_results['count_diff']) + len(db_results['missing']))
        perfect_pct = 100 * len(db_results['perfect']) / total if total > 0 else 0
        
        print(f"\n{db_name}:")
        print(f"  Perfect matches: {len(db_results['perfect'])}/{total} ({perfect_pct:.1f}%)")
        
        if db_results['count_diff']:
            print(f"  Count mismatches: {len(db_results['count_diff'])}")
        
        if db_results['data_diff']:
            print(f"  Data mismatches: {len(db_results['data_diff'])}")
        
        if db_results['missing']:
            print(f"  Missing files: {len(db_results['missing'])}")
    
    # Overall summary
    total = len(ese_py_files)
    print("\n" + "=" * 80)
    print("OVERALL SUMMARY")
    print("=" * 80)
    print(f"Total tables compared: {total}")
    print(f"Perfect matches: {len(perfect_matches)} ({100*len(perfect_matches)/total:.1f}%)")
    print(f"Count mismatches: {len(count_mismatches)} ({100*len(count_mismatches)/total:.1f}%)")
    print(f"Data mismatches: {len(data_mismatches)} ({100*len(data_mismatches)/total:.1f}%)")
    print(f"Missing files: {len(missing_files)} ({100*len(missing_files)/total:.1f}%)")
    
    # Success message
    if len(perfect_matches) == total:
        print("\n" + "=" * 80)
        print("🎉 SUCCESS! ALL THREE VERSIONS MATCH PERFECTLY! 🎉")
        print("=" * 80)
        return 0
    else:
        print("\n" + "=" * 80)
        print("⚠️  VALIDATION FAILED - DIFFERENCES DETECTED")
        print("=" * 80)
        return 1

if __name__ == '__main__':
    sys.exit(main())
